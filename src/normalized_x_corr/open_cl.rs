use crate::imgtools;
use crate::normalized_x_corr::{compute_integral_images, sum_region};
use image::{ImageBuffer, Luma};
use ocl::{Buffer, Context, Device, Kernel, Program, Queue};
use std::time;

use ocl;


/*
Different versions of OpenCL algorithm adaptation have been thought of or tried to implement but 
no success. Local memory usage is hard to utilize due to large integral images. Distributing work
into smaller tasks requires buffers to store values between those same tasks. The size 
of stored data can explode. For instance, 4000x3000 image x 50000 segments is like 4.5 TB of data.
reworking integral image algorithms requires additional calculations in a process that cannot be 
*/









// same algorithm as segmented but in OpenCL C
pub const OCL_KERNEL: &str = r#"
inline ulong sum_region(
    __global const ulong* integral,
    int x,
    int y,
    int width,
    int height,
    int image_width
) {
    int x2 = x + width - 1;
    int y2 = y + height - 1;

    ulong br = integral[y2 * image_width + x2];
    ulong bl = (x == 0) ? 0 : integral[y2 * image_width + (x - 1)];
    ulong tr = (y == 0) ? 0 : integral[(y - 1) * image_width + x2];
    ulong tl = (x == 0 || y == 0) ? 0 : integral[(y - 1) * image_width + (x - 1)];
    long sum = (long)br + (long)tl - (long)bl - (long)tr;


    return (ulong)sum;
}


inline ulong sum_region_squared(
    __global const ulong* integral_sq,
    int x,
    int y,
    int width,
    int height,
    int image_width
) {
    int x2 = x + width - 1;
    int y2 = y + height - 1;

    ulong br = integral_sq[y2 * image_width + x2];
    ulong bl = (x == 0) ? 0 : integral_sq[y2 * image_width + (x - 1)];
    ulong tr = (y == 0) ? 0 : integral_sq[(y - 1) * image_width + x2];
    ulong tl = (x == 0 || y == 0) ? 0 : integral_sq[(y - 1) * image_width + (x - 1)];
    long sum = (long)br + (long)tl - (long)bl - (long)tr;
    return (ulong)sum;
}


__kernel void segmented_match_integral(
    __global const ulong* integral,
    __global const ulong* integral_sq,
    __global const int4* segments,
    __global const float* segment_values,
    const int num_segments,
    const float template_mean,
    const float template_sq_dev,
    __global float* results,
    const int image_width,
    const int image_height,
    const int template_width,
    const int template_height,
    const float min_expected_corr,
    const int remainder_segments_fast,
    const int segments_per_thread_fast,
    const int pixels_per_workgroup,
    __local ulong* sum_template_region_buff,
    __local ulong* sum_sq_template_region_buff,
    __local ulong* thread_segment_sum_buff,
    __global atomic_int* valid_corr_count  
) {
    int global_id = get_global_id(0);
    int local_id = get_local_id(0);
    int workgroup_id = get_group_id(0);

    // num_segments is also count of threads per pixel for fast img
    if (local_id >= num_segments * pixels_per_workgroup) return; 


    int pixel_pos = (workgroup_id * pixels_per_workgroup) + (local_id / num_segments) ;
    int image_x = pixel_pos % image_width;
    int image_y = pixel_pos / image_width;

    int pixel_idx = local_id / num_segments;

    // first sum the region of template area for numerator calculations
    // we do it with first threads for each x,y position which workgroup processes
    // if there are 5 pixels processed, local_id 0-4 should process sum regions for each position, 5-9 for squared
    ulong patch_sum = 0;
    if (local_id < pixels_per_workgroup) {
        patch_sum = sum_region(integral, image_x, image_y, template_width, template_height, image_width);
    }
    
    // there will never be less than 2 segments 
    // meaning pixels per workgroup is never greater than workgroup_size / 2 
    if (local_id >= pixels_per_workgroup) && (local_id < pixels_per_workgroup * 2) {
        ulong patch_sq_sum = sum_region_squared(integral_sq, image_x, image_y, template_width, template_height, image_width);
        sum_sq_template_region_buff[pixel_idx] = patch_sq_sum;
    }
    
    int result_width = image_width - template_width + 1;
    int result_height = image_height - template_height + 1;
    float area = (float)(template_width * template_height);

    // wait  for threads to complete writing sum_area
    barrier(CLK_LOCAL_MEM_FENCE);

    
    float mean_img = (float)(sum_template_region_buff[pixel_idx]) / area;
    
    // AUDIT - DOUBLE CHECK THIS LOGIC
    int thread_segment_start = (local_id * segments_per_thread_fast) % num_segments;
    int thread_segment_end = thread_segment_start +  segments_per_thread_fast;

    float nominator = 0.0f;
    for (int i = thread_segment_start; i< thread_segment_end; i++) {
        int4 seg = segments[i];
        float seg_val = segment_values[i];
        int seg_area = seg.z* seg.w;
        ulong region_sum = sum_region(integral, image_x + seg.x, image_y + seg.y, seg.z, seg.w, image_width);
        nominator += ((float)(region_sum) - mean_img * seg_area) * (seg_val - template_mean);
    }
    thread_segment_sum_buff[local_id] = nominator;
    barrier(CLK_LOCAL_MEM_FENCE);

    if (local_id < pixels_per_workgroup) {
        float nominator_sum = 0.0f;
        int sum_start = local_id * num_segments;
        int sum_end = sum_start + (num_segments / segments_per_thread_fast );
        for (int i = sum_start; i< sum_end; i++) {
            nominator_sum = nominator_sum + thread_segment_sum_buff[i] ;
        }


        ulong patch_sq_sum_extracted = sum_sq_template_region_buff[local_id];
        float var_img = (float)(patch_sq_sum_extracted - (patch_sum*patch_sum))/area;

        float denominator = sqrt(var_img * patch_sq_sum_extracted);
        
        float corr = (denominator != 0.0f) ? (nominator_sum / denominator) : -1.0f;
        if (corr >= min_expected_corr) {
            int index = atomic_fetch_add(valid_corr_count, 1);
            results[index] = corr;
        }
    } 
}
"#;

