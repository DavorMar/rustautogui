use crate::normalized_x_corr::{compute_integral_images, sum_region};
use crate::{imgtools, KernelStorage};
use image::{ImageBuffer, Luma};
use ocl::{Buffer, Context, Kernel, Program, Queue};
use std::time;

use ocl;

use super::opencl_v2;

#[derive(Debug)]
pub struct GpuMemoryPointers {
    pub segments_fast_buffer: Buffer<ocl::prm::Int4>,
    pub segments_slow_buffer: Buffer<ocl::prm::Int4>,
    pub segment_fast_values_buffer: Buffer<f32>,
    pub segment_slow_values_buffer: Buffer<f32>,
    pub results_buffer: Buffer<f32>,
    pub buffer_image_integral: Buffer<u64>,
    pub buffer_image_integral_squared: Buffer<u64>,
    pub buffer_results_fast_v2: Buffer<ocl::core::Int2>,
    pub buffer_results_slow_positions_v2: Buffer<ocl::core::Int2>,
    pub buffer_results_slow_corrs_v2: Buffer<f32>,
    pub buffer_valid_corr_count_fast: Buffer<i32>,
    pub buffer_valid_corr_count_slow: Buffer<i32>,
    pub buffer_precision: Buffer<f32>,
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

        // BUFFERS FOR v2 ALGORITHM ADDITIONALLY
        let buffer_results_fast = Buffer::<ocl::core::Int2>::builder()
            .queue(queue.clone())
            .len(output_size)
            .build()?;

        let buffer_results_slow_positions = Buffer::<ocl::core::Int2>::builder()
            .queue(queue.clone())
            .len(output_size)
            .build()?;

        let buffer_results_slow_corrs = Buffer::<f32>::builder()
            .queue(queue.clone())
            .len(output_size)
            .build()?;

        let valid_corr_count_buf_fast: Buffer<i32> = Buffer::builder()
            .queue(queue.clone())
            .flags(ocl::flags::MEM_READ_WRITE)
            .len(1)
            .fill_val(0i32) // Init to 0
            .build()?;

        let valid_corr_count_buf_slow: Buffer<i32> = Buffer::builder()
            .queue(queue.clone())
            .flags(ocl::flags::MEM_READ_WRITE)
            .len(1)
            .fill_val(0i32) // Init to 0
            .build()?;

        let precision_buff: Buffer<f32> = Buffer::builder()
            .queue(queue.clone())
            .flags(ocl::flags::MEM_READ_WRITE)
            .len(1)
            .fill_val(0.99) // Init to 0
            .build()?;

        Ok(Self {
            segments_fast_buffer: buffer_segments_fast,
            segments_slow_buffer: buffer_segments_slow,
            segment_fast_values_buffer: buffer_segment_values_fast,
            segment_slow_values_buffer: buffer_segment_values_slow,
            results_buffer: buffer_results,
            buffer_image_integral,
            buffer_image_integral_squared,
            buffer_results_fast_v2: buffer_results_fast,
            buffer_results_slow_positions_v2: buffer_results_slow_positions,
            buffer_results_slow_corrs_v2: buffer_results_slow_corrs,
            buffer_valid_corr_count_fast: valid_corr_count_buf_fast,
            buffer_valid_corr_count_slow: valid_corr_count_buf_slow,
            buffer_precision: precision_buff,
        })
    }
}

pub fn gui_opencl_ncc_template_match(
    queue: &Queue,
    program: &Program,
    max_workgroup_size: u32,
    kernel_storage: &KernelStorage,
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
        f32,
        bool, // slow mean
    ),
) -> ocl::Result<Vec<(u32, u32, f32)>> {
    let (image_width, image_height) = image.dimensions();

    let (image_integral, squared_image_integral) = compute_integral_images_ocl(&image);

    let (
        _,
        template_segments_slow,
        template_width,
        template_height,
        _,
        slow_segments_sum_squared_deviations,
        _,
        slow_expected_corr,
        _,
        segments_mean_slow,
        used_threshold,
    ) = template_data;
    let slow_expected_corr = precision * (*slow_expected_corr - 0.001);
    let mut gpu_results: Vec<(u32, u32, f32)> = Vec::new();
    match used_threshold {
        false => {
            let kernel = &kernel_storage.v1_kernel;
            gpu_results = gui_opencl_ncc(
                kernel,
                &image_integral,
                &squared_image_integral,
                image_width,
                image_height,
                *template_width,
                *template_height,
                gpu_memory_pointers,
                precision,
            )?;
            gpu_results.retain(|&(_, _, value)| value >= slow_expected_corr);
            gpu_results.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        }
        true => {
            let slow_segment_count = template_segments_slow.len() as i32;
            let kernel = &kernel_storage.v2_kernel_fast;
            let segments_processed_by_thread_slow = slow_segment_count / max_workgroup_size as i32;
            let remainder_segments_slow = slow_segment_count % max_workgroup_size as i32;
            gpu_results = opencl_v2::gui_opencl_ncc_v2(
                kernel,
                &image_integral,
                &squared_image_integral,
                image_width,
                image_height,
                *template_width,
                *template_height,
                *slow_segments_sum_squared_deviations,
                *segments_mean_slow,
                slow_expected_corr,
                queue,
                program,
                gpu_memory_pointers,
                slow_segment_count,
                remainder_segments_slow,
                segments_processed_by_thread_slow,
                max_workgroup_size as i32,
                precision,
            )?;
        }
    }

    Ok(gpu_results)
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
