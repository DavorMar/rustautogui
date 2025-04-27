use crate::core::template_match::{
    compute_integral_image, compute_squared_integral_image, sum_region,
};

use crate::imgtools;
use rayon::prelude::*;

use image::{ImageBuffer, Luma};

#[allow(dead_code)]
pub fn slow_ncc_template_match(
    image: &ImageBuffer<Luma<u8>, Vec<u8>>,
    template: &ImageBuffer<Luma<u8>, Vec<u8>>,
) -> (u32, u32, f64) {
    let (image_width, image_height) = image.dimensions();
    let (template_width, template_height) = template.dimensions();
    let mut best_match_x = 0;
    let mut best_match_y = 0;
    let mut min_corr = -100.0;

    // Compute integral images
    let image_vec: Vec<Vec<u8>> = imgtools::imagebuffer_to_vec(image);

    let image_integral = compute_integral_image(&image_vec);
    let squared_image_integral = compute_squared_integral_image(&image_vec);

    let mut sum_template = 0;
    let mut sum_squared_template = 0;

    for y in 0..template_height {
        for x in 0..template_width {
            let template_value = template.get_pixel(x, y)[0] as f64;
            sum_template += template_value as u64;
            sum_squared_template += (template_value as u64).pow(2);
        }
    }

    // let mean_template_value =
    //     sum_template as f64 / (template_height as f64 * template_width as f64);

    // let mut template_sum_squared_deviations = 0.0;
    // for y in 0..template_height {
    //     for x in 0..template_width {
    //         let template_value = template.get_pixel(x, y)[0] as f64;
    //         let squared_deviation = (template_value - mean_template_value).powf(2.0);
    //         template_sum_squared_deviations += squared_deviation;
    //     }
    // }

    let template_sum_squared_deviations = sum_squared_template as f64
        - (sum_template as f64).powi(2) / (template_height as f64 * template_width as f64); //@audit unlikely but check if template_height*template_width != 0, also powi() result varies on platform, but think it is CPU so it will be deterministic after all if not used on Raspberry Pi for example

    // Slide the template over the image
    let coords: Vec<(u32, u32)> = (0..=(image_height - template_height)) //@audit if image_height is 0 this will underflow
        .flat_map(|y| (0..=(image_width - template_width)).map(move |x| (x, y))) //@audit if image_width is 0 this will underflow
        .collect();

    let results: Vec<(u32, u32, f64)> = coords
        .par_iter()
        .map(|&(x, y)| {
            let corr = calculate_corr_value(
                image,
                &image_integral,
                &squared_image_integral,
                template,
                template_width,
                template_height,
                sum_template,
                template_sum_squared_deviations,
                x,
                y,
            );
            (x, y, corr)
        })
        .collect();
    for (x, y, corr) in results {
        if corr > min_corr {
            min_corr = corr;
            best_match_x = x;
            best_match_y = y;
        }
    }
    (best_match_x, best_match_y, min_corr)
}

fn calculate_corr_value(
    image: &ImageBuffer<Luma<u8>, Vec<u8>>,
    image_integral: &[Vec<u64>],
    squared_image_integral: &[Vec<u64>],
    template: &ImageBuffer<Luma<u8>, Vec<u8>>,
    template_width: u32,
    template_height: u32,
    sum_template: u64,
    template_sum_squared_deviations: f64,
    x: u32, // big image x value
    y: u32, // big image y value
) -> f64 {
    let sum_image: u64 = sum_region(image_integral, x, y, template_width, template_height);
    let sum_squared_image: u64 = sum_region(
        squared_image_integral,
        x,
        y,
        template_width,
        template_height,
    );
    let mean_image = sum_image as f64 / (template_height * template_width) as f64;
    let mean_template = sum_template as f64 / (template_height * template_width) as f64;
    let mut numerator = 0.0;
    for y1 in 0..template_height {
        for x1 in 0..template_width {
            let image_pixel = image.get_pixel(x + x1, y + y1)[0];
            let template_pixel = template.get_pixel(x1, y1)[0];

            numerator +=
                (image_pixel as f64 - mean_image) * (template_pixel as f64 - mean_template);
        }
    }

    let image_sum_squared_deviations = sum_squared_image as f64
        - (sum_image as f64).powi(2) / (template_height * template_width) as f64; //@audit same as above
    let denominator = (image_sum_squared_deviations * template_sum_squared_deviations).sqrt();

    let corr = numerator / denominator;

    corr.abs()
}
