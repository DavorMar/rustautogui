/*
 * Template Matching Algorithm
 * Author: Davor Marušić, Siniša Popović, Zoran Kalafatić
 * License: GPLv3
 * (c) 2024 Davor Marušić, Siniša Popović, Zoran Kalafatić  All rights reserved.
 * Please read NOTICE.md file
 */

use crate::normalized_x_corr::{compute_integral_images, sum_region};

use crate::imgtools;
use image::{ImageBuffer, Luma};
use ocl::ffi::libc::printf;
use rand::prelude::*;
use rayon::prelude::*;
use rustfft::num_traits::Pow;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
#[allow(unused_imports)]
use std::time::Instant;

pub fn fast_ncc_template_match(
    image: &ImageBuffer<Luma<u8>, Vec<u8>>,
    precision: f32,
    template_data: &(
        Vec<(u32, u32, u32, u32, f32)>,
        Vec<(u32, u32, u32, u32, f32)>,
        u32,
        u32,
        f32,
        f32,
        f32,
        f32,
        f32,
        f32,
    ),
    debug: &bool,
) -> Vec<(u32, u32, f32)> {
    /// Process:
    /// Template preparation : done before calling template match
    /// Template is
    let (image_width, image_height) = image.dimensions();

    // compute image integral, or in other words sum tables where each pixel
    // corresponds to sum of all the pixels above and left
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

    // calculate precision into expected correlation
    let adjusted_fast_expected_corr: f32 = precision * fast_expected_corr - 0.0001 as f32;
    let adjusted_slow_expected_corr: f32 = precision * slow_expected_corr - 0.0001 as f32;

    if *debug {
        let fast_name = "debug/fast.png";
        save_template_segmented_images(
            template_segments_fast,
            *template_width,
            *template_height,
            fast_name,
        );
        let slow_name = "debug/slow.png";
        save_template_segmented_images(
            template_segments_slow,
            *template_width,
            *template_height,
            slow_name,
        );
    }

    let coords: Vec<(u32, u32)> = (0..=(image_height - template_height))
        .flat_map(|y| (0..=(image_width - template_width)).map(move |x| (x, y)))
        .collect();
    let mut found_points: Vec<(u32, u32, f32)> = coords
        .par_iter()
        .map(|&(x, y)| {
            let corr = fast_correlation_calculation(
                &image_integral,
                &squared_image_integral,
                &template_segments_fast,
                &template_segments_slow,
                *template_width,
                *template_height,
                *fast_segments_sum_squared_deviations,
                *slow_segments_sum_squared_deviations,
                *segments_mean_fast,
                *segments_mean_slow,
                x,
                y,
                adjusted_fast_expected_corr,
            );
            (x, y, corr as f32)
        })
        .filter(|&(_, _, corr)| corr as f32 >= adjusted_slow_expected_corr)
        .collect();

    // returned list of found points

    found_points.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
    if *debug {
        let found_point_len = found_points.len();
        if found_point_len > 0 {
            println!("first found position corr :({})", found_points[0].2);
        }
    }

    found_points
}

fn save_template_segmented_images(
    template_segments: &[(u32, u32, u32, u32, f32)],
    template_width: u32,
    template_height: u32,
    file_name: &str,
) {
    let mut blurred_template: ImageBuffer<Luma<u8>, Vec<u8>> =
        ImageBuffer::new(template_width, template_height);
    let mut rng = rand::rng();
    let debug_path = Path::new("debug");
    // not returning error , just printing it because debug mode shouldnt cause crashes here
    if !debug_path.exists() {
        if fs::create_dir_all(debug_path).is_err() {
            println!("Failed to create debug folder. Please create it manually in the root folder");
            return;
        }
    }
    for (x, y, segment_width, segment_height, segment_mean) in template_segments {
        let mut rng_mult: f32 = rng.random();
        if segment_mean < &127.5 {
            rng_mult += 1.0;
        }
        for y1 in 0..*segment_height {
            for x1 in 0..*segment_width {
                blurred_template.put_pixel(x + x1, y + y1, Luma([(segment_mean * rng_mult) as u8]));
            }
        }
    }

    let mut filename2 = String::new();
    if let Some(pos) = file_name.rfind('/') {
        // Create a new string with "random_" inserted after the last '/'
        let (left, right) = file_name.split_at(pos + 1); // split_at returns a tuple of two slices
        filename2 = filename2 + left + "random_" + right;
    } else {
        // If there's no '/', just prepend "random_"
        filename2 = filename2 + "random_" + file_name;
    }
    let error_catch = blurred_template.save(filename2);
    match error_catch {
        Ok(_) => (),
        Err(_) => println!("Failed to save image"),
    }

    let mut blurred_template2: ImageBuffer<Luma<u8>, Vec<u8>> =
        ImageBuffer::new(template_width, template_height);

    for (x, y, segment_width, segment_height, segment_mean) in template_segments {
        for y1 in 0..*segment_height {
            for x1 in 0..*segment_width {
                blurred_template2.put_pixel(x + x1, y + y1, Luma([*segment_mean as u8]));
            }
        }
    }
    let error_catch = blurred_template2.save(file_name);

    match error_catch {
        Ok(_) => (),
        Err(_) => println!("Failed to save image"),
    }
}

