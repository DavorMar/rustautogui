/*
 * Fast Normalized Cross correlation algorithm
 * Author of the algorithm: J.P.Lewis
 * http://scribblethink.org/Work/nvisionInterface/vi95_lewis.pdf
 */

use crate::imgtools;
use core::cmp::max;
use image::{ImageBuffer, Luma};
use rayon::prelude::*;
use rustfft::{num_complex::Complex, Fft, FftPlanner};

use super::{compute_integral_images, sum_region};

pub fn fft_ncc(
    image: &ImageBuffer<Luma<u8>, Vec<u8>>,
    precision: f32,
    prepared_data: &(Vec<Complex<f32>>, f32, u32, u32, u32),
) -> Vec<(u32, u32, f64)> {
    // retreive all precalculated template data, most importantly template with already fft and conjugation calculated
    // sum squared deviations will be needed for denominator
    let (
        template_conj_freq,
        template_sum_squared_deviations,
        template_width,
        template_height,
        padded_size,
    ) = prepared_data;

    let mut planner = FftPlanner::<f32>::new();
    let fft: std::sync::Arc<dyn Fft<f32>> =
        planner.plan_fft_forward((padded_size * padded_size) as usize);
    let (image_width, image_height) = image.dimensions();
    let image_vec: Vec<Vec<u8>> = imgtools::imagebuffer_to_vec(&image);

    // compute needed integral images for denominator calculation
    let (image_integral, squared_image_integral) = compute_integral_images(&image_vec);

    //// calculating zero mean image
    let sum_image: u64 = sum_region(&image_integral, 0, 0, image_width, image_height);

    // calculating zero mean image , meaning image pixel values - image zero value
    let image_average_total = sum_image as f32 / (image_height * image_width) as f32; //@audit check image_height*image_width != 0
    let mut zero_mean_image: Vec<Vec<f32>> =
        vec![vec![0.0; image_width as usize]; image_height as usize];
    for y in 0..image_height {
        for x in 0..image_width {
            let image_pixel_value = image.get_pixel(x, y)[0] as f32;
            zero_mean_image[y as usize][x as usize] = image_pixel_value - image_average_total;
        }
    }

    // padding to least squares and placing image in top left corner, same as template
    let mut image_padded: Vec<Complex<f32>> =
        vec![Complex::new(0.0, 0.0); (padded_size * padded_size) as usize];
    for dy in 0..image_height {
        for dx in 0..image_width {
            let image_pixel_value = zero_mean_image[dy as usize][dx as usize];
            image_padded[dy as usize * *padded_size as usize + dx as usize] =
                Complex::new(image_pixel_value, 0.0);
        }
    }

    // conver image into frequency domain
    let ifft: std::sync::Arc<dyn Fft<f32>> =
        planner.plan_fft_inverse((padded_size * padded_size) as usize);
    fft.process(&mut image_padded);

    // calculate F(image) * F(template).conjugate
    let product_freq: Vec<Complex<f32>> = image_padded
        .iter()
        .zip(template_conj_freq.iter())
        .map(|(&img_val, &tmpl_val)| img_val * tmpl_val)
        .collect();
    // do inverse fft
    let mut fft_result: Vec<Complex<f32>> = product_freq.clone();
    ifft.process(&mut fft_result);

    // flatten for multithreading
    let coords: Vec<(u32, u32)> = (0..=(image_height - template_height)) //@audit could underflow if image_height = 0
        .flat_map(|y| (0..=(image_width - template_width)).map(move |x| (x, y))) //@audit could underflow if image_width = 0
        .collect();
    // multithreading pixel by pixel template sliding, where correlations are filtered by precision
    // sending all needed data to calculate nominator and denominator at each of pixel positions
    let mut found_points: Vec<(u32, u32, f64)> = coords
        .par_iter()
        .map(|&(x, y)| {
            let corr = fft_correlation_calculation(
                &image_integral,
                &squared_image_integral,
                *template_width,
                *template_height,
                *template_sum_squared_deviations,
                x,
                y,
                *padded_size,
                &fft_result,
            );

            (x, y, corr)
        })
        .filter(|&(_, _, corr)| corr > precision as f64)
        .collect();
    found_points.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());

    found_points
}

