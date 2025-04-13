/*
 * Template Matching Algorithm
 * Author: Davor Marušić, Siniša Popović, Zoran Kalafatić
 * License: GPLv3
 * (c) 2024 Davor Marušić, Siniša Popović, Zoran Kalafatić  All rights reserved.
 * Please read NOTICE.md file
 */

use crate::normalized_x_corr::{compute_integral_images, sum_region};

use crate::{imgtools, Region, Segment, SegmentedData};
use image::{ImageBuffer, Luma};
use rand::prelude::*;
use rayon::prelude::*;
use rustfft::num_traits::Pow;
use std::collections::HashSet;
use std::fs;
use std::ops::Deref;
use std::path::Path;
#[allow(unused_imports)]
use std::time::Instant;

pub fn fast_ncc_template_match(
    image: &ImageBuffer<Luma<u8>, Vec<u8>>,
    precision: f32,
    template_data: &SegmentedData,
    debug: &bool,
) -> Vec<(u32, u32, f64)> {
    /// Process:
    /// Template preparation : done before calling template match
    /// Template is
    let (image_width, image_height) = image.dimensions();

    // compute image integral, or in other words sum tables where each pixel
    // corresponds to sum of all the pixels above and left
    let image_vec: Vec<Vec<u8>> = imgtools::imagebuffer_to_vec(image);
    let (image_integral, squared_image_integral) = compute_integral_images(&image_vec);
    let SegmentedData {
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
    } = template_data;

    // calculate precision into expected correlation
    let adjusted_fast_expected_corr: f32 = precision * fast_expected_corr - 0.0001;
    let adjusted_slow_expected_corr: f32 = precision * slow_expected_corr - 0.0001;

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
    let mut found_points: Vec<(u32, u32, f64)> = coords
        .par_iter()
        .map(|&(x, y)| {
            let corr = fast_correlation_calculation(
                &image_integral,
                &squared_image_integral,
                template_segments_fast,
                template_segments_slow,
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
            (x, y, corr)
        })
        .filter(|&(_, _, corr)| corr >= adjusted_slow_expected_corr as f64)
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
    template_segments: &[Segment],
    template_width: u32,
    template_height: u32,
    file_name: &str,
) {
    let mut blurred_template: ImageBuffer<Luma<u8>, Vec<u8>> =
        ImageBuffer::new(template_width, template_height);
    let mut rng = rand::rng();
    let debug_path = Path::new("debug");
    // not returning error , just printing it because debug mode shouldnt cause crashes here
    if !debug_path.exists() && fs::create_dir_all(debug_path).is_err() {
        println!("Failed to create debug folder. Please create it manually in the root folder");
        return;
    }
    for segment in template_segments {
        let mut rng_mult: f32 = rng.random();
        if segment.mean < 127.5 {
            rng_mult += 1.0;
        }
        for y1 in 0..segment.height {
            for x1 in 0..segment.width {
                blurred_template.put_pixel(
                    segment.x + x1,
                    segment.y + y1,
                    Luma([(segment.mean * rng_mult) as u8]),
                );
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

    for segment in template_segments {
        for y1 in 0..segment.height {
            for x1 in 0..segment.width {
                blurred_template2.put_pixel(
                    segment.x + x1,
                    segment.y + y1,
                    Luma([segment.mean as u8]),
                );
            }
        }
    }
    let error_catch = blurred_template2.save(file_name);

    match error_catch {
        Ok(_) => (),
        Err(_) => println!("Failed to save image"),
    }
}

#[allow(clippy::too_many_arguments)]
fn fast_correlation_calculation(
    image_integral: &[Vec<u64>],
    squared_image_integral: &[Vec<u64>],
    template_segments_fast: &[Segment], // roughly segmented, low number of segments
    template_segments_slow: &[Segment], // precisely segmented, high number of segments
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

    for segment in template_segments_fast {
        let segment_image_sum = sum_region(
            image_integral,
            x + segment.x,
            y + segment.y,
            segment.width,
            segment.height,
        );
        let segment_nominator_value: f32 = (segment_image_sum as f32
            - mean_image * (segment.height * segment.width) as f32)
            * (segment.mean - segments_fast_mean);

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
    let image_sum_squared_deviations =
        sum_squared_image as f32 - (sum_image as f32).powi(2) / template_area as f32;
    let denominator = (image_sum_squared_deviations * fast_segments_sum_squared_deviations).sqrt();

    ///////////////

    let mut corr: f32 = nominator / denominator;

    if corr > 1.1 || corr.is_nan() {
        corr = -100.0;
        return corr as f64;
    }

    // second calculation with more detailed picture
    if corr >= min_expected_corr {
        nominator = 0.0;
        for segment in template_segments_slow {
            let segment_image_sum = sum_region(
                image_integral,
                x + segment.x,
                y + segment.y,
                segment.width,
                segment.height,
            );
            let segment_nominator_value: f32 = (segment_image_sum as f32
                - mean_image * (segment.height * segment.width) as f32)
                * (segment.mean - segments_slow_mean);

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
) -> SegmentedData {
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
            let squared_deviation = (template_value - mean_template_value).powf(2.0);
            template_sum_squared_deviations += squared_deviation;
        }
    }
    let avg_deviation_of_template =
        (template_sum_squared_deviations / (template_width * template_height) as f32).sqrt();

    // create fast segmented image
    let (
        picture_segments_fast,
        segment_sum_squared_deviations_fast,
        expected_corr_fast,
        segments_mean_fast,
    ) = create_picture_segments(
        template,
        mean_template_value,
        avg_deviation_of_template,
        "fast",
    );
    // create slow segmented image
    let (
        picture_segments_slow,
        segment_sum_squared_deviations_slow,
        expected_corr_slow,
        segments_mean_slow,
    ) = create_picture_segments(
        template,
        mean_template_value,
        avg_deviation_of_template,
        "slow",
    );

    // merge pictures segments
    let mut picture_segments_fast = merge_picture_segments(picture_segments_fast);
    picture_segments_fast.sort_by(|a, b| {
        // Compare the areas
        a.area().cmp(&b.area())
    });
    let mut picture_segments_slow = merge_picture_segments(picture_segments_slow);
    picture_segments_slow.sort_by(|a, b| {
        // Compare the areas
        a.area().cmp(&b.area())
    });

    if *debug {
        let fast_segment_number = picture_segments_fast.len();
        let slow_segment_number = picture_segments_slow.len();
        println!("reduced number of segments to {fast_segment_number} for fast image and {slow_segment_number} for slow image" );
    }
    #[allow(clippy::needless_if)]
    if (picture_segments_fast.len() == 1) | (picture_segments_slow.len() == 1) {}

    SegmentedData {
        template_segments_fast: picture_segments_fast,
        template_segments_slow: picture_segments_slow,
        template_width,
        template_height,
        fast_segments_sum_squared_deviations: segment_sum_squared_deviations_fast,
        slow_segments_sum_squared_deviations: segment_sum_squared_deviations_slow,
        fast_expected_corr: expected_corr_fast,
        slow_expected_corr: expected_corr_slow,
        segments_mean_fast,
        segments_mean_slow,
    }
}

#[allow(unused_assignments)]
#[allow(clippy::type_complexity)]
fn create_picture_segments(
    template: &ImageBuffer<Luma<u8>, Vec<u8>>,
    mean_template_value: f32,
    avg_deviation_of_template: f32,
    template_type: &str,
) -> (Vec<Segment>, f32, f32, f32) {
    /// returns (picture_segments,segment_sum_squared_deviations, expected_corr, segments_mean)
    /// calls recursive divide and conquer binary segmentation function which divides
    /// picture based on threshold of minimal standard deviation
    ///
    /// If too many segments are created, threshold is increased in loop untill condition is satisfied
    let (template_width, template_height) = template.dimensions();
    let mut picture_segments = Vec::<Segment>::new();

    // call the recursive function to divide the picture into segments of similar pixel values

    let mut target_corr = 0.0;
    let mut threshold = 0.0;
    if template_type == "fast" {
        threshold = 0.99;
        target_corr = -0.95;
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
            template,
            0,
            0,
            threshold * avg_deviation_of_template,
        );

        threshold -= 0.05;
        if threshold <= 0.1 {
            break;
        }
        // iterate through segments to calculate sum
        segments_sum = 0;
        let mut segment_count_pixels = 0;
        for segment in &picture_segments {
            segments_sum += segment.mean as u32 * segment.area();
            segment_count_pixels += segment.area();
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
        for segment in &picture_segments {
            for y_segment in 0..segment.height {
                for x_segment in 0..segment.width {
                    let template_pixel_value =
                        template.get_pixel(segment.x + x_segment, segment.y + y_segment)[0];

                    let template_diff = template_pixel_value as f32 - mean_template_value;
                    let segment_diff = segment.mean - segments_mean;
                    segment_sum_squared_deviations += (segment.mean - segments_mean).powf(2.0);
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

    (
        picture_segments,
        segment_sum_squared_deviations,
        expected_corr,
        segments_mean,
    )
}

fn divide_and_conquer(
    picture_segments: &mut Vec<Segment>,
    segment: &ImageBuffer<Luma<u8>, Vec<u8>>,
    x: u32,
    y: u32,
    threshhold: f32,
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
        let segment_informations = Segment::new(x, y, segment_width, segment_height, segment_mean);
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

    if average_deviation > threshhold {
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

            let x1 = x + segment_width / 2 + additional_pixel;
            // go recursively into first and second image halfs
            divide_and_conquer(picture_segments, &image_1, x, y, threshhold);
            divide_and_conquer(picture_segments, &image_2, x1, y, threshhold);

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
            divide_and_conquer(picture_segments, &image_1, x, y, threshhold);
            divide_and_conquer(picture_segments, &image_2, x, y1, threshhold);
        };

    // recursion exit
    } else {
        let segment_informations = Segment::new(x, y, segment_width, segment_height, segment_mean);
        picture_segments.push(segment_informations);
    }
}

fn merge_picture_segments(
    // x,y, width, height
    segmented_template: Vec<Segment>,
) -> Vec<Segment> {
    // Make the input mutable for modifications
    let mut segmented_template = segmented_template;
    let mut changes_made = true;

    segmented_template.sort_by(|a, b| {
        a.x.partial_cmp(&b.x)
            .unwrap()
            .then(a.y.partial_cmp(&b.y).unwrap())
    });
    while changes_made {
        changes_made = false;
        // looping through ordered by x. Doing only vertical merges, not checking on y.
        'outer_loop: for segment_i in 0..segmented_template.len() {
            // get current segment
            let (part1, part2) = segmented_template.split_at_mut(segment_i + 1);
            let current = &mut part1[segment_i];
            // skip already merged segment
            if current.zero_size() {
                continue 'outer_loop;
            }
            // loop through all the next segments till x differs, break and continue
            'inner_loop: for second in part2 {
                if current.x != second.x {
                    continue 'outer_loop;
                }
                if second.zero_size() {
                    continue 'inner_loop;
                }
                if current.y + current.height < second.y {
                    continue 'outer_loop;
                }
                if current.width == second.width
                    && current.mean == second.mean
                    && (current.y + current.height == second.y)
                {
                    current.height += second.height;
                    second.width = 0;
                    second.height = 0;
                    changes_made = true;
                }
            }
        }

        // now doing horizontal merges
        'outer_loop: for segment_i in 0..segmented_template.len() {
            // get current segment
            let (part1, part2) = segmented_template.split_at_mut(segment_i + 1);
            let current = &mut part1[segment_i];
            if current.zero_size() {
                continue 'outer_loop;
            }
            // loop through all the next segments till x differs, break and continue
            'inner_loop: for second in part2 {
                if current.x + current.width < second.x {
                    continue 'outer_loop;
                }
                // skip already merged segment
                if second.zero_size() {
                    continue 'inner_loop;
                }
                if current.y != second.y {
                    continue 'inner_loop;
                }
                if current.height == second.height
                    && current.mean == second.mean
                    && (current.x + current.width == second.x)
                {
                    current.width += second.width;
                    second.width = 0; // width_second
                    second.height = 0; // height_second
                    changes_made = true;
                }
            }
        }
    }
    // Retain only those segments where both width and height are not zero
    segmented_template.retain(|&s| !s.zero_size());
    segmented_template
}

#[allow(dead_code)]
fn merge_picture_segments_old_slow(segmented_template: Vec<Segment>) -> Vec<Segment> {
    // Make the input mutable for modifications
    let mut segmented_template = segmented_template;
    let mut changes_made = true;

    // Loop until no more segments can be merged
    while changes_made {
        // Temporary vector to hold new merged segments
        let mut new_segmented_template = Vec::<Segment>::new();
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
            let b = segmented_template[i];
            let mut segment_merged = false;

            // Try to merge with another segment
            #[allow(clippy::needless_range_loop)]
            for j in (i + 1)..segmented_template.len() {
                // Skip already merged segments
                if removed_indexes.contains(&j) {
                    continue;
                }

                // Get other segment details
                let a = segmented_template[j];

                // Check for vertical merge
                if b.x == a.x
                    && b.width == a.width
                    && b.mean == a.mean
                    && (b.y + b.height == a.y || a.y + a.height == b.y)
                {
                    // Merge segments vertically
                    segment_merged = true;
                    changes_made = true;
                    let new_segment =
                        Segment::new(b.x, b.y.min(a.y), b.width, b.height + a.height, b.mean);
                    new_segmented_template.push(new_segment);
                    removed_indexes.insert(i);
                    removed_indexes.insert(j);

                    break;
                }
                // Check for horizontal merge
                else if b.y == a.y
                    && b.height == a.height
                    && b.mean == a.mean
                    && (b.x + b.width == a.x || a.x + a.width == b.x)
                {
                    // Merge segments horizontally
                    segment_merged = true;
                    changes_made = true;
                    let new_segment =
                        Segment::new(b.x.min(a.x), b.y, b.width + a.width, b.height, b.mean);
                    new_segmented_template.push(new_segment);
                    removed_indexes.insert(i);
                    removed_indexes.insert(j);

                    break;
                }
            }

            // If not merged, add the original segment
            if !segment_merged && !removed_indexes.contains(&i) {
                new_segmented_template.push(Segment::new(b.x, b.y, b.width, b.height, b.mean));
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
