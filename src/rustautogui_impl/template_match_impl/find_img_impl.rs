#![allow(clippy::type_complexity)]
use crate::core::CaptureableScreen;

use crate::core::template_match;
use crate::data::*;
#[cfg(feature = "opencl")]
use crate::template_match::open_cl::OclVersion;
use crate::{AutoGuiError, ImageProcessingError, MatchMode};
use crate::{DEFAULT_ALIAS, DEFAULT_BCKP_ALIAS};
use image::GrayImage;
use image::{ImageBuffer, Luma};
pub use std::{collections::HashMap, env, fmt, fs, path::Path, str::FromStr};

impl crate::RustAutoGui {
    /// Searches for prepared template on screen.
    /// On windows only main monitor search is supported, while on linux, all monitors work.
    /// more details in README
    #[allow(unused_variables)]
    pub fn find_image_on_screen(
        &mut self,
        precision: f32,
    ) -> Result<Option<Vec<(u32, u32, f32)>>, AutoGuiError> {
        let template = self.get_current_template()?;
        /// searches for image on screen and returns found locations in vector format
        let image: GrayImage = self
            .screen
            .grab_screen_image_grayscale(&template.borrow().region)?;

        if self.debug {
            let debug_path = Path::new("debug");
            if !debug_path.exists() {
                match fs::create_dir_all(debug_path) {
                    Ok(_) => {
                        println!(
                            "Created a debug folder in your root for saving segmented template images"
                        );
                        match image.save("debug/screen_capture.png") {
                            Ok(_) => (),
                            Err(x) => println!("{}", x),
                        };
                    }
                    Err(x) => {
                        println!("Failed to create debug folder");
                        println!("{}", x);
                    }
                };
            }
        };

        #[cfg(target_os = "macos")]
        let locations = match self.run_macos_xcorr_with_backup(image, precision)? {
            Some(x) => x,
            None => return Ok(None),
        };
        #[cfg(not(target_os = "macos"))]
        let locations = match self.run_x_corr(image, precision)? {
            Some(x) => x,
            None => return Ok(None),
        };

        let locations_ajusted: Vec<(u32, u32, f32)> = locations
            .iter()
            .map(|&(mut x, mut y, ref corr)| {
                x = x + template.borrow().region.0 + (template.borrow().width / 2);
                y = y + template.borrow().region.1 + (template.borrow().height / 2);
                (x, y, *corr)
            })
            .collect();

        Ok(Some(locations_ajusted))
    }

    // for macOS with retina display, two runs are made. One for resized template
    // and if not found , then second for normal sized template
    // since the function recursively calls find_stored_image_on_screen -> run_macos_xcorr_with_backup
    // covers are made to not run it for backup aswell

    #[cfg(target_os = "macos")]
    fn run_macos_xcorr_with_backup(
        &mut self,
        image: GrayImage,
        precision: f32,
    ) -> Result<Option<Vec<(u32, u32, f32)>>, AutoGuiError> {
        let first_match = self.run_x_corr(image, precision);
        // if retina and if this is not already a recursively ran backup
        if ((self.screen.screen_data.scaling_factor_x > 1.0)
            | (self.screen.screen_data.scaling_factor_y > 1.0))
            & (!self.current_template.contains(DEFAULT_BCKP_ALIAS))
        {
            match first_match? {
                Some(result) => return Ok(Some(result)),
                None => {
                    let mut bckp_alias = String::new();

                    // if its not a single image search, create a alias_backup hash
                    if self.current_template != DEFAULT_ALIAS.to_string() {
                        bckp_alias.push_str(self.current_template.as_str());
                        bckp_alias.push('_');
                    }
                    bckp_alias.push_str(DEFAULT_BCKP_ALIAS);
                    // this recursively searches again for backup
                    return self.find_stored_image_on_screen(precision, &bckp_alias);
                }
            }
        }
        first_match
    }

    /// loops until image is found and returns found values, or until it times out
    pub fn loop_find_image_on_screen(
        &mut self,
        precision: f32,
        timeout: u64,
    ) -> Result<Option<Vec<(u32, u32, f32)>>, AutoGuiError> {
        if (timeout == 0) & (!self.suppress_warnings) {
            eprintln!(
                "Warning: setting a timeout to 0 on a loop find image initiates an infinite loop"
            )
        }

        let timeout_start = std::time::Instant::now();
        loop {
            if (timeout_start.elapsed().as_secs() > timeout) & (timeout > 0) {
                Err(ImageProcessingError::new(
                    "loop find image timed out. Could not find image",
                ))?;
            }
            let result = self.find_image_on_screen(precision)?;
            match result {
                Some(r) => return Ok(Some(r)),
                None => continue,
            }
        }
    }

