use crate::normalized_x_corr::{compute_integral_images, sum_region};
use crate::{imgtools, print_mouse_position};
use image::{ImageBuffer, Luma};
use ocl::{Buffer, Context, Device, Kernel, Program, Queue};
use std::time::{self, Duration};

use crate::normalized_x_corr::open_cl::{compute_integral_images_ocl, GpuMemoryPointers};
use ocl;
use ocl::core::Int2;

/*
Different versions of OpenCL algorithm adaptation have been thought of or tried to implement but
no success. Local memory usage is hard to utilize due to large integral images. Distributing work
into smaller tasks requires buffers to store values between those same tasks. The size
of stored data can explode. For instance, 4000x3000 image x 50000 segments is like 4.5 TB of data.
reworking integral image algorithms requires additional calculations in a process that cannot be
*/

// same algorithm as segmented but in OpenCL C
pub const OCL_KERNEL_V2: &str = r#"
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


__kernel void segmented_match_integral_fast_pass(
    __global const ulong* integral,
    __global const ulong* integral_sq,
    __global const int4* segments,
    __global const float* segment_values,
    const int num_segments,
    const float template_mean,
    const float template_sq_dev,
    __global int2* results,
    const int image_width,
    const int image_height,
    const int template_width,
    const int template_height,
    const float min_expected_corr,
    const int remainder_segments_fast,
    const int segments_per_thread_fast,
    const int pixels_per_workgroup,
    const int workgroup_size,
    __local ulong* sum_template_region_buff,
    __local ulong* sum_sq_template_region_buff,
    __local float* thread_segment_sum_buff,
    __global int* valid_corr_count
) {
    int global_id = get_global_id(0);
    int local_id = get_local_id(0);
    int workgroup_id = get_group_id(0);
    int result_w = image_width - template_width;
    

    // num_segments is also count of threads per pixel for fast img
    if (local_id * segments_per_thread_fast +  remainder_segments_fast >= num_segments * pixels_per_workgroup) return ; // this solves more segments per thread

    int pixel_pos = (workgroup_id * pixels_per_workgroup) + (local_id / num_segments) ;
    int image_x = pixel_pos % result_w;
    int image_y = pixel_pos / result_w;

    // first sum the region of template area for numerator calculations
    // we do it with first threads for each x,y position which workgroup processes
    // if there are 5 pixels processed, local_id 0-4 should process sum regions for each position, 5-9 for squared
    ulong patch_sum = 0;
    if (local_id < pixels_per_workgroup) {
        patch_sum = sum_region(integral, image_x, image_y, template_width, template_height, image_width);
        sum_template_region_buff[local_id] = patch_sum;
        
    }
    
    // there will never be less than 2 segments 
    // meaning pixels per workgroup is never greater than workgroup_size / 2 
    if (local_id >= pixels_per_workgroup && local_id < pixels_per_workgroup * 2) {
        ulong patch_sq_sum = sum_region_squared(integral_sq, image_x, image_y, template_width, template_height, image_width);
        sum_sq_template_region_buff[local_id % pixels_per_workgroup] = patch_sq_sum;
    }
    
    int result_width = image_width - template_width + 1;
    int result_height = image_height - template_height + 1;
    float area = (float)(template_width * template_height);

    // wait  for threads to complete writing sum_area
    barrier(CLK_LOCAL_MEM_FENCE);

    
    float mean_img = (float)(sum_template_region_buff[local_id / num_segments]) / area;


    // this is to cover if we have more than 1 segment per thread. This method 
    // with remainder allows us to keep all threads working
    int remainder_offset = 0;
    int remainder_addition = 0;
    if (remainder_segments_fast > 0) {
        if (local_id >= remainder_segments_fast) {
            remainder_offset = remainder_segments_fast;
        } else {
            remainder_offset = local_id;
            remainder_addition = 1; 
        }
    
    }

    
    
    // AUDIT - DOUBLE CHECK THIS LOGIC
    int thread_segment_start = (local_id * segments_per_thread_fast + remainder_offset ) % num_segments;
    int thread_segment_end = thread_segment_start +  segments_per_thread_fast + remainder_addition;

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
        int sum_end = sum_start + (num_segments / segments_per_thread_fast ) - (remainder_segments_fast/segments_per_thread_fast);
        for (int i = sum_start; i< sum_end; i++) {
            nominator_sum = nominator_sum + thread_segment_sum_buff[i] ;
        }

        int pixel_pos_final = (workgroup_id * pixels_per_workgroup) + (local_id) ;
        int image_x = pixel_pos_final % result_w;
        int image_y = pixel_pos_final / result_w;


        ulong patch_sq_sum_extracted = sum_sq_template_region_buff[local_id];
        float var_img = (float)patch_sq_sum_extracted - ((float)patch_sum * (float)patch_sum)/ (float)area;
        float denominator = sqrt(var_img * (float)template_sq_dev);
        float corr = (denominator != 0.0f) ? (nominator_sum / denominator) : -1.0f;        
        
        if (corr >= min_expected_corr - 0.005 && corr < 2) {
        
            int index = atomic_add(valid_corr_count, 1);
            results[index] = (int2)(image_x, image_y);
            
        }
    } 
}



