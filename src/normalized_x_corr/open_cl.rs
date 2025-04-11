use ocl::{Buffer, Context, Kernel, Program, Queue};
use image::{ImageBuffer, Luma};
use crate::normalized_x_corr::{compute_integral_images, sum_region};
use crate::imgtools;
use std::time;

use ocl;

/// same algorithm as segmented but in OpenCL C
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
    __global const int4* segments_slow,
    __global const float* segment_values,
    __global const float* segment_values_slow,
    const int num_segments,
    const int num_segments_slow,
    const float template_mean,
    const float template_mean_slow,
    const float template_sq_dev,
    const float template_sq_dev_slow,
    __global float* results,
    const int image_width,
    const int image_height,
    const int template_width,
    const int template_height,
    const float min_expected_corr
) {
    int idx = get_global_id(0);
    int result_width = image_width - template_width + 1;
    int result_height = image_height - template_height + 1;

    if (idx >= result_width * result_height) return;

    int x = idx % result_width;
    int y = idx / result_width;


    ulong patch_sum = sum_region(integral, x, y, template_width, template_height, image_width);
    ulong patch_sq_sum = sum_region_squared(integral_sq, x, y, template_width, template_height, image_width);
    


    float area = (float)(template_width * template_height);
    float mean_img = (float)(patch_sum) / area;
    float var_img = (float)(patch_sq_sum) - ((float)(patch_sum) * (float)(patch_sum)) / area;
    
    float nominator = 0.0f;
    for (int i = 0; i < num_segments; i++) {
        int4 seg = segments[i];
        float seg_val = segment_values[i];
        int seg_area = seg.z * seg.w;

        ulong region_sum = sum_region(integral, x + seg.x, y + seg.y, seg.z, seg.w, image_width);

        nominator += ((float)(region_sum) - mean_img * seg_area) * (seg_val - template_mean);
    }

    float denominator = sqrt(var_img * template_sq_dev);
    
    float corr = (denominator != 0.0f) ? (nominator / denominator) : -1.0f;



    if (corr < min_expected_corr) {
        results[idx] = corr;
        return;
    } else {
        float denominator_slow = sqrt(var_img * template_sq_dev_slow);
        float nominator_slow = 0.0f;
        for (int i = 0; i < num_segments_slow; i++) {
            int4 seg_slow = segments_slow[i];
            float seg_val_slow = segment_values_slow[i];
            int seg_area = seg_slow.z * seg_slow.w;

            ulong region_sum = sum_region(integral, x + seg_slow.x, y + seg_slow.y, seg_slow.z, seg_slow.w, image_width);

            nominator_slow += ((float)(region_sum) - mean_img * seg_area) * (seg_val_slow - template_mean);
        }
        float corr_slow = (denominator_slow != 0.0f) ? (nominator_slow / denominator_slow) : -1.0f;
        results[idx] = corr_slow;
    }    
}
"#;



pub struct GpuMemoryPointers {
    segments_fast_buffer: Buffer<ocl::prm::Int4>,
    segments_slow_buffer: Buffer<ocl::prm::Int4>,
    segment_fast_values_buffer: Buffer<f32>,
    segment_slow_values_buffer: Buffer<f32>,
    results_buffer: Buffer<f32>,
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
    ) -> Result<Self,ocl::Error> {
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

        let segment_values_fast: Vec<f32> = template_segments_fast.iter().map(|&(_, _, _, _, v)| v).collect();
        let segment_values_slow: Vec<f32> = template_segments_slow.iter().map(|&(_, _, _, _, v)| v).collect();

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
        Ok(Self {
            segments_fast_buffer: buffer_segments_fast,
            segments_slow_buffer: buffer_segments_slow,
            segment_fast_values_buffer: buffer_segment_values_fast,
            segment_slow_values_buffer: buffer_segment_values_slow,
            results_buffer: buffer_results,
        })
    }
}

