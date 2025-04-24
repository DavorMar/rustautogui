use crate::AutoGuiError;
use ocl::{Buffer, Queue};

#[derive(Debug)]
pub struct KernelStorage {
    pub v1_kernel: ocl::Kernel,
    pub v2_kernel_fast: ocl::Kernel,
}

impl KernelStorage {
    pub fn new(
        gpu_memory_pointers: &GpuMemoryPointers,
        program: &ocl::Program,
        queue: &ocl::Queue,
        image_width: u32,
        image_height: u32,
        template_width: u32,
        template_height: u32,
        fast_segment_count: u32,
        slow_segment_count: u32,
        segments_mean_fast: f32,
        segments_mean_slow: f32,
        segment_sum_squared_deviation_fast: f32,
        segment_sum_squared_deviation_slow: f32,
        fast_expected_corr: f32,
        max_workgroup_size: usize,
    ) -> Result<Self, AutoGuiError> {
        let result_width = (image_width - template_width + 1) as usize;
        let result_height = (image_height - template_height + 1) as usize;
        let output_size = result_width * result_height;
        let kernel_v1 = ocl::Kernel::builder()
            .program(&program)
            .name("segmented_match_integral")
            .queue(queue.clone())
            .global_work_size(output_size)
            .arg(&gpu_memory_pointers.buffer_image_integral)
            .arg(&gpu_memory_pointers.buffer_image_integral_squared)
            .arg(&gpu_memory_pointers.segments_fast_buffer)
            .arg(&gpu_memory_pointers.segments_slow_buffer)
            .arg(&gpu_memory_pointers.segment_fast_values_buffer)
            .arg(&gpu_memory_pointers.segment_slow_values_buffer)
            .arg(&(fast_segment_count as i32))
            .arg(&(slow_segment_count as i32))
            .arg(&(segments_mean_fast as f32))
            .arg(&(segments_mean_slow as f32))
            .arg(&(segment_sum_squared_deviation_fast as f32))
            .arg(&(segment_sum_squared_deviation_slow as f32))
            .arg(&gpu_memory_pointers.results_buffer)
            .arg(&(image_width as i32))
            .arg(&(image_height as i32))
            .arg(&(template_width as i32))
            .arg(&(template_height as i32))
            .arg(&(fast_expected_corr as f32 - 0.01))
            .arg(&gpu_memory_pointers.buffer_precision)
            .build()?;

        let mut remainder_segments_fast = 0;

        let mut segments_processed_by_thread_fast = 1;

        let mut pixels_processed_by_workgroup = 1;
        let max_workgroup_size = max_workgroup_size;

        // if we have more segments than workgroup size, then that workgroup only processes
        // that single pixel. Each thread inside workgroup processes certain amount of equally distributed segments
        if fast_segment_count as usize > max_workgroup_size {
            segments_processed_by_thread_fast = fast_segment_count as usize / max_workgroup_size;
            remainder_segments_fast = (fast_segment_count as usize % max_workgroup_size) as i32;
        // else, if we have low thread count then 1 workgroup can process multiple pixels. IE workgroup with 256 threads
        // can process 64 pixels with 4 segments
        } else {
            pixels_processed_by_workgroup = max_workgroup_size / fast_segment_count as usize;
            // threads per pixel = fast_segmented_count
        }
        let global_workgroup_count =
            (output_size + pixels_processed_by_workgroup - 1) / pixels_processed_by_workgroup;
        // total amount of threads that need to be spawned
        let global_work_size = global_workgroup_count as usize * max_workgroup_size;

        // if the workgroup finds a succesfull correlation with fast pass, it will have to calculate it
        // with the slow pass aswell for that same x,y pos. But if we had low fast segment count
        // that workgroup will not be utilized nicely.  Will have to rework this part

        let v2_kernel_fast_pass = ocl::Kernel::builder()
            .program(&program)
            .name("v2_segmented_match_integral_fast_pass")
            .queue(queue.clone())
            .global_work_size(global_work_size)
            .arg(&gpu_memory_pointers.buffer_image_integral)
            .arg(&gpu_memory_pointers.buffer_image_integral_squared)
            .arg(&gpu_memory_pointers.segments_fast_buffer)
            .arg(&gpu_memory_pointers.segment_fast_values_buffer)
            .arg(&(fast_segment_count as i32))
            .arg(&(segments_mean_fast as f32))
            .arg(&(segment_sum_squared_deviation_fast as f32))
            .arg(&gpu_memory_pointers.buffer_results_fast_v2) ///////////////////////CHANGE THIS TO ONE FROM GPUMEMPOINTERS STRUCT
            .arg(&(image_width as i32))
            .arg(&(image_height as i32))
            .arg(&(template_width as i32))
            .arg(&(template_height as i32))
            .arg(&(fast_expected_corr as f32) - 0.01)
            .arg(&remainder_segments_fast)
            .arg(&(segments_processed_by_thread_fast as i32))
            .arg(&(pixels_processed_by_workgroup as i32))
            .arg(&(max_workgroup_size as i32))
            .arg_local::<u64>(pixels_processed_by_workgroup) // sum_template_region_buff
            .arg_local::<u64>(pixels_processed_by_workgroup) // sum_sq_template_region_buff
            .arg_local::<u64>(max_workgroup_size) // thread_segment_sum_buff
            .arg(&gpu_memory_pointers.buffer_valid_corr_count_fast) // <-- atomic int
            .arg(&gpu_memory_pointers.buffer_precision)
            .build()?;

        Ok(Self {
            v1_kernel: kernel_v1,
            v2_kernel_fast: v2_kernel_fast_pass,
        })
    }
}

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

#[derive(Debug)]
pub struct DevicesInfo {
    device: ocl::Device,
    pub index: u32,
    pub global_mem_size: u32,
    pub clock_frequency: u32,
    pub compute_units: u32,
    pub brand: String,
    pub name: String,
    pub score: u32,
}
#[cfg(feature = "opencl")]
impl DevicesInfo {
    pub fn new(
        device: ocl::Device,
        index: u32,
        global_mem_size: u32,
        clock_frequency: u32,
        compute_units: u32,
        brand: String,
        name: String,
        score: u32,
    ) -> Self {
        Self {
            device,
            index,
            global_mem_size,
            clock_frequency,
            compute_units,
            brand,
            name,
            score,
        }
    }
}
