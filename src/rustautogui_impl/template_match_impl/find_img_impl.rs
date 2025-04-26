use crate::core::{template_match, template_match::open_cl::OclVersion};
use crate::data::*;
use crate::{AutoGuiError, ImageProcessingError, MatchMode};

use image::{ImageBuffer, Luma};

pub use std::{collections::HashMap, env, fmt, fs, path::Path, str::FromStr};

impl crate::RustAutoGui {
    /// Searches for prepared template on screen.
    /// On windows only main monitor search is supported, while on linux, all monitors work.
    /// more details in README
    #[cfg(not(feature = "lite"))]
    #[allow(unused_variables)]
    pub fn find_image_on_screen(
        &mut self,
        precision: f32,
    ) -> Result<Option<Vec<(u32, u32, f32)>>, AutoGuiError> {
        /// searches for image on screen and returns found locations in vector format
        let image: ImageBuffer<Luma<u8>, Vec<u8>> = self
            .screen
            .grab_screen_image_grayscale(&self.template_data.region)?;

        if self.debug {
            let debug_path = Path::new("debug");
            if !debug_path.exists() {
                match fs::create_dir_all(debug_path) {
                    Ok(_) => {
                        println!("Created a debug folder in your root for saving segmented template images");
                        match image.save("debug/screen_capture.png") {
                            Ok(_) => (),
                            Err(x) => println!("{}", x.to_string()),
                        };
                    }
                    Err(x) => {
                        println!("Failed to create debug folder");
                        println!("{}", x.to_string());
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
            .map(|(mut x, mut y, corr)| {
                x = x + self.template_data.region.0 + (self.template_width / 2);
                y = y + self.template_data.region.1 + (self.template_height / 2);
                (x, y, *corr)
            })
            .collect();

        return Ok(Some(locations_ajusted));
    }

    // for macOS with retina display, two runs are made. One for resized template
    // and if not found , then second for normal sized template
    // since the function recursively calls find_stored_image_on_screen -> run_macos_xcorr_with_backup
    // covers are made to not run it for backup aswell
    #[cfg(not(feature = "lite"))]
    #[cfg(target_os = "macos")]
    fn run_macos_xcorr_with_backup(
        &mut self,
        image: ImageBuffer<Luma<u8>, Vec<u8>>,
        precision: f32,
    ) -> Result<Option<Vec<(u32, u32, f32)>>, AutoGuiError> {
        let first_match = self.run_x_corr(image, precision);
        // if retina and if this is not already a recursively ran backup
        if ((self.screen.screen_data.scaling_factor_x > 1.0)
            | (self.screen.screen_data.scaling_factor_y > 1.0))
            & (!self.alias_used.contains(DEFAULT_BCKP_ALIAS))
        {
            match first_match? {
                Some(result) => return Ok(Some(result)),
                None => {
                    let mut bckp_alias = String::new();

                    // if its not a single image search, create a alias_backup hash
                    if self.alias_used != DEFAULT_ALIAS.to_string() {
                        bckp_alias.push_str(self.alias_used.as_str());
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
    #[cfg(not(feature = "lite"))]
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
    #[cfg(not(feature = "lite"))]
    /// find image stored under provided alias
    pub fn find_stored_image_on_screen(
        &mut self,
        precision: f32,
        alias: &str,
    ) -> Result<Option<Vec<(u32, u32, f32)>>, AutoGuiError> {
        let (prepared_data, region, match_mode) = self
            .template_data
            .prepared_data_stored
            .get(alias)
            .ok_or(AutoGuiError::AliasError(
                "No template stored with selected alias".to_string(),
            ))?;
        // save to reset after finished
        let backup = BackupData {
            starting_data: self.template_data.prepared_data.clone(),
            starting_region: self.template_data.region.clone(),
            starting_match_mode: self.template_data.match_mode.clone(),
            starting_template_height: self.template_height.clone(),
            starting_template_width: self.template_width.clone(),
            starting_alias_used: self.template_data.alias_used.clone(),
        };

        self.template_data.alias_used = alias.into();
        self.template_data.prepared_data = prepared_data.clone();
        self.screen.screen_data.screen_region_width = region.2;
        self.screen.screen_data.screen_region_height = region.3;
        self.template_data.region = *region;
        self.template_data.match_mode = Some(match_mode.clone());
        match prepared_data {
            PreparedData::FFT(data) => {
                self.template_width = data.template_width;
                self.template_height = data.template_height;
            }
            PreparedData::Segmented(data) => {
                self.template_width = data.template_width;
                self.template_height = data.template_height;
            }
            PreparedData::None => Err(ImageProcessingError::new("No prepared data loaded"))?,
        };
        let points = self.find_image_on_screen(precision)?;
        // reset to starting info
        backup.update_rustautogui(self);

        Ok(points)
    }

    #[cfg(not(feature = "lite"))]
    /// loops until stored image is found and returns found values, or until it times out
    pub fn loop_find_stored_image_on_screen(
        &mut self,
        precision: f32,
        timeout: u64,
        alias: &str,
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
            let result = self.find_stored_image_on_screen(precision, alias)?;
            match result {
                Some(r) => return Ok(Some(r)),
                None => continue,
            }
        }
    }
    #[cfg(not(feature = "lite"))]
    /// searches for image stored under provided alias and moves mouse to position
    pub fn find_stored_image_on_screen_and_move_mouse(
        &mut self,
        precision: f32,
        moving_time: f32,
        alias: &str,
    ) -> Result<Option<Vec<(u32, u32, f32)>>, AutoGuiError> {
        let (prepared_data, region, match_mode) = self
            .template_data
            .prepared_data_stored
            .get(alias)
            .ok_or(AutoGuiError::AliasError(
                "No template stored with selected alias".to_string(),
            ))?;
        // save to reset after finished
        let backup = BackupData {
            starting_data: self.template_data.prepared_data.clone(),
            starting_region: self.template_data.region.clone(),
            starting_match_mode: self.template_data.match_mode.clone(),
            starting_template_height: self.template_height.clone(),
            starting_template_width: self.template_width.clone(),
            starting_alias_used: self.template_data.alias_used.clone(),
        };
        self.template_data.alias_used = alias.into();
        self.template_data.prepared_data = prepared_data.clone();
        self.template_data.region = *region;
        self.screen.screen_data.screen_region_width = region.2;
        self.screen.screen_data.screen_region_height = region.3;
        self.template_data.match_mode = Some(match_mode.clone());
        match prepared_data {
            PreparedData::FFT(data) => {
                self.template_width = data.template_width;
                self.template_height = data.template_height;
            }
            PreparedData::Segmented(data) => {
                self.template_width = data.template_width;
                self.template_height = data.template_height;
            }
            PreparedData::None => Err(ImageProcessingError::new("No prepared data loaded"))?,
        };
        let found_points = self.find_image_on_screen_and_move_mouse(precision, moving_time);

        // reset to starting info
        backup.update_rustautogui(self);

        found_points
    }
    #[cfg(not(feature = "lite"))]
    /// loops until stored image is found and moves mouse
    pub fn loop_find_stored_image_on_screen_and_move_mouse(
        &mut self,
        precision: f32,
        moving_time: f32,
        timeout: u64,
        alias: &str,
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
            let result =
                self.find_stored_image_on_screen_and_move_mouse(precision, moving_time, alias)?;
            match result {
                Some(r) => return Ok(Some(r)),
                None => continue,
            }
        }
    }
    #[cfg(not(feature = "lite"))]
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

        return Ok(Some(locations));
    }
    #[cfg(not(feature = "lite"))]
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

    #[cfg(not(feature = "lite"))]
    fn run_x_corr(
        &mut self,
        image: ImageBuffer<Luma<u8>, Vec<u8>>,
        precision: f32,
    ) -> Result<Option<Vec<(u32, u32, f32)>>, AutoGuiError> {
        let match_mode = self.template_data.match_mode.clone().ok_or(ImageProcessingError::new("No template chosen and no template data prepared. Please run load_and_prepare_template before searching image on screen"))?;
        let start = std::time::Instant::now();
        let found_locations: Vec<(u32, u32, f32)> = match match_mode {
            MatchMode::FFT => {
                println!("Running FFT mode");
                let data = match &self.template_data.prepared_data {
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
                println!("Running Segmented mode");
                let data = match &self.template_data.prepared_data {
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
                let data = match &self.template_data.prepared_data {
                    PreparedData::Segmented(data) => data,
                    _ => Err(ImageProcessingError::new(
                        "error in prepared data type. Matchmode does not match prepare data type",
                    ))?,
                };
                let gpu_memory_pointers = self
                    .opencl_data
                    .ocl_buffer_storage
                    .get(&self.template_data.alias_used)
                    .ok_or(ImageProcessingError::new("Error , no OCL data prepared"))?;
                template_match::open_cl::gui_opencl_ncc_template_match(
                    &self.opencl_data.ocl_queue,
                    &self.opencl_data.ocl_program,
                    self.opencl_data.ocl_workgroup_size,
                    &self.opencl_data.ocl_kernel_storage[&self.template_data.alias_used],
                    gpu_memory_pointers,
                    precision,
                    &image,
                    data,
                    OclVersion::V1,
                )?
            }
            #[cfg(feature = "opencl")]
            MatchMode::SegmentedOclV2 => {
                let data = match &self.template_data.prepared_data {
                    PreparedData::Segmented(data) => data,
                    _ => Err(ImageProcessingError::new(
                        "error in prepared data type. Matchmode does not match prepare data type",
                    ))?,
                };
                let gpu_memory_pointers = self
                    .opencl_data
                    .ocl_buffer_storage
                    .get(&self.template_data.alias_used)
                    .ok_or(ImageProcessingError::new("Error , no OCL data prepared"))?;
                template_match::open_cl::gui_opencl_ncc_template_match(
                    &self.opencl_data.ocl_queue,
                    &self.opencl_data.ocl_program,
                    self.opencl_data.ocl_workgroup_size,
                    &self.opencl_data.ocl_kernel_storage[&self.template_data.alias_used],
                    gpu_memory_pointers,
                    precision,
                    &image,
                    data,
                    OclVersion::V2,
                )?
            }
        };
        if found_locations.len() > 0 {
            if self.debug {
                let corrected_found_location: (u32, u32, f32);
                let x = found_locations[0].0 as u32
                    + (self.template_width / 2) as u32
                    + self.template_data.region.0 as u32;
                let y = found_locations[0].1 as u32
                    + (self.template_height / 2) as u32
                    + self.template_data.region.1 as u32;
                let corr = found_locations[0].2;
                corrected_found_location = (x, y, corr);

                println!(
                    "Location found at x: {}, y {}, corr {} ",
                    corrected_found_location.0,
                    corrected_found_location.1,
                    corrected_found_location.2
                )
            }
            return Ok(Some(found_locations));
        } else {
            return Ok(None);
        };
    }
}