fn fast_correlation_calculation(
    image_integral: &[Vec<u64>],
    squared_image_integral: &[Vec<u64>],
    template_segments_fast: &[(u32, u32, u32, u32, f32)], // roughly segmented, low number of segments
    template_segments_slow: &[(u32, u32, u32, u32, f32)], // precisely segmented, high number of segments
    template_width: u32,
    template_height: u32,
    fast_segments_sum_squared_deviations: f32,
    slow_segments_sum_squared_deviations: f32,
    segments_fast_mean: f32,
    segments_slow_mean: f32,
    x: u32, // big image x value
    y: u32, // big image y value
    min_expected_corr: f32,
) -> f64 {
    let template_area = template_height * template_width;

    /////////// numerator calculation
    let sum_image: u64 = sum_region(image_integral, x, y, template_width, template_height);
    let mean_image = sum_image as f32 / (template_height * template_width) as f32;
    
    let mut nominator = 0.0;
    
    for (x1, y1, segment_width, segment_height, segment_value) in template_segments_fast {
        let segment_image_sum = sum_region(
            image_integral,
            x + x1,
            y + y1,
            *segment_width,
            *segment_height,
        );
        let segment_nominator_value: f32 = (segment_image_sum as f32
            - mean_image * (segment_height * segment_width) as f32)
            * (*segment_value - segments_fast_mean);

        // let segment_nominator_value: f64 =
        //     (segment_image_sum as f64 * (*segment_value as f64 - segments_mean as f64)) - (mean_image as f64 * (segment_height*segment_width) as f64 * (*segment_value as f64 - segments_mean as f64) );
        nominator += segment_nominator_value;
    }

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
    if x==206 && y==1 {
        println!("Sum template and sum squared, and mean_img: {}, {}, {}", sum_image, sum_squared_image, mean_image);
    }
    
    let image_sum_squared_deviations =
        sum_squared_image as f32 - (sum_image as f32).powi(2) / template_area as f32;
    let denominator = (image_sum_squared_deviations * fast_segments_sum_squared_deviations).sqrt();
    if x==206 && y==1 {
        println!("Var img :{}, sum_sq_dev: {}", image_sum_squared_deviations, fast_segments_sum_squared_deviations);
    }
    
    let mut corr: f32 = nominator / denominator;
    
    if x==206 && y==1 {
        println!("fast corr: {}, nominator: {}, denom: {}" , corr, nominator , denominator);
    }
    ///////////////

    if corr > 1.1 || corr.is_nan() {
        corr = -100.0;
        return corr as f64;
    }

    // second calculation with more detailed picture
    if corr >= min_expected_corr {
        nominator = 0.0;
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

            // let segment_nominator_value: f64 =
            //     (segment_image_sum as f64 * (*segment_value as f64 - segments_mean as f64)) - (mean_image as f64 * (segment_height*segment_width) as f64 * (*segment_value as f64 - segments_mean as f64) );
            nominator += segment_nominator_value;
        }

        // if nominator <= 0.0 {
        //     return -1.0;
        // }

        let denominator =
            (image_sum_squared_deviations * slow_segments_sum_squared_deviations).sqrt();
        corr = nominator / denominator;
    }
    if corr > 1.1 || corr.is_nan() {
        corr = -100.0;
        return corr as f64;
    }

    corr as f64
}