pub fn gui_opencl_ncc_template_match (
    queue: &Queue,
    program: &Program,
    gpu_memory_pointers: &GpuMemoryPointers,
    precision: f32,
    image: &ImageBuffer<Luma<u8>, Vec<u8>>,
    template_data: &(
            Vec<(u32, u32, u32, u32, f32)>, // fast segments (x, y, w, h, val)
            Vec<(u32, u32, u32, u32, f32)>, // slow segments (x, y, w, h, val)
            u32,                             // template width
            u32,                             // template height
            f32,                             // fast sum_squared_deviations
            f32,                             // slow sum_squared_deviations
            f32,                             // fast expected corr
            f32,                             // slow expected corr
            f32,                             // fast mean
            f32,                             // slow mean
        )
) -> ocl::Result<Vec<(u32, u32, f32)>> {
    let (image_width, image_height) = image.dimensions();
    let image_vec: Vec<Vec<u8>> = imgtools::imagebuffer_to_vec(&image);
    let (image_integral, squared_image_integral) = compute_integral_images(&image_vec);
    

    let start = time::Instant::now();
    let flat_integral: Vec<u64> = image_integral
        .iter()
        .flat_map(|row| row.iter())
        .copied()
        .collect();
    let flat_squared_integral: Vec<u64> = squared_image_integral
        .iter()
        .flat_map(|row| row.iter())
        .copied()
        .collect();
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
    let min_expected_corr = precision * *fast_expected_corr - 0.001;
    let slow_expected_corr = precision * *slow_expected_corr - 0.001;
    let fast_segment_count = template_segments_fast.len();
    let slow_segment_count = template_segments_slow.len();

    let dur = start.elapsed().as_secs_f32();
    // println!("First part took : {}", dur);

    let start = time::Instant::now();
    let mut gpu_results = gui_opencl_ncc(
        &flat_integral,
        &flat_squared_integral,
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
    )?;
    let dur = start.elapsed().as_secs_f32();
    // println!("whole gpu results part took : {}", dur);
    gpu_results.retain(|&(_, _, value)| value >= slow_expected_corr);

    

    gpu_results.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
    let dur = start.elapsed().as_secs_f32();
    // println!("whole gpu results part 2 took : {}", dur);
    Ok(gpu_results)
    


}