pub struct GpuMemoryPointers {
    segments_fast_buffer: Buffer<ocl::prm::Int4>,
    segments_slow_buffer: Buffer<ocl::prm::Int4>,
    segment_fast_values_buffer: Buffer<f32>,
    segment_slow_values_buffer: Buffer<f32>,
    results_buffer: Buffer<f32>,
    buffer_image_integral: Buffer<u64>,
    buffer_image_integral_squared: Buffer<u64>,
}
impl GpuMemoryPointers {
    pub fn new(
        image_width: u32,
        image_height: u32,
        template_width: u32,
        template_height: u32,
        queue: &Queue,
        template_segments_slow: &[(u32, u32, u32, u32, f32)],
        template_segments_fast: &[(u32, u32, u32, u32, f32)],
    ) -> Result<Self, ocl::Error> {
        let result_width = (image_width - template_width + 1) as usize;
        let result_height = (image_height - template_height + 1) as usize;
        let output_size = result_width * result_height;
        let segment_fast_int4: Vec<ocl::prm::Int4> = template_segments_fast
            .iter()
            .map(|&(x, y, w, h, _)| ocl::prm::Int4::new(x as i32, y as i32, w as i32, h as i32))
            .collect();

        let segment_slow_int4: Vec<ocl::prm::Int4> = template_segments_slow
            .iter()
            .map(|&(x, y, w, h, _)| ocl::prm::Int4::new(x as i32, y as i32, w as i32, h as i32))
            .collect();

        let segment_values_fast: Vec<f32> = template_segments_fast
            .iter()
            .map(|&(_, _, _, _, v)| v)
            .collect();
        let segment_values_slow: Vec<f32> = template_segments_slow
            .iter()
            .map(|&(_, _, _, _, v)| v)
            .collect();

        let buffer_segments_fast: Buffer<ocl::prm::Int4> = Buffer::<ocl::prm::Int4>::builder()
            .queue(queue.clone())
            .len(segment_fast_int4.len())
            .copy_host_slice(&segment_fast_int4)
            .build()?;

        let buffer_segments_slow: Buffer<ocl::prm::Int4> = Buffer::<ocl::prm::Int4>::builder()
            .queue(queue.clone())
            .len(segment_slow_int4.len())
            .copy_host_slice(&segment_slow_int4)
            .build()?;

        let buffer_segment_values_fast: Buffer<f32> = Buffer::<f32>::builder()
            .queue(queue.clone())
            .len(segment_values_fast.len())
            .copy_host_slice(&segment_values_fast)
            .build()?;

        let buffer_segment_values_slow: Buffer<f32> = Buffer::<f32>::builder()
            .queue(queue.clone())
            .len(segment_values_slow.len())
            .copy_host_slice(&segment_values_slow)
            .build()?;

        let buffer_results = Buffer::<f32>::builder()
            .queue(queue.clone())
            .len(output_size)
            .build()?;

        let buffer_image_integral = Buffer::<u64>::builder()
            .queue(queue.clone())
            .len(image_width * image_height)
            .build()?;

        let buffer_image_integral_squared = Buffer::<u64>::builder()
            .queue(queue.clone())
            .len(image_width * image_height)
            .build()?;
        Ok(Self {
            segments_fast_buffer: buffer_segments_fast,
            segments_slow_buffer: buffer_segments_slow,
            segment_fast_values_buffer: buffer_segment_values_fast,
            segment_slow_values_buffer: buffer_segment_values_slow,
            results_buffer: buffer_results,
            buffer_image_integral,
            buffer_image_integral_squared,
        })
    }
}