pub fn prepare_template_picture(
    template: &ImageBuffer<Luma<u8>, Vec<u8>>,
    debug: &bool,
    ocl: bool,
) -> (
    Vec<(u32, u32, u32, u32, f32)>,
    Vec<(u32, u32, u32, u32, f32)>,
    u32,
    u32,
    f32,
    f32,
    f32,
    f32,
    f32,
    f32,
) {
    ///
    ///preprocess all the picture subimages
    ///returns picture_segments_fast, -- segmented picture with least number of segments for low precision and high speed
    ///    picture_segments_slow, -- segmented picture with high number of segments for high precision and low speed
    ///    template_width,
    ///    template_height,
    ///    segment_sum_squared_deviations_fast, -- sum of squared deviations for denominator calculation
    ///    segment_sum_squared_deviations_slow,
    ///    expected_corr_fast, -- correlation between segmented template and original. Used to determine minimal expected correlation
    ///    expected_corr_slow,
    ///    segments_mean_fast, -- average of segmented template image
    ///    segments_mean_slow,
    ///
    ///
    /// Image is segmented based on its average standard deviation, using binary segmentation
    /// Each segment represents region with average standard deviation from its mean lower than certain threshold
    /// 2 segmented images are created, fast one with very high threshold, meaning higher deviation between pixels
    /// in same region, and slow one, with high precision, meaning low deviation and more segments
    /// All pixels in each segment are set to values of its mean.
    /// After that merging is performed, which connects neighbouring segments of same contact axis size and same value
    let (template_width, template_height) = template.dimensions();
    let mut sum_template = 0.0;

    if *debug {
        let pixel_number = template_height * template_width;
        println! {"starting with {pixel_number}"};
    }
    // calculate needed sums
    for y in 0..template_height {
        for x in 0..template_width {
            let template_value = template.get_pixel(x, y)[0] as f32;
            sum_template += template_value;
        }
    }
    let mean_template_value = sum_template / (template_height * template_width) as f32;

    let mut template_sum_squared_deviations: f32 = 0.0;
    for y in 0..template_height {
        for x in 0..template_width {
            let template_value = template.get_pixel(x, y)[0] as f32;
            let squared_deviation = (template_value - mean_template_value as f32).powf(2.0);
            template_sum_squared_deviations += squared_deviation;
        }
    }
    let avg_deviation_of_template =
        (template_sum_squared_deviations / (template_width * template_height) as f32).sqrt();

    // create fast segmented image
    let (
        mut picture_segments_fast,
        segment_sum_squared_deviations_fast,
        expected_corr_fast,
        segments_mean_fast,
    ) = create_picture_segments(
        &template,
        mean_template_value,
        avg_deviation_of_template,
        "fast",
        ocl,
    );
    // create slow segmented image
    let (
        mut picture_segments_slow,
        segment_sum_squared_deviations_slow,
        expected_corr_slow,
        segments_mean_slow,
    ) = create_picture_segments(
        &template,
        mean_template_value,
        avg_deviation_of_template,
        "slow",
        ocl,
    );

    if !ocl {
        // merge pictures segments
        picture_segments_fast = merge_picture_segments(picture_segments_fast);
        picture_segments_fast.sort_by(|a, b| {
            let area_a = a.2 * a.3; // width * height for segment a
            let area_b = b.2 * b.3; // width * height for segment b
            area_a.cmp(&area_b) // Compare the areas
        });
        picture_segments_slow = merge_picture_segments(picture_segments_slow);
        picture_segments_slow.sort_by(|a, b| {
            let area_a = a.2 * a.3; // width * height for segment a
            let area_b = b.2 * b.3; // width * height for segment b
            area_a.cmp(&area_b) // Compare the areas
        });
    }

    if *debug {
        let fast_segment_number = picture_segments_fast.len();
        let slow_segment_number = picture_segments_slow.len();
        println!("reduced number of segments to {fast_segment_number} for fast image and {slow_segment_number} for slow image" );
    }
    println!("Fast and slow segments : {}, {}", picture_segments_fast.len(), picture_segments_slow.len());
    let return_value: (
        Vec<(u32, u32, u32, u32, f32)>,
        Vec<(u32, u32, u32, u32, f32)>,
        u32,
        u32,
        f32,
        f32,
        f32,
        f32,
        f32,
        f32,
    ) = (
        picture_segments_fast,
        picture_segments_slow,
        template_width,
        template_height,
        segment_sum_squared_deviations_fast,
        segment_sum_squared_deviations_slow,
        expected_corr_fast,
        expected_corr_slow,
        segments_mean_fast,
        segments_mean_slow,
    );
    
    return_value
}

