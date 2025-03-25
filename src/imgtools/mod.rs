/*
Functions used throughout the code that have more of a general purpose, like
loading images from disk, converting image to black-white or RGB, cutting image
and converting image to vector.
*/
use crate::errors::AutoGuiError;
use image::{
    error::LimitError, io::Reader as ImageReader, DynamicImage, GrayImage, ImageBuffer, Luma,
    Pixel, Primitive, Rgb, Rgba,
};

use rustfft::{num_complex::Complex, num_traits::ToPrimitive};
/// Loads image from the provided path and converts to black-white format
/// Returns image in image::ImageBuffer format
pub fn load_image_bw(location: &str) -> Result<ImageBuffer<Luma<u8>, Vec<u8>>, AutoGuiError> {
    let img = ImageReader::open(location)?;

    let img = img.decode()?;

    let gray_image: ImageBuffer<Luma<u8>, Vec<u8>> = img.to_luma8();
    Ok(gray_image)
}

/// Loads image from the provided path and converts to RGBA format
/// Returns image in image::ImageBuffer format
pub fn load_image_rgba(location: &str) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, AutoGuiError> {
    let img = ImageReader::open(location)?;
    let img = img.decode()?;
    Ok(img.to_rgba8()) // return rgba image
}

pub fn check_imagebuffer_color_scheme<P, T>(
    image: &ImageBuffer<P, Vec<T>>,
) -> Result<u32, AutoGuiError>
where
    P: Pixel<Subpixel = T> + 'static,
    T: Primitive + ToPrimitive + 'static,
{
    let buff_len = image.as_raw().len() as u32;
    let (img_w, img_h) = image.dimensions();
    if (&img_w * &img_h) == 0 {
        let err = "Error: The buffer provided is empty and has no size".to_string();
        return Err(AutoGuiError::ImgError(err));
    }
    Ok(buff_len / (img_w * img_h))
}

pub fn convert_t_imgbuffer_to_luma<P, T>(
    image: &ImageBuffer<P, Vec<T>>,
    color_scheme: &u32,
) -> Result<ImageBuffer<Luma<u8>, Vec<u8>>, AutoGuiError>
where
    P: Pixel<Subpixel = T> + 'static,
    T: Primitive + ToPrimitive + 'static,
{
    let (img_w, img_h) = image.dimensions();
    match color_scheme {
        1 => {
            // Black and white image (Luma)
            // convert from Vec<T> to Vec<u8>
            let raw_img: Result<Vec<u8>, AutoGuiError> = image
                .as_raw()
                .into_iter()
                .map(|x| {
                    x.to_u8().ok_or(AutoGuiError::ImgError(
                        "Pixel conversion to raw failed".to_string(),
                    ))
                })
                .collect();

            ImageBuffer::<Luma<u8>, Vec<u8>>::from_raw(img_w, img_h, raw_img?).ok_or(
                AutoGuiError::ImgError("failed to convert to Luma".to_string()),
            )
        }
        3 => {
            // Rgb
            let raw_img: Result<Vec<u8>, AutoGuiError> = image
                .as_raw()
                .into_iter()
                .map(|x| {
                    x.to_u8().ok_or(AutoGuiError::ImgError(
                        "Pixel conversion to raw failed".to_string(),
                    ))
                })
                .collect();
            let rgb_img = ImageBuffer::<Rgb<u8>, Vec<u8>>::from_raw(img_w, img_h, raw_img?).ok_or(
                AutoGuiError::ImgError("Failed conversion to RGB".to_string()),
            )?;
            Ok(DynamicImage::ImageRgb8(rgb_img).to_luma8())
        }
        4 => {
            // Rgba
            let raw_img: Result<Vec<u8>, AutoGuiError> = image
                .as_raw()
                .into_iter()
                .map(|x| {
                    x.to_u8().ok_or(AutoGuiError::ImgError(
                        "Pixel conversion to raw failed".to_string(),
                    ))
                })
                .collect();
            let rgba_img = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(img_w, img_h, raw_img?)
                .ok_or(AutoGuiError::ImgError(
                    "Failed conversion to RGBA".to_string(),
                ))?;
            Ok(DynamicImage::ImageRgba8(rgba_img).to_luma8())
        }
        _ => {
            return Err(AutoGuiError::ImgError(
                "Unknown image format. Load works only for Rgb/Rgba/Luma(BW) formats".to_string(),
            ))
        }
    }
}

/// Does conversion from ImageBuffer RGBA to ImageBuffer Black and White(Luma)
pub fn convert_rgba_to_bw(
    image: ImageBuffer<Rgba<u8>, Vec<u8>>,
) -> Result<ImageBuffer<Luma<u8>, Vec<u8>>, AutoGuiError> {
    let (img_w, img_h) = image.dimensions();
    let raw_img: Result<Vec<u8>, AutoGuiError> = image
        .as_raw()
        .into_iter()
        .map(|x| {
            x.to_u8().ok_or(AutoGuiError::ImgError(
                "Pixel conversion to raw failed".to_string(),
            ))
        })
        .collect();
    let rgba_img = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(img_w, img_h, raw_img?).ok_or(
        AutoGuiError::ImgError("Failed to convert to RGBA".to_string()),
    )?;
    Ok(DynamicImage::ImageRgba8(rgba_img).to_luma8())
}

/// Does conversion from ImageBuffer RGBA to ImageBuffer Black and White(Luma)
pub fn convert_rgba_to_bw_old(
    image: ImageBuffer<Rgba<u8>, Vec<u8>>,
) -> Result<ImageBuffer<Luma<u8>, Vec<u8>>, AutoGuiError> {
    let mut grayscale_data: Vec<u8> = Vec::with_capacity(image.len());
    let image_width = image.width();
    let image_height = image.height();
    for chunk in image.chunks_exact(4) {
        let r = chunk[2] as u32;
        let g = chunk[1] as u32;
        let b = chunk[0] as u32;
        // Calculate the grayscale value using the luminance formula
        let gray_value = ((r * 30 + g * 59 + b * 11) / 100) as u8;
        grayscale_data.push(gray_value);
    }
    GrayImage::from_raw(image_width as u32, image_height as u32, grayscale_data).ok_or(
        AutoGuiError::ImgError("Failed to convert to grayscale".to_string()),
    )
}

/// Cuts Region of image. Inputs are top left x , y pixel coordinates on image,
///     width and height of region and the image being cut.
///     Returns image os same datatype
pub fn cut_screen_region<T>(
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    screen_image: &ImageBuffer<T, Vec<u8>>,
) -> ImageBuffer<T, Vec<u8>>
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