pub fn gui_opencl_ncc_template_match(
    queue: &Queue,
    program: &Program,
    max_workgroup_size: u32,
    gpu_memory_pointers: &GpuMemoryPointers,
    precision: f32,
    image: &ImageBuffer<Luma<u8>, Vec<u8>>,
    template_data: &(
        Vec<(u32, u32, u32, u32, f32)>, // fast segments (x, y, w, h, val)
        Vec<(u32, u32, u32, u32, f32)>, // slow segments (x, y, w, h, val)
        u32,                            // template width
        u32,                            // template height
        f32,                            // fast sum_squared_deviations
        f32,                            // slow sum_squared_deviations
        f32,                            // fast expected corr
        f32,                            // slow expected corr
        f32,                            // fast mean
        f32,                            // slow mean
    ),
) -> ocl::Result<Vec<(u32, u32, f32)>> {
    let (image_width, image_height) = image.dimensions();
    let (image_integral, squared_image_integral) = compute_integral_images_ocl(&image);

    let (
        template_segments_fast,
        template_segments_slow,
        template_width,
        template_height,
        fast_segments_sum_squared_deviations,
        slow_segments_sum_squared_deviations,
        fast_expected_corr,
        slow_expected_corr,
        segments_mean_fast,
        segments_mean_slow,
    ) = template_data;
    let min_expected_corr = precision * *fast_expected_corr - 0.01;
    let slow_expected_corr = precision * *slow_expected_corr - 0.001;
    let fast_segment_count = template_segments_fast.len();
    let slow_segment_count = template_segments_slow.len();



    let mut remainder_segments_fast = 0;
    let mut remainder_segments_slow = 0;
    let mut segments_processed_by_thread_fast = 1;
    let mut segments_processed_by_thread_slow = 1;
    let mut pixels_processed_by_workgroup = 1;
    let mut threads_per_pixel = max_workgroup_size;;
    let max_workgroup_size = max_workgroup_size as usize;
    
    // if we have more segments than workgroup size, then that workgroup only processes
    // that single pixel. Each thread inside workgroup processes certain amount of equally distributed segments
    if fast_segment_count > max_workgroup_size {
        segments_processed_by_thread_fast = fast_segment_count / max_workgroup_size;
        remainder_segments_fast = fast_segment_count % max_workgroup_size;
    // else, if we have low thread count then 1 workgroup can process multiple pixels. IE workgroup with 256 threads 
    // can process 64 pixels with 4 segments
    } else {
        pixels_processed_by_workgroup = max_workgroup_size / fast_segment_count;
        // threads per pixel = fast_segmented_count   
    }

    // if the workgroup finds a succesfull correlation with fast pass, it will have to calculate it 
    // with the slow pass aswell for that same x,y pos. But if we had low fast segment count
    // that workgroup will not be utilized nicely.  Will have to rework this part
    
    let total_slow_segment_count_in_workgroup = slow_segment_count * pixels_processed_by_workgroup;
    if total_slow_segment_count_in_workgroup > max_workgroup_size {
        segments_processed_by_thread_slow = slow_segment_count / max_workgroup_size;
        remainder_segments_slow = slow_segment_count % max_workgroup_size;
    } else {
        
    }   

    let result_width = (image_width - template_width + 1) as usize;
    let result_height = (image_height - template_height + 1) as usize;
    let output_size = result_width * result_height;
    // round up division for how many workgroups needs to be spawned
    let global_workgroup_count = (output_size * pixels_processed_by_workgroup - 1) / pixels_processed_by_workgroup;
    // total amount of threads that need to be spawned
    let global_work_size = global_workgroup_count * max_workgroup_size;



    let mut gpu_results = gui_opencl_ncc(
        &image_integral,
        &squared_image_integral,
        image_width,
        image_height,
        *template_width,
        *template_height,
        *fast_segments_sum_squared_deviations,
        *slow_segments_sum_squared_deviations,
        *segments_mean_fast,
        *segments_mean_slow,
        min_expected_corr,
        queue,
        program,
        gpu_memory_pointers,
        fast_segment_count as i32,
        slow_segment_count as i32,
        remainder_segments_fast as i32,
        remainder_segments_slow as i32,
        segments_processed_by_thread_fast as i32,
        segments_processed_by_thread_slow as i32,
        pixels_processed_by_workgroup as i32,
        global_work_size as i32,
        max_workgroup_size as i32,

    )?;
    gpu_results.retain(|&(_, _, value)| value >= slow_expected_corr);
    gpu_results.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
    Ok(gpu_results)
}