#[allow(unused_assignments)]
fn create_picture_segments(
    template: &ImageBuffer<Luma<u8>, Vec<u8>>,
    mean_template_value: f32,
    avg_deviation_of_template: f32,
    template_type: &str,
    ocl: bool,
) -> (Vec<(u32, u32, u32, u32, f32)>, f32, f32, f32) {
    /// returns (picture_segments,segment_sum_squared_deviations, expected_corr, segments_mean)
    /// calls recursive divide and conquer binary segmentation function which divides
    /// picture based on threshold of minimal standard deviation
    ///
    /// If too many segments are created, threshold is increased in loop untill condition is satisfied
    let (template_width, template_height) = template.dimensions();
    let mut picture_segments: Vec<(u32, u32, u32, u32, f32)> = Vec::new();

    // call the recursive function to divide the picture into segments of similar pixel values

    let mut target_corr = 0.0;
    let mut threshold = 0.0;

    if template_type == "fast" {
        if ocl {
            threshold = 0.99;
            target_corr = 0.99;
        } else {
            threshold = 0.99;
            target_corr = 0.99;
        }
    } else if template_type == "slow" {
        threshold = 0.85;
        target_corr = 0.99;
    }

    let mut expected_corr = -1.0;
    let mut segments_sum = 0;
    let mut segment_sum_squared_deviations = 0.0;
    let mut segments_mean = 0.0;
    while expected_corr < target_corr {
        divide_and_conquer(
            &mut picture_segments,
            &template,
            0,
            0,
            threshold * avg_deviation_of_template,
            ocl,
        );

        threshold -= 0.05;
        if threshold <= 0.1 {
            break;
        }
        // iterate through segments to calculate sum
        segments_sum = 0;
        let mut segment_count_pixels = 0;
        for (_, _, segment_width, segment_height, segment_value) in &picture_segments {
            segments_sum += *segment_value as u32 * (segment_width * segment_height);
            segment_count_pixels += segment_width * segment_height;
        }
        assert!(segment_count_pixels == (template_height * template_width));
        let mut numerator = 0.0;
        let mut denom1 = 0.0;
        let mut denom2 = 0.0;
        segment_sum_squared_deviations = 0.0;
        segments_mean = 0.0;

        segments_mean = segments_sum as f32 / (template_height * template_width) as f32;

        let mut count = 0;
        // calculate correlation between segmented picture and real template picture
        // use this correlation to know which correlation to  expect when searching on big image
        for (x, y, segment_width, segment_height, segment_value) in &picture_segments {
            for y_segment in 0..*segment_height {
                for x_segment in 0..*segment_width {
                    let template_pixel_value = template.get_pixel(x + x_segment, y + y_segment)[0];

                    let template_diff = template_pixel_value as f32 - mean_template_value;
                    let segment_diff = *segment_value as f32 - segments_mean;
                    segment_sum_squared_deviations +=
                        (segment_value - segments_mean as f32).powf(2.0);
                    numerator += template_diff * segment_diff;
                    denom1 += template_diff.powf(2.0);
                    denom2 += segment_diff.powf(2.0);

                    count += 1;
                }
            }
        }
        assert!(count == template_height * template_width);
        let denominator = (denom1 * denom2).sqrt();
        expected_corr = numerator / denominator;
        if expected_corr < target_corr {
            picture_segments = Vec::new();
        }
    }
    return (
        picture_segments,
        segment_sum_squared_deviations,
        expected_corr,
        segments_mean,
    );
}

