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

pub fn gui_opencl_ncc_v2(
    v2_kernel_fast_pass: &Kernel,
    image_integral: &[u64],
    squared_image_integral: &[u64],
    image_width: u32,
    image_height: u32,
    template_width: u32,
    template_height: u32,
    segments_sum_squared_deviation_slow: f32,
    segments_mean_slow: f32,
    slow_expected_corr: f32,
    queue: &Queue,
    program: &Program,
    gpu_memory_pointers: &GpuMemoryPointers,
    slow_segment_count: i32,
    remainder_segments_slow: i32,
    segments_processed_by_thread_slow: i32,
    workgroup_size: i32,
) -> ocl::Result<Vec<(u32, u32, f32)>> {
    println!("Using V2 !!");
    gpu_memory_pointers
        .buffer_image_integral
        .write(image_integral)
        .enq()?;
    gpu_memory_pointers
        .buffer_image_integral_squared
        .write(squared_image_integral)
        .enq()?;

    unsafe {
        v2_kernel_fast_pass.enq()?;
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

    // Some temporary value determined to limit count of threads - almost i32::max
    // if new_global_work_size >= 2_000_000_000 {
    //     return Err(ocl::Error::from("Too high global work size on slow pass. Try tuning your segmentation threshold higher up or use smaller template"));
    // }

    let v2_kernel_slow_pass = Kernel::builder()
        .program(&program)
        .name("v2_segmented_match_integral_slow_pass")
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
        v2_kernel_slow_pass.enq()?;
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