__kernel void segmented_match_integral_slow_pass (
    __global const ulong* integral,
    __global const ulong* integral_sq,
    __global const int4* segments,
    __global const float* segment_values,
    const int num_segments,
    const float template_mean,
    const float template_sq_dev,
    __global int2* position_results,
    __global float* corr_results,
    const int image_width,
    const int image_height,
    const int template_width,
    const int template_height,
    const float min_expected_corr,
    const int remainder_segments_slow,
    const int segments_per_thread_slow,
    const int workgroup_size,
    __local ulong* sum_template_region_buff,
    __local ulong* sum_sq_template_region_buff,
    __local float* thread_segment_sum_buff,
    __global int* valid_corr_count,
    __global int* valid_corr_count_fast,
    __global int2* fast_pass_results
) {

    int global_id = get_global_id(0);
    int local_id = get_local_id(0);
    int workgroup_id = get_group_id(0);

    int image_x = fast_pass_results[workgroup_id].x;
    int image_y = fast_pass_results[workgroup_id].y;

    int result_w = image_width - template_width;

    // num_segments is also count of threads per pixel for fast img
    if (local_id * segments_per_thread_slow +  remainder_segments_slow >= num_segments) return ; // this solves more segments per thread


    // first sum the region of template area for numerator calculations
    // we do it with first threads for each x,y position which workgroup processes
    // if there are 5 pixels processed, local_id 0-4 should process sum regions for each position, 5-9 for squared
    ulong patch_sum = 0;
    if (local_id == 0) {
        patch_sum = sum_region(integral, image_x, image_y, template_width, template_height, image_width);
        sum_template_region_buff[0] = patch_sum;
        
    }
    
    // there will never be less than 2 segments 
    // meaning pixels per workgroup is never greater than workgroup_size / 2 
    if (local_id == 1) {
        ulong patch_sq_sum = sum_region_squared(integral_sq, image_x, image_y, template_width, template_height, image_width);
        sum_sq_template_region_buff[0] = patch_sq_sum;
    }
    int result_width = image_width - template_width + 1;
    int result_height = image_height - template_height + 1;
    float area = (float)(template_width * template_height);
    // wait  for threads to complete writing sum_area
    barrier(CLK_LOCAL_MEM_FENCE);
    float mean_img = (float)(sum_template_region_buff[0]) / area;
    // this is to cover if we have more than 1 segment per thread. This method 


    // with remainder allows us to keep all threads working
    int remainder_offset = 0;
    int remainder_addition = 0;
    if (remainder_segments_slow > 0) {
        if (local_id >= remainder_segments_slow) {
            remainder_offset = remainder_segments_slow;
        } else {
            remainder_offset = local_id;
            remainder_addition = 1; 
        }
    
    }

    int thread_segment_start = (local_id * segments_per_thread_slow + remainder_offset ) % num_segments;
    int thread_segment_end = thread_segment_start +  segments_per_thread_slow + remainder_addition;


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
    if (local_id == 0) {
        float nominator_sum = 0.0f;
        int sum_start = 0;
        int sum_end = sum_start + (num_segments / segments_per_thread_slow ) - (remainder_segments_slow/segments_per_thread_slow);
        for (int i = sum_start; i< sum_end; i++) {
            nominator_sum = nominator_sum + thread_segment_sum_buff[i] ;
        }

        


        ulong patch_sq_sum_extracted = sum_sq_template_region_buff[0];
        float var_img = (float)patch_sq_sum_extracted - ((float)patch_sum * (float)patch_sum)/ (float)area;
        float denominator = sqrt(var_img * (float)template_sq_dev);
        float corr = (denominator != 0.0f) ? (nominator_sum / denominator) : -1.0f;        

        if (corr >= min_expected_corr  && corr < 2) {
            int index = atomic_add(valid_corr_count, 1);
            position_results[index] = (int2)(image_x, image_y);
            corr_results[index] = corr;
        }
    } 
}

