extern crate rayon;

pub mod fast_segment_x_corr;
pub mod fft_ncc;
pub mod slow_ncc;

fn compute_integral_image(image: &[Vec<u8>]) -> Vec<Vec<u64>> {
    /*
    Function that takes an image as input and computes an integral table (sum table).
    Table is calculated in a way : f(x,y) = sum where f(x1<=x, y1<=y), meaning it always
    sums all the pixels that are above and left of the value, including the value. Meaning if we take middle of
    the picture as point, the top left corner will be summed and be represented in that value.
    This is done a bit faster with an algorithm that says:
        s(x,y) = f(x,y) + s(x-1,y) + s(x, y-1) - s(x-1, y-1), meaning it takes pixel value and adds
        already calculated sum value left from it, above from it and subtracts the pixel value that is
        above and left from it.
    for x=0 or y=0 we just omit part of the formula, and for x=0 and y=0 we insert just the pixel value.
    This table is made in order to more easily compute sums of images for further calculations in the algorithm.
    example:
    [1 1 1 1]
    [1 1 1 1]
    [1 1 1 1]
    [1 1 1 1]
    becomes
    [ 1 2 3  4  ]
    [ 2 4 6  8  ]
    [ 3 6 9  12 ]
    [ 4 8 12 16 ]
    */

    let height = image.len() as u32;
    let width = if height > 0 { image[0].len() as u32 } else { 0 };

    let mut integral_image = vec![vec![0u64; width as usize]; height as usize];

    for y in 0..height {
        for x in 0..width {
            let pixel_value = image[y as usize][x as usize] as u64;
            let integral_value = if x == 0 && y == 0 {
                pixel_value
            } else if x == 0 {
                pixel_value + integral_image[(y - 1) as usize][x as usize]
            } else if y == 0 {
                pixel_value + integral_image[y as usize][(x - 1) as usize]
            } else {
                pixel_value
                    + integral_image[(y - 1) as usize][x as usize]
                    + integral_image[y as usize][(x - 1) as usize]
                    - integral_image[(y - 1) as usize][(x - 1) as usize]
            };
            integral_image[y as usize][x as usize] = integral_value;
        }
    }

    integral_image
}

fn compute_squared_integral_image(image: &[Vec<u8>]) -> Vec<Vec<u64>> {
    /*
    Same as compute_integral_image, except we always take squared value of pixel f(x,y).
    */

    let height = image.len() as u32;
    let width = if height > 0 { image[0].len() as u32 } else { 0 };

    let mut integral_image = vec![vec![0u64; width as usize]; height as usize];

    for y in 0..height {
        for x in 0..width {
            let pixel_value = (image[y as usize][x as usize] as u64).pow(2);
            let integral_value = if x == 0 && y == 0 {
                pixel_value
            } else if x == 0 {
                pixel_value + integral_image[(y - 1) as usize][x as usize]
            } else if y == 0 {
                pixel_value + integral_image[y as usize][(x - 1) as usize]
            } else {
                pixel_value
                    + integral_image[(y - 1) as usize][x as usize]
                    + integral_image[y as usize][(x - 1) as usize]
                    - integral_image[(y - 1) as usize][(x - 1) as usize]
            };
            integral_image[y as usize][x as usize] = integral_value;
        }
    }

    integral_image
}

/// Compute both normal and squared integral image
fn compute_integral_images(image: &[Vec<u8>]) -> (Vec<Vec<u64>>, Vec<Vec<u64>>) {
    let height = image.len() as u32;
    let width = if height > 0 { image[0].len() as u32 } else { 0 };
    let mut integral_image = vec![vec![0u64; width as usize]; height as usize];
    let mut squared_integral_image = vec![vec![0u64; width as usize]; height as usize];
    for y in 0..height {
        for x in 0..width {
            let pixel_value = image[y as usize][x as usize] as u64;
            let pixel_value_squared = (image[y as usize][x as usize] as u64).pow(2);
            let (integral_value, squared_integral_value) = if x == 0 && y == 0 {
                (pixel_value, pixel_value_squared)
            } else if x == 0 {
                (
                    pixel_value + integral_image[(y - 1) as usize][x as usize],
                    pixel_value_squared + squared_integral_image[(y - 1) as usize][x as usize],
                )
            } else if y == 0 {
                (
                    pixel_value + integral_image[y as usize][(x - 1) as usize],
                    pixel_value_squared + squared_integral_image[y as usize][(x - 1) as usize],
                )
            } else {
                (
                    pixel_value
                        + integral_image[(y - 1) as usize][x as usize]
                        + integral_image[y as usize][(x - 1) as usize]
                        - integral_image[(y - 1) as usize][(x - 1) as usize],
                    pixel_value_squared
                        + squared_integral_image[(y - 1) as usize][x as usize]
                        + squared_integral_image[y as usize][(x - 1) as usize]
                        - squared_integral_image[(y - 1) as usize][(x - 1) as usize],
                )
            };
            integral_image[y as usize][x as usize] = integral_value;
            squared_integral_image[y as usize][x as usize] = squared_integral_value;
        }
    }

    (integral_image, squared_integral_image)
}

fn sum_region(integral_image: &[Vec<u64>], x: u32, y: u32, width: u32, height: u32) -> u64 {
    /*
    Used to calculate sum region of an integral image. Bottom right pixel will have summed up value of everything above and left.
    In order to get exact sum value of subregion of the image, we take that sum from bottom right,
    subtract from it the value that is on the top right , in the first row above the image, subtract the value that is on
    the bottom left, in the first column left of the image, and add up the value that is top left in the
    first row and colum above and left of the image.
    [ 1 2   3  4 ]
    [ 2 4/  6  8/]
    [ 3 6   9 12 ]
    [ 4 8/ 12 16/]
     - ive marked most important numbers with /  that are important to calculate subimage that is like
     [9  12] (bottom right of the image)
     [12 16]
     In order to calculate subimage sum, we take 16 - 8 - 8 + 4, which is 4
     */
    let mut sum = integral_image[(y + height - 1) as usize][(x + width - 1) as usize];
    if x == 0 && y == 0 {
        // do nothing
    } else if y == 0 {
        sum -= integral_image[(y + height - 1) as usize][(x - 1) as usize];
    } else if x == 0 {
        sum -= integral_image[(y - 1) as usize][(x + width - 1) as usize]
    } else {
        sum += integral_image[(y - 1) as usize][(x - 1) as usize];
        sum -= integral_image[(y + height - 1) as usize][(x - 1) as usize];
        sum -= integral_image[(y - 1) as usize][(x + width - 1) as usize];
    }
    sum
}