pub fn gui_opencl_ncc(
    image_integral: &[u64],
    squared_image_integral: &[u64],
    image_width: u32,
    image_height: u32,
    template_width: u32,
    template_height: u32,
    segments_sum_squared_deviation_fast: f32,
    segments_sum_squared_deviation_slow: f32,
    segments_mean_fast: f32,
    segments_mean_slow: f32,
    fast_expected_corr: f32,
    queue: &Queue,
    program: &Program,
    gpu_memory_pointers: &GpuMemoryPointers,
    fast_segment_count: i32,
    slow_segment_count: i32,
    remainder_segments_fast: i32,
    remainder_segments_slow:i32,
    segments_processed_by_thread_fast: i32,
    segments_processed_by_thread_slow: i32,
    pixels_processed_by_workgroup: i32,
    global_work_size: i32,
    workgroup_size: i32
) -> ocl::Result<Vec<(u32, u32, f32)>> {
    let result_width = (image_width - template_width + 1) as usize;
    let result_height = (image_height - template_height + 1) as usize;
    let output_size = result_width * result_height;
    gpu_memory_pointers
        .buffer_image_integral
        .write(image_integral)
        .enq()?;
    gpu_memory_pointers
        .buffer_image_integral_squared
        .write(squared_image_integral)
        .enq()?;
    let kernel = Kernel::builder()
        .program(&program)
        .name("segmented_match_integral")
        .queue(queue.clone())
        .global_work_size(output_size)
        .arg(&gpu_memory_pointers.buffer_image_integral)
        .arg(&gpu_memory_pointers.buffer_image_integral_squared)
        .arg(&gpu_memory_pointers.segments_fast_buffer)
        .arg(&gpu_memory_pointers.segment_fast_values_buffer)
        .arg(&fast_segment_count)
        .arg(&(segments_mean_fast as f32))
        .arg(&(segments_sum_squared_deviation_fast as f32))
        .arg(&gpu_memory_pointers.results_buffer)
        .arg(&(image_width as i32))
        .arg(&(image_height as i32))
        .arg(&(template_width as i32))
        .arg(&(template_height as i32))
        .arg(&(fast_expected_corr as f32))
        .arg(&remainder_segments_fast)
        .arg(&segments_processed_by_thread_fast)
        .arg(&pixels_processed_by_workgroup)
        .arg_local::<u64>(pixels_processed_by_workgroup as usize)
        .arg_local::<u64>(pixels_processed_by_workgroup as usize)
        .arg_local::<u64>(workgroup_size as usize)
        .arg_local::<u64>(pixels_processed_by_workgroup as usize)
        .build()?;

    unsafe {
        kernel.enq()?;
    }
    let mut valid_correlations = 
    let mut results = vec![0.0f32; output_size];
    gpu_memory_pointers
        .results_buffer
        .read(&mut results)
        .enq()?;

    let final_results: Vec<(u32, u32, f32)> = results
        .into_iter()
        .enumerate()
        .map(|(idx, corr)| {
            let x = (idx % result_width) as u32;
            let y = (idx / result_width) as u32;
            (x, y, corr)
        })
        .collect();

    Ok(final_results)
}

fn compute_integral_images_ocl(image: &ImageBuffer<Luma<u8>, Vec<u8>>) -> (Vec<u64>, Vec<u64>) {
    let (width, height) = image.dimensions();
    let image = image.as_raw();
    let mut integral_image = vec![0u64; (width * height) as usize];
    let mut squared_integral_image = vec![0u64; (width * height) as usize];
    for y in 0..height {
        for x in 0..width {
            let pixel_value = image[(y * width + x) as usize] as u64;
            let pixel_value_squared = (image[(y * width + x) as usize] as u64).pow(2);
            let (integral_value, squared_integral_value) = if x == 0 && y == 0 {
                (pixel_value, pixel_value_squared)
            } else if x == 0 {
                (
                    pixel_value + integral_image[((y - 1) * width + x) as usize],
                    pixel_value_squared + squared_integral_image[((y - 1) * width + x) as usize],
                )
            } else if y == 0 {
                (
                    pixel_value + integral_image[(y * width + (x - 1)) as usize],
                    pixel_value_squared + squared_integral_image[(y * width + (x - 1)) as usize],
                )
            } else {
                (
                    pixel_value
                        + integral_image[((y - 1) * width + x) as usize]
                        + integral_image[(y * width + (x - 1)) as usize]
                        - integral_image[((y - 1) * width + (x - 1)) as usize],
                    pixel_value_squared
                        + squared_integral_image[((y - 1) * width + x) as usize]
                        + squared_integral_image[(y * width + (x - 1)) as usize]
                        - squared_integral_image[((y - 1) * width + (x - 1)) as usize],
                )
            };
            integral_image[(y * width + x) as usize] = integral_value;
            squared_integral_image[(y * width + x) as usize] = squared_integral_value;
        }
    }

    (integral_image, squared_integral_image)
}