"#;

pub fn gui_opencl_ncc_template_match_v2(
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
    let fast_expected_corr = precision * *fast_expected_corr - 0.001;
    let slow_expected_corr = precision * *slow_expected_corr - 0.001;
    let fast_segment_count = template_segments_fast.len();
    let slow_segment_count = template_segments_slow.len();

    let mut remainder_segments_fast = 0;

    let mut segments_processed_by_thread_fast = 1;

    let mut pixels_processed_by_workgroup = 1;
    let max_workgroup_size = max_workgroup_size as usize;
    let mut remainder_segments_slow = 0;
    let mut segments_processed_by_thread_slow = 1;

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
    let global_workgroup_count =
        (output_size + pixels_processed_by_workgroup - 1) / pixels_processed_by_workgroup;
    // total amount of threads that need to be spawned
    let global_work_size = global_workgroup_count * max_workgroup_size;
    let mut gpu_results = gui_opencl_ncc_v2(
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
        fast_expected_corr,
        slow_expected_corr,
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
        global_work_size,
        max_workgroup_size as i32,
    )?;
    gpu_results.retain(|&(_, _, value)| value >= slow_expected_corr);
    gpu_results.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
    Ok(gpu_results)
}

pub fn gui_opencl_ncc_v2(
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
    slow_expected_corr: f32,
    queue: &Queue,
    program: &Program,
    gpu_memory_pointers: &GpuMemoryPointers,
    fast_segment_count: i32,
    slow_segment_count: i32,
    remainder_segments_fast: i32,
    remainder_segments_slow: i32,
    segments_processed_by_thread_fast: i32,
    segments_processed_by_thread_slow: i32,
    pixels_processed_by_workgroup: i32,
    global_work_size: usize,
    workgroup_size: i32,
) -> ocl::Result<Vec<(u32, u32, f32)>> {
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
        .name("segmented_match_integral_fast_pass")
        .queue(queue.clone())
        .global_work_size(global_work_size)
        .arg(&gpu_memory_pointers.buffer_image_integral)
        .arg(&gpu_memory_pointers.buffer_image_integral_squared)
        .arg(&gpu_memory_pointers.segments_fast_buffer)
        .arg(&gpu_memory_pointers.segment_fast_values_buffer)
        .arg(&fast_segment_count)
        .arg(&(segments_mean_fast as f32))
        .arg(&(segments_sum_squared_deviation_fast as f32))
        .arg(&gpu_memory_pointers.buffer_results_fast_v2) ///////////////////////CHANGE THIS TO ONE FROM GPUMEMPOINTERS STRUCT
        .arg(&(image_width as i32))
        .arg(&(image_height as i32))
        .arg(&(template_width as i32))
        .arg(&(template_height as i32))
        .arg(&(fast_expected_corr as f32))
        .arg(&remainder_segments_fast)
        .arg(&segments_processed_by_thread_fast)
        .arg(&pixels_processed_by_workgroup)
        .arg(&workgroup_size)
        .arg_local::<u64>(pixels_processed_by_workgroup as usize) // sum_template_region_buff
        .arg_local::<u64>(pixels_processed_by_workgroup as usize) // sum_sq_template_region_buff
        .arg_local::<u64>(workgroup_size as usize) // thread_segment_sum_buff
        .arg(&gpu_memory_pointers.buffer_valid_corr_count_fast) // <-- atomic int
        .build()?;

    unsafe {
        kernel.enq()?;
    }
    // get how many points have been found with fast pass
    let mut valid_corr_count_host = vec![0i32; 1];
    gpu_memory_pointers
        .buffer_valid_corr_count_fast
        .read(&mut valid_corr_count_host)
        .enq()?;
    let valid_corr_count = valid_corr_count_host[0] as usize;
    // gather those points
    if valid_corr_count > 0 {
        let mut fast_pass_positions = vec![ocl::core::Int2::zero(); valid_corr_count];
        gpu_memory_pointers
            .buffer_results_fast_v2
            .read(&mut fast_pass_positions)
            .enq()?;
    } else {
        let final_results: Vec<(u32, u32, f32)> = Vec::new();
        return Ok(final_results);
    }

    let new_global_work_size = valid_corr_count * workgroup_size as usize;

    let kernel_slow = Kernel::builder()
        .program(&program)
        .name("segmented_match_integral_slow_pass")
        .queue(queue.clone())
        .global_work_size(new_global_work_size)
        .arg(&gpu_memory_pointers.buffer_image_integral)
        .arg(&gpu_memory_pointers.buffer_image_integral_squared)
        .arg(&gpu_memory_pointers.segments_slow_buffer)
        .arg(&gpu_memory_pointers.segment_slow_values_buffer)
        .arg(&slow_segment_count)
        .arg(&(segments_mean_slow as f32))
        .arg(&(segments_sum_squared_deviation_slow as f32))
        .arg(&gpu_memory_pointers.buffer_results_slow_positions_v2)
        .arg(&gpu_memory_pointers.buffer_results_slow_corrs_v2)
        .arg(&(image_width as i32))
        .arg(&(image_height as i32))
        .arg(&(template_width as i32))
        .arg(&(template_height as i32))
        .arg(&(slow_expected_corr as f32))
        .arg(&remainder_segments_slow)
        .arg(&segments_processed_by_thread_slow)
        .arg(&workgroup_size)
        .arg_local::<u64>(1) // sum_template_region_buff
        .arg_local::<u64>(1) // sum_sq_template_region_buff
        .arg_local::<u64>(workgroup_size as usize) // thread_segment_sum_buff
        .arg(&gpu_memory_pointers.buffer_valid_corr_count_slow)
        .arg(&gpu_memory_pointers.buffer_valid_corr_count_fast) // <-- atomic int
        .arg(&gpu_memory_pointers.buffer_results_fast_v2)
        .build()?;
    unsafe {
        kernel_slow.enq()?;
    }

    let mut valid_corr_count_host_slow = vec![0i32; 1];
    gpu_memory_pointers
        .buffer_valid_corr_count_slow
        .read(&mut valid_corr_count_host_slow)
        .enq()?;
    let valid_corr_count_slow = valid_corr_count_host_slow[0] as usize;
    if valid_corr_count_slow > 0 {
        let mut slow_pass_corrs = vec![0.0; valid_corr_count_slow];
        let mut slow_pass_positions = vec![ocl::core::Int2::zero(); valid_corr_count_slow];
        gpu_memory_pointers
            .buffer_results_slow_positions_v2
            .read(&mut slow_pass_positions)
            .enq()?;

        gpu_memory_pointers
            .buffer_results_slow_corrs_v2
            .read(&mut slow_pass_corrs)
            .enq()?;

        let mut result_vec: Vec<(u32, u32, f32)> = slow_pass_positions
            .iter()
            .zip(slow_pass_corrs.iter())
            .map(|(pos, &corr)| (pos[0] as u32, pos[1] as u32, corr))
            .collect();

        result_vec
            .sort_unstable_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        return Ok(result_vec);
    } else {
        let final_results: Vec<(u32, u32, f32)> = Vec::new();
        return Ok(final_results);
    }
}
