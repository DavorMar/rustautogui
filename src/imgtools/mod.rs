/*
Functions used throughout the code that have more of a general purpose, like
loading images from disk, converting image to black-white or RGB, cutting image 
and converting image to vector.
*/


use image::{io::Reader as ImageReader, ImageBuffer, Luma, Rgba, GrayImage, Pixel, Primitive};

/// Loads image from the provided path and converts to black-white format
/// Returns image in image::ImageBuffer format
pub fn load_image_bw(location:&str) -> Result<ImageBuffer<Luma<u8>, Vec<u8>>,String>  {
    let img  = match ImageReader::open(location){
        Ok(x) => x,
        Err(y) => return Err(y.to_string()),
    };
    
    let img = match img.decode() {
        Ok(x) => x,
        Err(y) => return Err(y.to_string()),
    };
    
    let gray_image: ImageBuffer<Luma<u8>, Vec<u8>> = img.to_luma8();
    Ok(gray_image)
}


/// Loads image from the provided path and converts to RGBA format
/// Returns image in image::ImageBuffer format
pub fn load_image_rgba(location:&str) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>,String>  {
    let img =  match ImageReader::open(location){
        Ok(x) => x,
        Err(y) => {
            return Err(y.to_string())
        } 
    };
    
    let img = match img.decode() {
        Ok(x) => x,
        Err(y) => return Err(y.to_string()),
    };
    
    let rgba_image: ImageBuffer<Rgba<u8>, Vec<u8>> = img.to_rgba8();
    Ok(rgba_image)
}

/// Does conversion from ImageBuffer RGBA to ImageBuffer Black and White(Luma)
pub fn convert_image_to_bw(image:ImageBuffer<Rgba<u8>,Vec<u8>>) -> Result<ImageBuffer<Luma<u8>,Vec<u8>>,&'static str> {
    let mut grayscale_data: Vec<u8> = Vec::with_capacity(image.len() as usize);
    let screen_width = image.width();
    let screen_height = image.height();
    for chunk in image.chunks_exact(4) {
        let r = chunk[2] as u32;
        let g = chunk[1] as u32;
        let b = chunk[0] as u32;
        // Calculate the grayscale value using the luminance formula
        let gray_value = ((r * 30 + g * 59 + b * 11) / 100) as u8;
        grayscale_data.push(gray_value);
    }
    let grayscale = GrayImage::from_raw(
        screen_width as u32,
        screen_height as u32,
        grayscale_data
        );
    match grayscale {
        Some(x ) => return Ok(x),
        None => return Err("failed to convert image to grayscale")
    }
}


/// Cuts Region of image. Inputs are top left x , y pixel coordinates on image,
///     width and height of region and the image being cut.
///     Returns image os same datatype
pub fn cut_screen_region<T>(x: u32, y: u32, width: u32, height: u32, screen_image: &ImageBuffer<T, Vec<u8>>) -> ImageBuffer<T, Vec<u8>>
where
    T: Pixel<Subpixel = u8> + 'static,
{
    assert!(x + width <= screen_image.width());
    assert!(y + height <= screen_image.height());

    let mut sub_image: ImageBuffer<T, Vec<u8>> = ImageBuffer::new(width, height);

    // copy pixels from the original image buffer to the sub-image buffer
    for y_sub in 0..height {
        for x_sub in 0..width {
            let pixel = screen_image.get_pixel(x + x_sub, y + y_sub);
            sub_image.put_pixel(x_sub, y_sub, *pixel);
        }
    }
    sub_image
}


///Converts Imagebuffer to Vector format
pub fn imagebuffer_to_vec<T: Copy + Primitive + 'static>(
    image: &ImageBuffer<Luma<T>, Vec<T>>,
) -> Vec<Vec<T>> {
    
    let (width, height) = image.dimensions();
    let zero_pixel = image.get_pixel(0, 0)[0];
    let mut vec: Vec<Vec<T>> = vec![vec![zero_pixel; width as usize]; height as usize];

    for y in 0..height {
        for x in 0..width {
            vec[y as usize][x as usize] = image.get_pixel(x, y)[0];
        }
    }
    vec
}