    /// find image stored under provided alias
    pub fn find_stored_image_on_screen(
        &mut self,
        precision: f32,
        alias: impl ToString,
    ) -> Result<Option<Vec<(u32, u32, f32)>>, AutoGuiError> {
        self.current_template = alias.to_string();
        let points = self.find_image_on_screen(precision)?;
        Ok(points)
    }

    /// loops until stored image is found and returns found values, or until it times out
    pub fn loop_find_stored_image_on_screen(
        &mut self,
        precision: f32,
        timeout: u64,
        alias: impl ToString,
    ) -> Result<Option<Vec<(u32, u32, f32)>>, AutoGuiError> {
        if (timeout == 0) & (!self.suppress_warnings) {
            eprintln!(
                "Warning: setting a timeout to 0 on a loop find image initiates an infinite loop"
            )
        }
        let timeout_start = std::time::Instant::now();
        self.current_template = alias.to_string();
        loop {
            if (timeout_start.elapsed().as_secs() > timeout) & (timeout > 0) {
                Err(ImageProcessingError::new(
                    "loop find image timed out. Could not find image",
                ))?;
            }
            let result = self.find_image_on_screen(precision)?;
            match result {
                Some(r) => return Ok(Some(r)),
                None => continue,
            }
        }
    }

    /// searches for image stored under provided alias and moves mouse to position
    pub fn find_stored_image_on_screen_and_move_mouse(
        &mut self,
        precision: f32,
        moving_time: f32,
        alias: impl ToString,
    ) -> Result<Option<Vec<(u32, u32, f32)>>, AutoGuiError> {
        self.current_template = alias.to_string();

        self.find_image_on_screen_and_move_mouse(precision, moving_time)
    }

    /// loops until stored image is found and moves mouse
    pub fn loop_find_stored_image_on_screen_and_move_mouse(
        &mut self,
        precision: f32,
        moving_time: f32,
        timeout: u64,
        alias: impl ToString,
    ) -> Result<Option<Vec<(u32, u32, f32)>>, AutoGuiError> {
        if (timeout == 0) & (!self.suppress_warnings) {
            eprintln!(
                "Warning: setting a timeout to 0 on a loop find image initiates an infinite loop"
            )
        }
        let timeout_start = std::time::Instant::now();
        self.current_template = alias.to_string();
        loop {
            if (timeout_start.elapsed().as_secs() > timeout) & (timeout > 0) {
                Err(ImageProcessingError::new(
                    "loop find image timed out. Could not find image",
                ))?;
            }
            let result = self.find_image_on_screen_and_move_mouse(precision, moving_time)?;
            match result {
                Some(r) => return Ok(Some(r)),
                None => continue,
            }
        }
    }

    /// executes find_image_on_screen and moves mouse to the middle of the image.
    pub fn find_image_on_screen_and_move_mouse(
        &mut self,
        precision: f32,
        moving_time: f32,
    ) -> Result<Option<Vec<(u32, u32, f32)>>, AutoGuiError> {
        /// finds coordinates of the image on the screen and moves mouse to it. Returns None if no image found
        ///  Best used in loops
        let found_locations = self.find_image_on_screen(precision)?;

        let locations = match found_locations.clone() {
            Some(locations) => locations,
            None => return Ok(None),
        };

        let (target_x, target_y, _) = locations[0];

        self.move_mouse_to_pos(target_x, target_y, moving_time)?;

        Ok(Some(locations))
    }

    /// loops until image is found and returns found values, or until it times out
    pub fn loop_find_image_on_screen_and_move_mouse(
        &mut self,
        precision: f32,
        moving_time: f32,
        timeout: u64,
    ) -> Result<Option<Vec<(u32, u32, f32)>>, AutoGuiError> {
        if (timeout == 0) & (!self.suppress_warnings) {
            eprintln!(
                "Warning: setting a timeout to 0 on a loop find image initiates an infinite loop"
            )
        }
        let timeout_start = std::time::Instant::now();
        loop {
            if (timeout_start.elapsed().as_secs() > timeout) & (timeout > 0) {
                Err(ImageProcessingError::new(
                    "loop find image timed out. Could not find image",
                ))?;
            }
            let result = self.find_image_on_screen_and_move_mouse(precision, moving_time)?;
            match result {
                Some(e) => return Ok(Some(e)),
                None => continue,
            }
        }
    }

