use crate::core::{CaptureableScreen, keyboard::Keyboard};
#[cfg(not(feature = "lite"))]
use crate::errors::AutoGuiError;
use crate::imgtools;
#[cfg(not(feature = "lite"))]
use crate::imgtools::convert_rgba_to_bw;
use crossbeam_channel::Receiver;
#[cfg(not(feature = "lite"))]
use image::{GrayImage, ImageBuffer, Luma, Rgba, RgbaImage};
use std::{
    sync::{Arc, RwLock, mpsc::RecvTimeoutError},
    thread,
    time::Duration,
};
#[cfg(not(feature = "lite"))]
use waycap_rs::{Capture, RgbaImageEncoder};
pub struct WaylandScreen {
    capture: Capture<RgbaImageEncoder>,
    latest_img: Receiver<RgbaImage>,
    img: RgbaImage,
}

impl CaptureableScreen for WaylandScreen {
    fn new() -> Result<Self, AutoGuiError>
    where
        Self: std::marker::Sized,
    {
        let mut cap = Capture::new_with_encoder(RgbaImageEncoder::default(), false, 60).unwrap();
        let recv = cap.get_output();
        let recv_thread = recv.clone();
        thread::spawn(move || {
            loop {
                // drain channel to get latest message
                while recv_thread.len() > 1 {
                    recv_thread.try_recv().ok();
                }
                // avoid busy waiting
                std::thread::yield_now();
            }
        });
        let img = recv.recv().expect("Could not get first image");
        let screen = WaylandScreen {
            capture: cap,
            latest_img: recv,
            img,
        };
        screen.dimension();
        Ok(screen)
    }
    /// returns screen dimensions. All monitors included
    fn dimension(&self) -> (i32, i32) {
        (self.img.width() as i32, self.img.height() as i32)
    }

    fn destroy(&mut self) {
        self.capture.finish().ok();
    }

    #[allow(dead_code)]
    /// executes convert_bitmap_to_rgba, meaning it converts Vector of values to RGBA and crops the image
    /// as inputted region area. Not used anywhere at the moment
    #[cfg(not(feature = "lite"))]
    fn grab_screen_image(
        &mut self,
        (x, y, width, height): (u32, u32, u32, u32),
    ) -> Result<RgbaImage, AutoGuiError> {
        self.capture_screen()?;
        Ok(image::imageops::crop(&mut self.img, x, y, width, height).to_image())
    }

    /// executes convert_bitmap_to_grayscale, meaning it converts Vector of values to grayscale and crops the image
    /// as inputted region area
    #[cfg(not(feature = "lite"))]
    fn grab_screen_image_grayscale(
        &mut self,
        &(x, y, width, height): &(u32, u32, u32, u32),
    ) -> Result<GrayImage, AutoGuiError> {
        self.capture_screen()?;
        convert_rgba_to_bw(imgtools::cut_screen_region(x, y, width, height, &self.img))
    }
    #[cfg(not(feature = "lite"))]
    /// captures and saves screenshot of monitors
    fn grab_screenshot(&mut self, image_path: &str) -> Result<(), AutoGuiError> {
        self.capture_screen()?;
        self.img.save(image_path).map_err(Into::into)
    }
    #[cfg(not(feature = "lite"))]
    /// first order capture screen function. it captures screen image and stores it as vector in self.pixel_data
    fn capture_screen(&mut self) -> Result<(), AutoGuiError> {
        if !self.latest_img.is_empty() {
            self.img = self
                .latest_img
                .try_recv()
                .expect("reciever was not empty, but could not get a image");
        }
        Ok(())
    }
    #[cfg(not(feature = "lite"))]
    /// convert vector to Luma Imagebuffer
    fn convert_bitmap_to_grayscale(&self) -> Result<GrayImage, AutoGuiError> {
        convert_rgba_to_bw(self.img.clone())
    }
    #[cfg(not(feature = "lite"))]
    /// convert vector to RGBA ImageBuffer
    fn convert_bitmap_to_rgba(&self) -> Result<RgbaImage, AutoGuiError> {
        Ok(self.img.clone())
    }

    fn create_keyboard(&mut self) -> crate::core::keyboard::Keyboard {
        Keyboard::Wayland
    }

    fn create_mouse(&mut self) -> crate::core::mouse::Mouse {
        crate::core::mouse::Mouse::Wayland
    }
}