fn divide_and_conquer(
    picture_segments: &mut Vec<(u32, u32, u32, u32, f32)>,
    segment: &ImageBuffer<Luma<u8>, Vec<u8>>,
    x: u32,
    y: u32,
    threshhold: f32,
    ocl: bool,
) {
    /*
    function that segments template image into areas that have similar color
    calculated with average standard deviation formula which goes against a threshold
    X and Y are segment locations on whole template image. Basically a binary segmentation
     */

    let (segment_width, segment_height) = segment.dimensions();
    let mut sum_squared_deviations: i64 = 0;
    let mut pixels_sum: u32 = 0;

    for y in 0..segment_height {
        for x in 0..segment_width {
            let pixel_value = segment.get_pixel(x, y)[0];
            pixels_sum += pixel_value as u32;
        }
    }
    let segment_mean = pixels_sum as f32 / (segment_height * segment_width) as f32;

    if segment_height == 1 && segment_width == 1 {
        let segment_informations: (u32, u32, u32, u32, f32) =
            (x, y, segment_width, segment_height, segment_mean);
        picture_segments.push(segment_informations);
        return;
    }

    for y in 0..segment_height {
        for x in 0..segment_width {
            let pixel_value = segment.get_pixel(x, y)[0];
            let squared_deviation = (pixel_value as i32 - segment_mean as i32).pow(2);
            sum_squared_deviations += squared_deviation as i64;
        }
    }
    let average_deviation =
        (sum_squared_deviations as f32 / (segment_width * segment_height) as f32).sqrt();
    let mut additional_pixel = 0;

    if (average_deviation > threshhold) 
    // || (ocl && (segment_width > 50 || segment_height > 50)) 
    {
        //split image
        // let (image_1, image_2) =
        if segment_width >= segment_height || segment_height == 1 {
            // if image wider than taller
            if segment_width % 2 == 1 {
                additional_pixel += 1;
            }

            let image_1 = imgtools::cut_screen_region(
                0,
                0,
                segment_width / 2 + additional_pixel,
                segment_height,
                segment,
            );
            let image_2 = imgtools::cut_screen_region(
                segment_width / 2 + additional_pixel,
                0,
                segment_width / 2,
                segment_height,
                segment,
            );

            let x1 = &x + segment_width / 2 + additional_pixel;
            // go recursively into first and second image halfs
            divide_and_conquer(picture_segments, &image_1, x, y, threshhold, ocl);
            divide_and_conquer(picture_segments, &image_2, x1, y, threshhold, ocl);

            //if image taller than wider
        } else {
            //for uneven pixel size segments need to add pixel to second picture positions
            if segment_height % 2 == 1 {
                additional_pixel += 1;
            }

            let image_1 = imgtools::cut_screen_region(
                0,
                0,
                segment_width,
                segment_height / 2 + additional_pixel,
                segment,
            );
            let image_2 = imgtools::cut_screen_region(
                0,
                segment_height / 2 + additional_pixel,
                segment_width,
                segment_height / 2,
                segment,
            );
            let y1 = y + segment_height / 2 + additional_pixel;
            // go recursively into first and second image halfs
            divide_and_conquer(picture_segments, &image_1, x, y, threshhold, ocl);
            divide_and_conquer(picture_segments, &image_2, x, y1, threshhold, ocl);
        };

    // recursion exit
    } else {
        let segment_informations: (u32, u32, u32, u32, f32) =
            (x, y, segment_width, segment_height, segment_mean as f32);
        picture_segments.push(segment_informations);
        return;
    }
}

