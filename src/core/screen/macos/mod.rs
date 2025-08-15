use core_graphics::display;
use core_graphics::display::CGDisplay;

use crate::{errors::AutoGuiError, imgtools, Region};

#[cfg(not(feature = "lite"))]
use image::{
    imageops::{resize, FilterType::Nearest},
    GrayImage, ImageBuffer, Luma, Rgba,
};

#[derive(Debug, Clone)]
pub struct Screen {
    pub screen_width: i32,
    pub screen_height: i32,
    #[cfg(not(feature = "lite"))]
    pub screen_data: ScreenImgData,
}
#[cfg(not(feature = "lite"))]
#[derive(Debug, Clone)]
pub struct ScreenImgData {
    pub display: CGDisplay,
    pub pixel_data: Vec<u8>,
    pub scaling_factor_x: f32, // difference between logical and phyisical resolution
    pub scaling_factor_y: f32,
    pub screen_region_width: u32,
    pub screen_region_height: u32,
}

impl Screen {
    pub fn new() -> Result<Self, AutoGuiError> {
        unsafe {
            let main_display_id = display::CGMainDisplayID();
            let main_display = CGDisplay::new(main_display_id);
            // because of retina display,  and scaling factors, image captured can be double the size
            // for that detection of retina is needed to divide all the pixel positions
            // by the factor. As far as i understood it should actually always be 2 but leaving it like this
            // shouldnt produce errors and covers any different case
            #[allow(unused_variables)]
            let image = main_display.image().ok_or(AutoGuiError::OSFailure(
                "Failed to create CGImage from display".to_string(),
            ))?;

            let screen_width = main_display.pixels_wide() as i32;
            let screen_height = main_display.pixels_high() as i32;

            #[cfg(not(feature = "lite"))]
            let screen_data = ScreenImgData {
                display: main_display,
                pixel_data: vec![0u8; (screen_width * screen_height * 4) as usize],
                scaling_factor_x: image.width() as f32 / screen_width as f32,
                scaling_factor_y: image.height() as f32 / screen_height as f32,
                screen_region_width: 0,
                screen_region_height: 0,
            };
            Ok(Self {
                screen_height,
                screen_width,

                #[cfg(not(feature = "lite"))]
                screen_data,
            })
        }
    }

    /// returns screen dimensions. All monitors included
    pub fn dimension(&self) -> (i32, i32) {
        let dimensions = (self.screen_width, self.screen_height);
        dimensions
    }
    #[cfg(not(feature = "lite"))]
    #[allow(dead_code)]
    /// return region dimension which is set up when template is precalculated
    pub fn region_dimension(&self) -> (u32, u32) {
        let dimensions = (
            self.screen_data.screen_region_width,
            self.screen_data.screen_region_height,
        );
        dimensions
    }
    #[cfg(not(feature = "lite"))]
    #[allow(dead_code)]
    /// executes convert_bitmap_to_rgba, meaning it converts Vector of values to RGBA and crops the image
    /// as inputted region area. Not used anywhere at the moment
    pub fn grab_screen_image(
        &mut self,
        region: Region,
    ) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, AutoGuiError> {
        let Region {
            x,
            y,
            width,
            height,
        } = region;
        self.screen_data.screen_region_width = width;
        self.screen_data.screen_region_height = height;
        self.capture_screen()?;
        let image = self.convert_bitmap_to_rgba()?;
        let cropped_image: ImageBuffer<Rgba<u8>, Vec<u8>> =
            imgtools::cut_screen_region(x, y, width, height, &image);
        Ok(cropped_image)
    }
    #[cfg(not(feature = "lite"))]
    /// executes convert_bitmap_to_grayscale, meaning it converts Vector of values to grayscale and crops the image
    /// as inputted region area
    pub fn grab_screen_image_grayscale(
        &mut self,
        region: Region,
    ) -> Result<ImageBuffer<Luma<u8>, Vec<u8>>, AutoGuiError> {
        let Region {
            x,
            y,
            width,
            height,
        } = region;
        self.screen_data.screen_region_width = width;
        self.screen_data.screen_region_height = height;
        self.capture_screen()?;
        let image: ImageBuffer<Luma<u8>, Vec<u8>> = self.convert_bitmap_to_grayscale()?;
        let cropped_image: ImageBuffer<Luma<u8>, Vec<u8>> =
            imgtools::cut_screen_region(x, y, width, height, &image);
        Ok(cropped_image)
    }
    #[cfg(not(feature = "lite"))]
    /// captures and saves screenshot of monitors
    pub fn grab_screenshot(&mut self, image_path: &str) -> Result<(), AutoGuiError> {
        self.capture_screen()?;
        let image = self.convert_bitmap_to_rgba()?;
        Ok(image.save(image_path)?)
    }
    #[cfg(not(feature = "lite"))]
    /// first order capture screen function. it captures screen image and stores it as vector in self.pixel_data
    fn capture_screen(&mut self) -> Result<(), AutoGuiError> {
        let image = self
            .screen_data
            .display
            .image()
            .ok_or(AutoGuiError::OSFailure(
                "Failed to capture screen image".to_string(),
            ))?;

        let pixel_data: Vec<u8> = image
            .data()
            .bytes()
            .chunks(4)
            .flat_map(|chunk| {
                // reorder color components
                if let &[b, g, r, a] = chunk {
                    vec![r, g, b, a]
                } else {
                    unreachable!()
                }
            })
            .collect();
        self.screen_data.pixel_data = pixel_data;
        Ok(())
    }
    #[cfg(not(feature = "lite"))]
    /// convert vector to Luma Imagebuffer
    fn convert_bitmap_to_grayscale(&self) -> Result<ImageBuffer<Luma<u8>, Vec<u8>>, AutoGuiError> {
        let mut grayscale_data =
            Vec::with_capacity((self.screen_width * self.screen_height) as usize);
        for chunk in self.screen_data.pixel_data.chunks_exact(4) {
            let r = chunk[2] as u32;
            let g = chunk[1] as u32;
            let b = chunk[0] as u32;
            // calculate the grayscale value using the luminance formula
            let gray_value = ((r * 30 + g * 59 + b * 11) / 100) as u8;
            grayscale_data.push(gray_value);
        }
        let image = GrayImage::from_raw(
            (self.screen_data.scaling_factor_x * self.screen_width as f32) as u32,
            (self.screen_data.scaling_factor_y * self.screen_height as f32) as u32,
            grayscale_data,
        )
        .ok_or(AutoGuiError::ImgError(
            "Could not convert image to grayscale".to_string(),
        ))?;
        let image = resize(
            &image,
            self.screen_width as u32,
            self.screen_height as u32,
            Nearest,
        );
        Ok(image)
    }
    #[cfg(not(feature = "lite"))]
    /// convert vector to RGBA ImageBuffer
    fn convert_bitmap_to_rgba(&self) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, AutoGuiError> {
        ImageBuffer::from_raw(
            (self.screen_data.scaling_factor_x * self.screen_width as f32) as u32,
            (self.screen_data.scaling_factor_y * self.screen_height as f32) as u32,
            self.screen_data.pixel_data.clone(),
        )
        .ok_or(AutoGuiError::ImgError(
            "Could not convert image to rgba".to_string(),
        ))
    }
}
