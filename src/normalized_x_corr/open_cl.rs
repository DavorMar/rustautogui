use crate::data_structs::SegmentedData;
use crate::normalized_x_corr::{compute_integral_images, sum_region};
use crate::{
    data_structs::{GpuMemoryPointers, KernelStorage},
    imgtools,
};
use image::{ImageBuffer, Luma};
use ocl::{Buffer, Context, Kernel, Program, Queue};
use std::time;

use ocl;

use super::opencl_v2;

pub enum OclVersion {
    V1,
    V2,
}

pub fn gui_opencl_ncc_template_match(
    queue: &Queue,
    program: &Program,
    max_workgroup_size: u32,
    kernel_storage: &KernelStorage,
    gpu_memory_pointers: &GpuMemoryPointers,
    precision: f32,
    image: &ImageBuffer<Luma<u8>, Vec<u8>>,
    template_data: &SegmentedData,
    ocl_version: OclVersion,
) -> ocl::Result<Vec<(u32, u32, f32)>> {
    let (image_width, image_height) = image.dimensions();

    let (image_integral, squared_image_integral) = compute_integral_images_ocl(&image);

    let slow_expected_corr = precision * (template_data.expected_corr_slow - 0.001);
    match ocl_version {
        OclVersion::V1 => {
            let kernel = &kernel_storage.v1_kernel;
            let mut gpu_results = gui_opencl_ncc(
                kernel,
                &image_integral,
                &squared_image_integral,
                image_width,
                image_height,
                template_data.template_width,
                template_data.template_height,
                gpu_memory_pointers,
                precision,
            )?;
            gpu_results.retain(|&(_, _, value)| value >= slow_expected_corr);
            gpu_results.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
            return Ok(gpu_results);
        }
        OclVersion::V2 => {
            let slow_segment_count = template_data.template_segments_slow.len() as i32;
            let kernel = &kernel_storage.v2_kernel_fast;
            let segments_processed_by_thread_slow = slow_segment_count / max_workgroup_size as i32;
            let remainder_segments_slow = slow_segment_count % max_workgroup_size as i32;
            return opencl_v2::gui_opencl_ncc_v2(
                kernel,
                &image_integral,
                &squared_image_integral,
                image_width,
                image_height,
                template_data.template_width,
                template_data.template_height,
                template_data.segment_sum_squared_deviations_slow,
                template_data.segments_mean_slow,
                slow_expected_corr,
                queue,
                program,
                gpu_memory_pointers,
                slow_segment_count,
                remainder_segments_slow,
                segments_processed_by_thread_slow,
                max_workgroup_size as i32,
                precision,
            );
        }
    }
}

pub fn gui_opencl_ncc(
    kernel: &Kernel,
    image_integral: &[u64],
    squared_image_integral: &[u64],
    image_width: u32,
    image_height: u32,
    template_width: u32,
    template_height: u32,
    gpu_memory_pointers: &GpuMemoryPointers,
    precision: f32,
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

    gpu_memory_pointers
        .buffer_precision
        .write(&vec![precision])
        .enq()?;

    unsafe {
        kernel.enq()?;
    }
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

pub fn compute_integral_images_ocl(image: &ImageBuffer<Luma<u8>, Vec<u8>>) -> (Vec<u64>, Vec<u64>) {
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