pub fn fast_ncc_template_match_ocl(
    image: &ImageBuffer<Luma<u8>, Vec<u8>>,
    precision: f32,
    template_data: &(
        Vec<(u32, u32, u32, u32, f32)>, // fast segments (x, y, w, h, val)
        Vec<(u32, u32, u32, u32, f32)>, // slow segments (x, y, w, h, val)
        u32,                             // template width
        u32,                             // template height
        f32,                             // fast sum_squared_deviations
        f32,                             // slow sum_squared_deviations
        f32,                             // fast expected corr
        f32,                             // slow expected corr
        f32,                             // fast mean
        f32,                             // slow mean
    ),
    debug: &bool,
) -> ocl::Result<Vec<(u32, u32, f32)>> {
    let (image_width, image_height) = image.dimensions();
    let image_vec: Vec<Vec<u8>> = imgtools::imagebuffer_to_vec(&image);
    let (image_integral, squared_image_integral) = compute_integral_images(&image_vec);
    
    let flat_integral: Vec<u64> = image_integral
        .iter()
        .flat_map(|row| row.iter())
        .copied()
        .collect();
    let flat_squared_integral: Vec<u64> = squared_image_integral
        .iter()
        .flat_map(|row| row.iter())
        .copied()
        .collect();
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

    let min_expected_corr = precision * *fast_expected_corr - 0.001;
    let slow_expected_corr = precision * *slow_expected_corr - 0.001;
    
    let mut gpu_results = opencl_ncc(
        &flat_integral,
        &flat_squared_integral,
        image_width,
        image_height,
        *template_width,
        *template_height,
        template_segments_fast,
        template_segments_slow,
        *fast_segments_sum_squared_deviations,
        *slow_segments_sum_squared_deviations,
        *segments_mean_fast,
        *segments_mean_slow,
        min_expected_corr
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
    fast_expected_corr:f32,
    queue: &Queue,
    program: &Program,
    gpu_memory_pointers: &GpuMemoryPointers,
    fast_segment_count: i32,
    slow_segment_count: i32
) -> ocl::Result<Vec<(u32, u32, f32)>> {
    let start = time::Instant::now();
    let result_width = (image_width - template_width + 1) as usize;
    let result_height = (image_height - template_height + 1) as usize;
    let output_size = result_width * result_height;
    let buffer_image_integral = Buffer::<u64>::builder()
        .queue(queue.clone())
        .len(image_integral.len())
        .copy_host_slice(image_integral)
        .build()?;

    let buffer_image_integral_squared = Buffer::<u64>::builder()
        .queue(queue.clone())
        .len(squared_image_integral.len())
        .copy_host_slice(squared_image_integral)
        .build()?;
    let dur = start.elapsed().as_secs_f32();
    println!("before kernel preparation took : {}", dur);
    let start = time::Instant::now();
    let kernel = Kernel::builder()
        .program(&program)
        .name("segmented_match_integral")
        .queue(queue.clone())
        .global_work_size(output_size)
        .arg(&buffer_image_integral)
        .arg(&buffer_image_integral_squared)
        .arg(&gpu_memory_pointers.segments_fast_buffer)
        .arg(&gpu_memory_pointers.segments_slow_buffer)
        .arg(&gpu_memory_pointers.segment_fast_values_buffer)
        .arg(&gpu_memory_pointers.segment_slow_values_buffer)
        .arg(&fast_segment_count)
        .arg(&slow_segment_count)
        .arg(&(segments_mean_fast as f32))
        .arg(&(segments_mean_slow as f32))
        .arg(&(segments_sum_squared_deviation_fast as f32))
        .arg(&(segments_sum_squared_deviation_slow as f32))
        .arg(&gpu_memory_pointers.results_buffer)
        .arg(&(image_width as i32))
        .arg(&(image_height as i32))
        .arg(&(template_width as i32))
        .arg(&(template_height as i32))
        .arg(&(fast_expected_corr as f32))
        .build()?;


    
    unsafe { kernel.enq()?; }
    let duration = start.elapsed().as_secs_f32();
    println!("Opencl part lasted: {}", duration);
    let mut results = vec![0.0f32; output_size];
    gpu_memory_pointers.results_buffer.read(&mut results).enq()?;

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


/// OpenCL-based segmented NCC template matching that returns a vector of (x, y, correlation).
pub fn opencl_ncc(
    image_integral: &[u64],
    squared_image_integral: &[u64],
    image_width: u32,
    image_height: u32,
    template_width: u32,
    template_height: u32,
    segments: &[(u32, u32, u32, u32, f32)], // (x, y, width, height, value)
    template_segments_slow: &[(u32, u32, u32, u32, f32)], 
    segments_sum_squared_deviation: f32,
    segments_sum_squared_deviation_slow: f32,
    segments_mean: f32,
    segments_mean_slow: f32,
    fast_expected_corr:f32,
) -> ocl::Result<Vec<(u32, u32, f32)>> {
    use ocl::{Context, Queue, Program, Buffer, Kernel};

    let context = Context::builder().build()?;
    let queue = Queue::new(&context, context.devices()[0], None)?;
    let program_source = OCL_KERNEL;
    let program: Program = Program::builder().src(program_source).build(&context)?;




    let result_width = (image_width - template_width + 1) as usize;
    let result_height = (image_height - template_height + 1) as usize;
    let output_size = result_width * result_height;

    let buffer_image_integral = Buffer::<u64>::builder()
        .queue(queue.clone())
        .len(image_integral.len())
        .copy_host_slice(image_integral)
        .build()?;

    let buffer_image_integral_squared = Buffer::<u64>::builder()
        .queue(queue.clone())
        .len(squared_image_integral.len())
        .copy_host_slice(squared_image_integral)
        .build()?;

    let segment_int4: Vec<ocl::prm::Int4> = segments
        .iter()
        .map(|&(x, y, w, h, _)| ocl::prm::Int4::new(x as i32, y as i32, w as i32, h as i32))
        .collect();

    let segment_slow_int4: Vec<ocl::prm::Int4> = template_segments_slow
        .iter()
        .map(|&(x, y, w, h, _)| ocl::prm::Int4::new(x as i32, y as i32, w as i32, h as i32))
        .collect();

    let segment_values: Vec<f32> = segments.iter().map(|&(_, _, _, _, v)| v).collect();
    let segment_values_slow: Vec<f32> = template_segments_slow.iter().map(|&(_, _, _, _, v)| v).collect();

    let buffer_segments: Buffer<ocl::prm::Int4> = Buffer::<ocl::prm::Int4>::builder()
        .queue(queue.clone())
        .len(segment_int4.len())
        .copy_host_slice(&segment_int4)
        .build()?;

    let buffer_segments_slow: Buffer<ocl::prm::Int4> = Buffer::<ocl::prm::Int4>::builder()
        .queue(queue.clone())
        .len(segment_slow_int4.len())
        .copy_host_slice(&segment_slow_int4)
        .build()?;

    let buffer_segment_values: Buffer<f32> = Buffer::<f32>::builder()
        .queue(queue.clone())
        .len(segment_values.len())
        .copy_host_slice(&segment_values)
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

    let start = time::Instant::now();
    let kernel = Kernel::builder()
        .program(&program)
        .name("segmented_match_integral")
        .queue(queue.clone())
        .global_work_size(output_size)
        .arg(&buffer_image_integral)
        .arg(&buffer_image_integral_squared)
        .arg(&buffer_segments)
        .arg(&buffer_segments_slow)
        .arg(&buffer_segment_values)
        .arg(&buffer_segment_values_slow)
        .arg(&(segment_int4.len() as i32))
        .arg(&(segment_slow_int4.len() as i32))
        .arg(&(segments_mean as f32))
        .arg(&(segments_mean_slow as f32))
        .arg(&(segments_sum_squared_deviation as f32))
        .arg(&(segments_sum_squared_deviation_slow as f32))
        .arg(&buffer_results)
        .arg(&(image_width as i32))
        .arg(&(image_height as i32))
        .arg(&(template_width as i32))
        .arg(&(template_height as i32))
        .arg(&(fast_expected_corr as f32))
        .build()?;

    unsafe { kernel.enq()?; }
    let duration = start.elapsed().as_secs_f32();
    println!("Opencl part lasted: {}", duration);
    let mut results = vec![0.0f32; output_size];
    buffer_results.read(&mut results).enq()?;

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