#[allow(dead_code)]
fn fft_correlation_calculation(
    image_integral: &[Vec<u64>],
    squared_image_integral: &[Vec<u64>],
    template_width: u32,
    template_height: u32,
    template_sum_squared_deviations: f32,
    x: u32, // big image x value
    y: u32, // big image y value,
    padded_size: u32,
    fft_result: &[Complex<f32>],
) -> f64 {
    /// Function for calculation of correlation at each pixel position
    ////////// denominator calculation
    let sum_image: u64 = sum_region(image_integral, x, y, template_width, template_height);

    let sum_squared_image: u64 = sum_region(
        squared_image_integral,
        x,
        y,
        template_width,
        template_height,
    );
    let image_sum_squared_deviations = sum_squared_image as f64
        - (sum_image as f64).powi(2) / (template_height * template_width) as f64; //@audit check template_height*template_width!=0
    let denominator =
        (image_sum_squared_deviations * template_sum_squared_deviations as f64).sqrt();

    /////////////// NOMINATOR CALCULATION

    // fft result is calculated invert of whole image and template that were padded and zero valued
    // each pixel position shows value for that template position
    let numerator_value =
        fft_result[(y * padded_size) as usize + x as usize].re / (padded_size * padded_size) as f32; //@audit guess the padded_size is always non zero but could be checked
    let mut corr = numerator_value as f64 / denominator;

    if corr > 2.0 {
        corr = -100.0;
    }
    corr
}

pub fn prepare_template_picture(
    template: &ImageBuffer<Luma<u8>, Vec<u8>>,
    image_width: u32,
    image_height: u32,
) -> (Vec<Complex<f32>>, f32, u32, u32, u32) {
    /// precalculate all the neccessary data so its not slowing down main process
    /// returning template in frequency domain, with calculated conjugate
    let (template_width, template_height) = template.dimensions();
    let padded_width = image_width.next_power_of_two();
    let padded_height = image_height.next_power_of_two();
    let padded_size = max(padded_width, padded_height);

    let mut sum_template = 0.0;
    // calculate needed sums
    for y in 0..template_height {
        for x in 0..template_width {
            let template_value = template.get_pixel(x, y)[0] as f32;
            sum_template += template_value;
        }
    }
    let mean_template_value = sum_template / (template_height * template_width) as f32; //@audit check template_height*template_width!=0
                                                                                        // create zero mean template
    let mut zero_mean_template: Vec<Vec<f32>> =
        vec![vec![0.0; template_width as usize]; template_height as usize];
    let mut template_sum_squared_deviations: f32 = 0.0;
    for y in 0..template_height {
        for x in 0..template_width {
            let template_value = template.get_pixel(x, y)[0] as f32;
            let squared_deviation = (template_value - mean_template_value as f32).powf(2.0);
            template_sum_squared_deviations += squared_deviation;

            // set zero mean value on new template
            zero_mean_template[y as usize][x as usize] = template_value - mean_template_value;
        }
    }
    // pad the zero mean template
    let mut template_padded: Vec<Complex<f32>> =
        vec![Complex::new(0.0, 0.0); (padded_size * padded_size) as usize];
    for dy in 0..template_height {
        for dx in 0..template_width {
            let template_pixel_value = zero_mean_template[dy as usize][dx as usize];
            template_padded[dy as usize * padded_size as usize + dx as usize] =
                Complex::new(template_pixel_value, 0.0);
        }
    }
    // convert template to frequency domain
    let mut planner = FftPlanner::<f32>::new();
    let fft: std::sync::Arc<dyn Fft<f32>> =
        planner.plan_fft_forward((padded_size * padded_size) as usize);
    fft.process(&mut template_padded);
    // calculate template conjugate
    let template_conj_freq: Vec<Complex<f32>> =
        template_padded.iter().map(|&val| val.conj()).collect();
    (
        template_conj_freq,
        template_sum_squared_deviations,
        template_width,
        template_height,
        padded_size,
    )
}
