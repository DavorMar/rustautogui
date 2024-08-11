use core_graphics::display::CGDisplay;
use core_graphics::display;

use image::{ imageops::{resize, FilterType::Nearest}, GrayImage, ImageBuffer, Luma, Rgba};
use crate::imgtools;


#[derive(Debug, Clone)]
pub struct Screen {
    pub screen_width: i32,
    pub screen_height: i32, 
    pub screen_region_width: u32,
    pub screen_region_height: u32,
    pub display: CGDisplay,
    pub pixel_data: Vec<u8>,
    scaling_factor_x: f64, // difference between logical and phyisical resolution
    scaling_factor_y: f64
}

impl Screen {
    pub fn new() -> Self {
        unsafe {
            let main_display_id = display::CGMainDisplayID();
            let main_display = CGDisplay::new(main_display_id);
            // because of retina display,  and scaling factors, image captured can be double the size
            // for that detection of retina is needed to divide all the pixel positions
            // by the factor. As far as i understood it should actually always be 2 but leaving it like this
            // shouldnt produce errors and covers any different case
            let image = main_display.image().expect("Failed to grab screen image");
            let image_height = image.height() as i32;
            let image_width = image.width() as i32;
            let screen_width = main_display.pixels_wide() as i32;
            let screen_height = main_display.pixels_high() as i32;
            let scaling_factor_x = image_width as f64 / screen_width as f64;
            let scaling_factor_y = image_height as f64 / screen_height as f64;
            Self {
                screen_height:screen_height,
                screen_width: screen_width,
                screen_region_height: 0,
                screen_region_width:0,
                pixel_data:vec![0u8; (screen_width * screen_height * 4) as usize],
                display: main_display,
                scaling_factor_x:scaling_factor_x,
                scaling_factor_y:scaling_factor_y,
            }
        }
        
    }

    /// returns screen dimensions. All monitors included
    pub fn dimension (&self) -> (i32, i32) {
        let dimensions = (self.screen_width, self.screen_height);
        dimensions
    }
    /// return region dimension which is set up when template is precalculated
    pub fn region_dimension(&self) -> (u32,u32) {
        let dimensions = (self.screen_region_width, self.screen_region_height);
        dimensions
    }


    /// executes convert_bitmap_to_rgba, meaning it converts Vector of values to RGBA and crops the image 
    /// as inputted region area. Not used anywhere at the moment
    pub fn grab_screen_image(&mut self,  region: (u32, u32, u32, u32)) -> ImageBuffer<Rgba<u8>, Vec<u8>>{
        let (x, y, width, height) = region;
        self.screen_region_width = width;
        self.screen_region_height = height;
        self.capture_screen();
        let image = self.convert_bitmap_to_rgba();
        let cropped_image: ImageBuffer<Rgba<u8>, Vec<u8>> = imgtools::cut_screen_region(x, y, width, height, &image);
        cropped_image
    }

    /// executes convert_bitmap_to_grayscale, meaning it converts Vector of values to grayscale and crops the image 
    /// as inputted region area
    pub fn grab_screen_image_grayscale(&mut self,  region: &(u32, u32, u32, u32)) -> ImageBuffer<Luma<u8>, Vec<u8>>{
        let (x, y, width, height) = region;
        self.screen_region_width = *width;
        self.screen_region_height = *height;
        self.capture_screen();
        let image: ImageBuffer<Luma<u8>, Vec<u8>> = self.convert_bitmap_to_grayscale();
        let cropped_image: ImageBuffer<Luma<u8>, Vec<u8>> = imgtools::cut_screen_region(*x, *y, *width, *height, &image);
        cropped_image
    }

    /// captures and saves screenshot of monitors
    pub fn grab_screenshot(&mut self, image_path: &str) {
        self.capture_screen();
        let image = self.convert_bitmap_to_rgba(); 
        image.save(image_path).unwrap();
    }



    /// first order capture screen function. it captures screen image and stores it as vector in self.pixel_data
    fn capture_screen(&mut self) {    
        let image = self.display.image().expect("Failed to grab screen image");

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
        self.pixel_data = pixel_data;
    }

    /// convert vector to Luma Imagebuffer 
    fn convert_bitmap_to_grayscale(&self) -> ImageBuffer<Luma<u8>, Vec<u8>> {
        let mut grayscale_data = Vec::with_capacity((self.screen_width * self.screen_height) as usize);
        for chunk in self.pixel_data.chunks_exact(4) {
            let r = chunk[2] as u32;
            let g = chunk[1] as u32;
            let b = chunk[0] as u32;
            // calculate the grayscale value using the luminance formula
            let gray_value = ((r * 30 + g * 59 + b * 11) / 100) as u8;
            grayscale_data.push(gray_value);
        }
        let mut image = GrayImage::from_raw(
                (self.scaling_factor_x * self.screen_width as f64) as u32,
                (self.scaling_factor_y * self.screen_height as f64) as u32,
                grayscale_data
            ).expect("Couldn't convert to GrayImage");
        resize(&mut image, self.screen_width as u32, self.screen_height as u32, Nearest)
    }

    /// convert vector to RGBA ImageBuffer
    fn convert_bitmap_to_rgba(&self) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        ImageBuffer::from_raw(
            (self.scaling_factor_x * self.screen_width as f64) as u32,
            (self.scaling_factor_y * self.screen_height as f64) as u32,
            self.pixel_data.clone(),
        ).expect("Couldn't convert to ImageBuffer")
    }
}