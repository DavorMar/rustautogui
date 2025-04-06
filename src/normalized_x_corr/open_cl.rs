use ocl::{Buffer, Context, Kernel, Program, Queue};
use image::{ImageBuffer, Luma};
use crate::normalized_x_corr::{compute_integral_images, sum_region};
use crate::imgtools;


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

    let min_expected_corr = precision * *fast_expected_corr;

    let gpu_results = opencl_ncc(
        image,
        image_width,
        image_height,
        *template_width,
        *template_height,
        template_segments_fast,
    )?;

    let mut final_matches = Vec::new();

    for (x, y, fast_corr) in gpu_results {
        if fast_corr >= min_expected_corr {
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

            if refined_corr >= min_expected_corr as f64 {
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
    image: &[u8],
    image_width: u32,
    image_height: u32,
    template_width: u32,
    template_height: u32,
    segments: &[(u32, u32, u32, u32, f32)], // (x, y, width, height, value)
) -> ocl::Result<Vec<(u32, u32, f32)>> {
    use ocl::{Context, Queue, Program, Buffer, Kernel};

    let context = Context::builder().build()?;
    let queue = Queue::new(&context, context.devices()[0], None)?;

    let program_source = r#"
        __kernel void segmented_template_match(
            __global const uchar* image,
            __global const int4* segments, // (x, y, width, height)
            __global const float* segment_values,
            const int num_segments,
            __global float* results,
            const int image_width,
            const int image_height,
            const int template_width,
            const int template_height
        ) {
            int idx = get_global_id(0);
            int total_width = image_width - template_width + 1;
            int x = idx % total_width;
            int y = idx / total_width;

            if (x >= total_width || y >= (image_height - template_height + 1))
                return;

            float sum_image = 0.0f;
            float sum_image_sq = 0.0f;
            float sum_template = 0.0f;
            float sum_template_sq = 0.0f;
            float sum_image_template = 0.0f;

            for (int i = 0; i < num_segments; i++) {
                int4 seg = segments[i];
                float tpl_val = segment_values[i];

                for (int dy = 0; dy < seg.w; dy++) {
                    for (int dx = 0; dx < seg.z; dx++) {
                        int img_x = x + seg.x + dx;
                        int img_y = y + seg.y + dy;
                        float img_val = (float)(image[img_y * image_width + img_x]);

                        sum_image += img_val;
                        sum_template += tpl_val;
                        sum_image_sq += img_val * img_val;
                        sum_template_sq += tpl_val * tpl_val;
                        sum_image_template += img_val * tpl_val;
                    }
                }
            }

            float num_pixels = (float)(0);
            for (int i = 0; i < num_segments; i++) {
                int4 seg = segments[i];
                num_pixels += (float)(seg.z * seg.w);
            }

            float mean_image = sum_image / num_pixels;
            float mean_template = sum_template / num_pixels;
            float numerator = sum_image_template - num_pixels * mean_image * mean_template;
            float denom_img = sum_image_sq - num_pixels * mean_image * mean_image;
            float denom_tpl = sum_template_sq - num_pixels * mean_template * mean_template;
            float denominator = sqrt(denom_img * denom_tpl);
            float result = denominator != 0.0f ? numerator / denominator : -1.0f;
            results[idx] = result;
        }
    "#;

    let program = Program::builder().src(program_source).build(&context)?;

    let result_width = (image_width - template_width + 1) as usize;
    let result_height = (image_height - template_height + 1) as usize;
    let output_size = result_width * result_height;

    let buffer_image = Buffer::<u8>::builder()
        .queue(queue.clone())
        .len(image.len())
        .copy_host_slice(image)
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

    let kernel = Kernel::builder()
        .program(&program)
        .name("segmented_template_match")
        .queue(queue.clone())
        .global_work_size(output_size)
        .arg(&buffer_image)
        .arg(&buffer_segments)
        .arg(&buffer_segment_values)
        .arg(&(segment_int4.len() as i32))
        .arg(&buffer_results)
        .arg(&(image_width as i32))
        .arg(&(image_height as i32))
        .arg(&(template_width as i32))
        .arg(&(template_height as i32))
        .build()?;

    unsafe { kernel.enq()?; }

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