fn merge_picture_segments(
    // x,y, width, height
    segmented_template: Vec<(u32, u32, u32, u32, f32)>,
) -> Vec<(u32, u32, u32, u32, f32)> {
    // Make the input mutable for modifications
    let mut segmented_template = segmented_template;
    let mut changes_made = true;

    segmented_template.sort_by(|a, b| {
        a.0.partial_cmp(&b.0)
            .unwrap()
            .then(a.1.partial_cmp(&b.1).unwrap())
    });
    while changes_made {
        changes_made = false;
        // looping through ordered by x. Doing only vertical merges, not checking on y.
        'outer_loop: for segment_i in 0..segmented_template.len() {
            // get current segment
            let (x_current, y_current, width_current, mut height_current, value_current) =
                segmented_template[segment_i];
            // skip already merged segment
            if width_current == 0 || height_current == 0 {
                continue 'outer_loop;
            }
            // loop through all the next segments till x differs, break and continue
            'inner_loop: for second_segment_i in (segment_i + 1)..segmented_template.len() {
                let (x_second, y_second, width_second, height_second, value_second) =
                    segmented_template[second_segment_i];
                if x_current != x_second {
                    continue 'outer_loop;
                }
                if width_second == 0 || height_second == 0 {
                    continue 'inner_loop;
                }
                if y_current + height_current < y_second {
                    continue 'outer_loop;
                }
                if width_current == width_second
                    && value_current == value_second
                    && (y_current + height_current == y_second)
                {
                    height_current = height_current + height_second;
                    segmented_template[segment_i].3 = height_current;
                    segmented_template[second_segment_i].2 = 0; // width_second
                    segmented_template[second_segment_i].3 = 0; // height_second
                    changes_made = true;
                }
            }
        }

        // now doing horizontal merges
        'outer_loop: for segment_i in 0..segmented_template.len() {
            // get current segment
            let (x_current, y_current, mut width_current, height_current, value_current) =
                segmented_template[segment_i];
            if width_current == 0 || height_current == 0 {
                continue 'outer_loop;
            }
            // loop through all the next segments till x differs, break and continue
            'inner_loop: for second_segment_i in (segment_i + 1)..segmented_template.len() {
                let (x_second, y_second, width_second, height_second, value_second) =
                    segmented_template[second_segment_i];
                if x_current + width_current < x_second {
                    continue 'outer_loop;
                }
                // skip already merged segment
                if width_second == 0 || height_second == 0 {
                    continue 'inner_loop;
                }
                if y_current != y_second {
                    continue 'inner_loop;
                }
                if height_current == height_second
                    && value_current == value_second
                    && (x_current + width_current == x_second)
                {
                    width_current = width_current + width_second;
                    segmented_template[segment_i].2 = width_current;
                    segmented_template[second_segment_i].2 = 0; // width_second
                    segmented_template[second_segment_i].3 = 0; // height_second
                    changes_made = true;
                }
            }
        }
    }
    // Retain only those segments where both width and height are not zero
    segmented_template.retain(|&(_, _, width, height, _)| width != 0 && height != 0);
    segmented_template
}

#[allow(dead_code)]
fn merge_picture_segments_old_slow(
    segmented_template: Vec<(u32, u32, u32, u32, f32)>,
) -> Vec<(u32, u32, u32, u32, f32)> {
    // Make the input mutable for modifications
    let mut segmented_template = segmented_template;
    let mut changes_made = true;

    // Loop until no more segments can be merged
    while changes_made {
        // Temporary vector to hold new merged segments
        let mut new_segmented_template: Vec<(u32, u32, u32, u32, f32)> = Vec::new();
        // Set to keep track of merged segment indices
        let mut removed_indexes: HashSet<usize> = HashSet::new();
        // Reset changes made flag
        changes_made = false;

        // Iterate over each segment
        for i in 0..segmented_template.len() {
            // Skip already merged segments
            if removed_indexes.contains(&i) {
                continue;
            }

            // Get current segment details
            let (x_b, y_b, width_b, height_b, value_b) = segmented_template[i];
            let mut segment_merged = false;

            // Try to merge with another segment
            for j in (i + 1)..segmented_template.len() {
                // Skip already merged segments
                if removed_indexes.contains(&j) {
                    continue;
                }

                // Get other segment details
                let (x_a, y_a, width_a, height_a, value_a) = segmented_template[j];

                // Check for vertical merge
                if x_b == x_a
                    && width_b == width_a
                    && value_b == value_a
                    && (y_b + height_b == y_a || y_a + height_a == y_b)
                {
                    // Merge segments vertically
                    segment_merged = true;
                    changes_made = true;
                    let new_segment = (x_b, y_b.min(y_a), width_b, height_b + height_a, value_b);
                    new_segmented_template.push(new_segment);
                    removed_indexes.insert(i);
                    removed_indexes.insert(j);

                    break;
                }
                // Check for horizontal merge
                else if y_b == y_a
                    && height_b == height_a
                    && value_b == value_a
                    && (x_b + width_b == x_a || x_a + width_a == x_b)
                {
                    // Merge segments horizontally
                    segment_merged = true;
                    changes_made = true;
                    let new_segment = (x_b.min(x_a), y_b, width_b + width_a, height_b, value_b);
                    new_segmented_template.push(new_segment);
                    removed_indexes.insert(i);
                    removed_indexes.insert(j);

                    break;
                }
            }

            // If not merged, add the original segment
            if !segment_merged && !removed_indexes.contains(&i) {
                new_segmented_template.push((x_b, y_b, width_b, height_b, value_b));
            }
        }

        // Sort and remove duplicates from the new list
        new_segmented_template.sort_by(|a, b| a.partial_cmp(b).unwrap());
        new_segmented_template.dedup();

        // Update the segmented template with the new list
        segmented_template = new_segmented_template;
    }

    // Return the merged segments
    segmented_template
}
