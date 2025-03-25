extern crate rayon;
extern crate winapi;

use image::{GrayImage, ImageBuffer, ImageError, Luma, Rgba};
use std::mem::size_of;
use std::ptr::null_mut;
use winapi::shared::minwindef::{DWORD, HGLOBAL, LPVOID, UINT};
use winapi::um::wingdi::DIB_RGB_COLORS;
use winapi::um::wingdi::{
    BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits,
    SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, RGBQUAD, SRCCOPY,
};
use winapi::um::winuser::{GetDC, ReleaseDC};

use crate::{imgtools, AutoGuiError};

#[derive(Debug, Clone)]
pub struct Screen {
    pub screen_width: i32,
    pub screen_height: i32,
    pub screen_region_width: u32,
    pub screen_region_height: u32,
    pub pixel_data: Vec<u8>,
    h_screen_dc: *mut winapi::shared::windef::HDC__,
    h_memory_dc: *mut winapi::shared::windef::HDC__,
    h_bitmap: *mut winapi::shared::windef::HBITMAP__,
}

impl Screen {
    ///Creates struct that holds information about screen
    pub fn new() -> Result<Self, AutoGuiError> {
        unsafe {
            let screen_width: i32 = winapi::um::winuser::GetSystemMetrics(0);
            let screen_height = winapi::um::winuser::GetSystemMetrics(1);
            // capture Device Context is a windows struct type that hold information that is written to the screen or printer
            let h_screen_dc: *mut winapi::shared::windef::HDC__ = GetDC(null_mut());
            // here we create a compatible device context in memory, which will have same properties, and we will tell windows to write a screen to it
            let h_mem_dc: *mut winapi::shared::windef::HDC__ = CreateCompatibleDC(h_screen_dc);
            // create a bitmap where the actual pixel array data will be stored
            let h_bitmap: *mut winapi::shared::windef::HBITMAP__ =
                CreateCompatibleBitmap(h_screen_dc, screen_width, screen_height);
            Ok(Screen {
                screen_height: screen_height,
                screen_width: screen_width,
                screen_region_height: screen_height as u32,
                screen_region_width: screen_width as u32,
                pixel_data: vec![0u8; (screen_width * screen_height * 4) as usize],
                h_screen_dc: h_screen_dc,
                h_memory_dc: h_mem_dc,
                h_bitmap: h_bitmap,
            })
        }
    }
    pub fn dimension(&self) -> (i32, i32) {
        return (self.screen_width, self.screen_height);
    }

    pub fn region_dimension(&self) -> (u32, u32) {
        return (self.screen_region_width, self.screen_region_height);
    }

    /// clear memory and delete screen
    pub fn destroy(&self) {
        unsafe {
            DeleteObject(self.h_bitmap as HGLOBAL);
            DeleteDC(self.h_memory_dc);
            ReleaseDC(null_mut(), self.h_screen_dc);
        }
    }

    /// captures screen and returns Imagebuffer in RGBA cropped for the selected region
    pub fn grab_screen_image(
        &mut self,
        region: (u32, u32, u32, u32),
    ) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, AutoGuiError> {
        let (x, y, width, height) = region;
        self.screen_region_width = width;
        self.screen_region_height = height;
        self.capture_screen();
        let image = self.convert_bitmap_to_rgba()?;

        let cropped_image: ImageBuffer<Rgba<u8>, Vec<u8>> =
            imgtools::cut_screen_region(x, y, width, height, &image);
        Ok(cropped_image)
    }

    /// captures screen, and returns grayscale Imagebuffer cropped for the selected region
    pub fn grab_screen_image_grayscale(
        &mut self,
        region: &(u32, u32, u32, u32),
    ) -> Result<ImageBuffer<Luma<u8>, Vec<u8>>, AutoGuiError> {
        let (x, y, width, height) = region;
        self.screen_region_width = *width;
        self.screen_region_height = *height;
        self.capture_screen();
        let image = self.convert_bitmap_to_grayscale()?;

        let cropped_image: ImageBuffer<Luma<u8>, Vec<u8>> =
            imgtools::cut_screen_region(*x, *y, *width, *height, &image);
        Ok(cropped_image)
    }

    /// grabs screen image and saves file at provided
    pub fn grab_screenshot(&mut self, image_path: &str) -> Result<(), AutoGuiError> {
        self.capture_screen();
        let image = self.convert_bitmap_to_rgba()?;
        Ok(image.save(image_path)?)
    }

    fn capture_screen(&mut self) {
        unsafe {
            // here we select the memory device context and the bitmap as main ones
            SelectObject(self.h_memory_dc, self.h_bitmap as HGLOBAL);
            // this function writes data to memory device context
            BitBlt(
                self.h_memory_dc,
                0,
                0,
                self.screen_width,
                self.screen_height,
                self.h_screen_dc,
                0,
                0,
                SRCCOPY,
            );
            let mut bitmap_info = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: size_of::<BITMAPINFOHEADER>() as DWORD,
                    biWidth: self.screen_width,
                    biHeight: -self.screen_height, // Negative to indicate top-down DIB
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB,
                    biSizeImage: 0,
                    biXPelsPerMeter: 0,
                    biYPelsPerMeter: 0,
                    biClrUsed: 0,
                    biClrImportant: 0,
                },
                bmiColors: [RGBQUAD {
                    rgbBlue: 0,
                    rgbGreen: 0,
                    rgbRed: 0,
                    rgbReserved: 0,
                }; 1],
            };

            // Allocate buffer for the bitmap data
            let mut bitmap_data: Vec<u8> =
                vec![0u8; (self.screen_width * self.screen_height * 4) as usize];

            // Get the bitmap data
            GetDIBits(
                self.h_memory_dc,
                self.h_bitmap,
                0,
                self.screen_height as UINT,
                bitmap_data.as_mut_ptr() as LPVOID,
                &mut bitmap_info,
                DIB_RGB_COLORS,
            );

            self.pixel_data = bitmap_data
        }
    }

    fn convert_bitmap_to_grayscale(&self) -> Result<ImageBuffer<Luma<u8>, Vec<u8>>, AutoGuiError> {
        let mut grayscale_data =
            Vec::with_capacity((self.screen_width * self.screen_height) as usize);
        for chunk in self.pixel_data.chunks_exact(4) {
            let r = chunk[2] as u32;
            let g = chunk[1] as u32;
            let b = chunk[0] as u32;
            // calculate the grayscale value using the luminance formula
            let gray_value = ((r * 30 + g * 59 + b * 11) / 100) as u8;
            grayscale_data.push(gray_value);
        }

        GrayImage::from_raw(
            self.screen_width as u32,
            self.screen_height as u32,
            grayscale_data,
        )
        .ok_or(AutoGuiError::ImgError(
            "could not convert image to grayscale".to_string(),
        ))
    }

    fn convert_bitmap_to_rgba(&self) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, AutoGuiError> {
        ImageBuffer::from_raw(
            self.screen_width as u32,
            self.screen_height as u32,
            self.pixel_data.clone(),
        )
        .ok_or(AutoGuiError::ImgError(
            "failed to convert to RGBA".to_string(),
        ))
    }
}
