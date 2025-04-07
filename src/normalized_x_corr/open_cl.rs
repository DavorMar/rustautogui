use ocl::{Buffer, Context, Kernel, Program, Queue};
use image::{ImageBuffer, Luma};
use crate::normalized_x_corr::{compute_integral_images, sum_region};
use crate::imgtools;
use std::time;

use ocl;

pub fn hybrid_segmented_template_match(
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
) -> ocl::Result<Vec<(u32, u32, f64)>> {
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
    
    let gpu_results = opencl_ncc(
        &flat_integral,
        &flat_squared_integral,
        image_width,
        image_height,
        *template_width,
        *template_height,
        template_segments_fast,
        *fast_segments_sum_squared_deviations,
        *segments_mean_fast
    )?;
    
    
    let mut final_matches = Vec::new();
    
    for (x, y, fast_corr) in gpu_results {
        if (fast_corr >= min_expected_corr ) && (fast_corr < 1.1){
            println!("x,y , corr: {}, {}, {}", x, y, fast_corr);
            let refined_corr = fast_correlation_calculation(
                &image_integral,
                &squared_image_integral,
                template_segments_slow,
                *template_width,
                *template_height,
                *slow_segments_sum_squared_deviations,
                *segments_mean_slow,
                x,
                y,
            );

            if refined_corr >= slow_expected_corr as f64 {
                final_matches.push((x, y, refined_corr));
            }
        }
    }

    Ok(final_matches)
}

pub fn fast_correlation_calculation(
    image_integral: &[Vec<u64>],
    squared_image_integral: &[Vec<u64>],
    
    template_segments_slow: &[(u32, u32, u32, u32, f32)], // precisely segmented, high number of segments
    template_width: u32,
    template_height: u32,
    
    slow_segments_sum_squared_deviations: f32,
    
    segments_slow_mean: f32,
    x: u32, // big image x value
    y: u32, // big image y value
    
) -> f64 {
    let template_area = template_height * template_width;

    /////////// numerator calculation
    let sum_image: u64 = sum_region(image_integral, x, y, template_width, template_height);
    let mean_image = sum_image as f32 / (template_height * template_width) as f32;
    let mut nominator = 0.0;

    

    // if nominator <= 0.0 {
    //     return -1.0;
    // }

    ////////// denominator calculation

    let sum_squared_image: u64 = sum_region(
        squared_image_integral,
        x,
        y,
        template_width,
        template_height,
    );
    let image_sum_squared_deviations =
        sum_squared_image as f32 - (sum_image as f32).powi(2) / template_area as f32;
   
    ///////////////

    let mut corr: f32 = -1.0;

    
    // second calculation with more detailed picture
    
        
    for (x1, y1, segment_width, segment_height, segment_value) in template_segments_slow {
        let segment_image_sum = sum_region(
            image_integral,
            x + x1,
            y + y1,
            *segment_width,
            *segment_height,
        );
        let segment_nominator_value: f32 = (segment_image_sum as f32
            - mean_image * (segment_height * segment_width) as f32)
            * (*segment_value as f32 - segments_slow_mean as f32);

    
        nominator += segment_nominator_value;
    }

    // if nominator <= 0.0 {
    //     return -1.0;
    // }

    let denominator =
        (image_sum_squared_deviations * slow_segments_sum_squared_deviations).sqrt();

    corr = nominator / denominator;
    
    if corr > 1.1 || corr.is_nan() {
        corr = -100.0;
        return corr as f64;
    }

    corr as f64
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
    segments_sum_squared_deviation: f32,
    segments_mean: f32,
) -> ocl::Result<Vec<(u32, u32, f32)>> {
    use ocl::{Context, Queue, Program, Buffer, Kernel};

    let context = Context::builder().build()?;
    let queue = Queue::new(&context, context.devices()[0], None)?;

    let program_source = r#"
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

        ulong sum = integral[y2 * image_width + x2];

        if (x == 0 && y == 0) {
            // nothing to subtract
        } else if (y == 0) {
            sum -= integral[y2 * image_width + (x - 1)];
        } else if (x == 0) {
            sum -= integral[(y - 1) * image_width + x2];
        } else {
            sum += integral[(y - 1) * image_width + (x - 1)];
            sum -= integral[y2 * image_width + (x - 1)];
            sum -= integral[(y - 1) * image_width + x2];
        }

        return sum;
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
        const int template_height
    ) {
        int idx = get_global_id(0);
        int result_width = image_width - template_width + 1;
        int result_height = image_height - template_height + 1;

        if (idx >= result_width * result_height) return;

        int x = idx % result_width;
        int y = idx / result_width;

        ulong patch_sum = sum_region(integral, image_width, x, y, template_width, template_height);
        ulong patch_sq_sum = sum_region(integral_sq, image_width, x, y, template_width, template_height);
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

        results[idx] = corr;
    }
    "#;

    let program = Program::builder().src(program_source).build(&context)?;

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
        .copy_host_slice(image_integral)
        .build()?;

    let segment_int4: Vec<ocl::prm::Int4> = segments
        .iter()
        .map(|&(x, y, w, h, _)| ocl::prm::Int4::new(x as i32, y as i32, w as i32, h as i32))
        .collect();

    let segment_values: Vec<f32> = segments.iter().map(|&(_, _, _, _, v)| v).collect();

    let buffer_segments = Buffer::<ocl::prm::Int4>::builder()
        .queue(queue.clone())
        .len(segment_int4.len())
        .copy_host_slice(&segment_int4)
        .build()?;

    let buffer_segment_values = Buffer::<f32>::builder()
        .queue(queue.clone())
        .len(segment_values.len())
        .copy_host_slice(&segment_values)
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
        .arg(&buffer_segment_values)
        .arg(&(segment_int4.len() as i32))
        .arg(&(segments_mean as f32))
        .arg(&(segments_sum_squared_deviation as f32))
        .arg(&buffer_results)
        .arg(&(image_width as i32))
        .arg(&(image_height as i32))
        .arg(&(template_width as i32))
        .arg(&(template_height as i32))
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