    fn run_x_corr(
        &mut self,
        image: GrayImage,
        precision: f32,
    ) -> Result<Option<Vec<(u32, u32, f32)>>, AutoGuiError> {
        let template = self.get_current_template()?;
        let template = template.borrow();
        let match_mode = template.match_mode.clone().ok_or(ImageProcessingError::new("No template chosen and no template data prepared. Please run load_and_prepare_template before searching image on screen"))?;
        let found_locations: Vec<(u32, u32, f32)> = match match_mode {
            MatchMode::FFT => {
                if self.debug {
                    println!("Running FFT mode");
                }
                let data = match &template.prepared_data {
                    PreparedData::FFT(data) => data,
                    _ => Err(ImageProcessingError::new(
                        "error in prepared data type. Matchmode does not match prepare data type",
                    ))?,
                };
                let found_locations: Vec<(u32, u32, f64)> =
                    template_match::fft_ncc::fft_ncc(&image, precision, data);
                found_locations
                    .into_iter()
                    .map(|(x, y, value)| (x, y, value as f32))
                    .collect()
            }
            MatchMode::Segmented => {
                if self.debug {
                    println!("Running Segmented mode");
                }
                let data = match &template.prepared_data {
                    PreparedData::Segmented(data) => data,
                    _ => Err(ImageProcessingError::new(
                        "error in prepared data type. Matchmode does not match prepare data type",
                    ))?,
                };
                template_match::segmented_ncc::fast_ncc_template_match(
                    &image,
                    precision,
                    data,
                    &self.debug,
                )
            }
            #[cfg(feature = "opencl")]
            MatchMode::SegmentedOcl => {
                let data = match &template.prepared_data {
                    PreparedData::Segmented(data) => data,
                    _ => Err(ImageProcessingError::new(
                        "error in prepared data type. Matchmode does not match prepare data type",
                    ))?,
                };
                let gpu_memory_pointers = self
                    .opencl_data
                    .ocl_buffer_storage
                    .get(&self.current_template)
                    .ok_or(ImageProcessingError::new("Error , no OCL data prepared"))?;
                template_match::open_cl::gui_opencl_ncc_template_match(
                    &self.opencl_data.ocl_queue,
                    &self.opencl_data.ocl_program,
                    self.opencl_data.ocl_workgroup_size,
                    &self.opencl_data.ocl_kernel_storage[&self.current_template],
                    gpu_memory_pointers,
                    precision,
                    &image,
                    data,
                    OclVersion::V1,
                )?
            }
            #[cfg(feature = "opencl")]
            MatchMode::SegmentedOclV2 => {
                let data = match &template.prepared_data {
                    PreparedData::Segmented(data) => data,
                    _ => Err(ImageProcessingError::new(
                        "error in prepared data type. Matchmode does not match prepare data type",
                    ))?,
                };
                let gpu_memory_pointers = self
                    .opencl_data
                    .ocl_buffer_storage
                    .get(&self.current_template)
                    .ok_or(ImageProcessingError::new("Error , no OCL data prepared"))?;
                template_match::open_cl::gui_opencl_ncc_template_match(
                    &self.opencl_data.ocl_queue,
                    &self.opencl_data.ocl_program,
                    self.opencl_data.ocl_workgroup_size,
                    &self.opencl_data.ocl_kernel_storage[&self.current_template],
                    gpu_memory_pointers,
                    precision,
                    &image,
                    data,
                    OclVersion::V2,
                )?
            }
        };
        if !found_locations.is_empty() {
            if self.debug {
                let x = found_locations[0].0 + (template.width / 2) + template.region.0;
                let y = found_locations[0].1 + (template.height / 2) + template.region.1;
                let corr = found_locations[0].2;
                let corrected_found_location = (x, y, corr);

                println!(
                    "Location found at x: {}, y {}, corr {} ",
                    corrected_found_location.0,
                    corrected_found_location.1,
                    corrected_found_location.2
                )
            }
            Ok(Some(found_locations))
        } else {
            Ok(None)
        }
    }
}
