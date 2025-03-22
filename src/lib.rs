#![allow(unused_doc_comments, unused_imports)]
pub mod imgtools;
pub mod normalized_x_corr;

use image::{
    imageops::{resize, FilterType::Nearest},
    DynamicImage, GrayImage, ImageBuffer, Luma, Pixel, Primitive, Rgb, Rgba,
};
use normalized_x_corr::fast_segment_x_corr::prepare_template_picture;

use rustfft::{num_complex::Complex, num_traits::ToPrimitive};

use std::{collections::HashMap, env, fs, path::Path, str::FromStr};

#[cfg(target_os = "windows")]
pub use crate::{keyboard::windows::Keyboard, mouse::windows::Mouse, screen::windows::Screen};

#[cfg(target_os = "linux")]
pub use crate::{keyboard::linux::Keyboard, mouse::linux::Mouse, screen::linux::Screen};

#[cfg(target_os = "macos")]
pub use crate::{keyboard::macos::Keyboard, mouse::macos::Mouse, screen::macos::Screen};

pub mod keyboard;
pub mod mouse;
pub mod screen;

/// Struct of prepared data for each correlation method used
/// Segmented consists of two image vectors and associated mean value, sum of squared deviations, sizes
/// FFT vector consists of template vector converted to frequency domain and conjugated, sum squared deviations, size and padded size
pub enum PreparedData {
    Segmented(
        (
            Vec<(u32, u32, u32, u32, f32)>, // template_segments_fast
            Vec<(u32, u32, u32, u32, f32)>, // template_segments_slow
            u32,                            // template_width
            u32,                            // template_height
            f32,                            // segment_sum_squared_deviations_fast
            f32,                            // segment_sum_squared_deviations_slow
            f32,                            // expected_corr_fast
            f32,                            // expected_corr_slow
            f32,                            // segments_mean_fast
            f32,                            // segments_mean_slow
        ),
    ),
    FFT(
        (
            Vec<Complex<f32>>, // template_conj_freq
            f32,               // template_sum_squared_deviations
            u32,               // template_width
            u32,               // template_height
            u32,               // padded_size
        ),
    ),

    None,
}

impl Clone for PreparedData {
    fn clone(&self) -> Self {
        match self {
            PreparedData::Segmented(data) => PreparedData::Segmented(data.clone()),
            PreparedData::FFT(data) => PreparedData::FFT(data.clone()),
            PreparedData::None => PreparedData::None,
        }
    }
}

/// Matchmode Segmented correlation and Fourier transform correlation
#[derive(PartialEq)]
pub enum MatchMode {
    Segmented,
    FFT,
}
impl Clone for MatchMode {
    fn clone(&self) -> Self {
        match self {
            MatchMode::Segmented => MatchMode::Segmented,
            MatchMode::FFT => MatchMode::Segmented,
        }
    }
}

struct BackupData {
    starting_data: PreparedData,
    starting_region: (u32, u32, u32, u32),
    starting_match_mode: Option<MatchMode>,
    starting_template_height: u32,
    starting_template_width: u32,
}
impl BackupData {
    fn update_rustautogui(self, target: &mut RustAutoGui) {
        target.prepared_data = self.starting_data.clone();
        target.region = self.starting_region;
        target.match_mode = self.starting_match_mode;
        target.screen.screen_region_width = self.starting_region.2;
        target.screen.screen_region_height = self.starting_region.3;
        target.template_width = self.starting_template_width;
        target.template_height = self.starting_template_height;
    }
}

/// Main struct for Rustautogui
/// Struct gets assigned keyboard, mouse and struct to it implemented functions execute commands from each of assigned substructs
/// executes also correlation algorithms when doing find_image_on_screen
#[allow(dead_code)]
pub struct RustAutoGui {
    // most of the fields are set up in load_and_prepare_template method
    template: Option<ImageBuffer<Luma<u8>, Vec<u8>>>,
    prepared_data: PreparedData, // used direct load and search
    prepared_data_stored: HashMap<String, (PreparedData, (u32, u32, u32, u32))>, // used if multiple images need to be preloaded and searched. Good for simultaneous search
    debug: bool,
    template_height: u32,
    template_width: u32,
    keyboard: Keyboard,
    mouse: Mouse,
    screen: Screen,
    match_mode: Option<MatchMode>,
    max_segments: Option<u32>,
    region: (u32, u32, u32, u32),
    suppress_warnings: bool,
}
impl RustAutoGui {
    /// initiation of screen, keyboard and mouse that are assigned to new rustautogui struct.
    /// all the other struct fields are initiated as 0 or None
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    pub fn new(debug: bool) -> Result<Self, &'static str> {
        // initiation of screen, keyboard and mouse
        // on windows there is no need to share display pointer accross other structs
        let screen = Screen::new()?;
        let keyboard = Keyboard::new();
        let mouse_struct: Mouse = Mouse::new();
        // check for env variable to suppress warnings, otherwise set default false value
        let suppress_warnings = env::var("RUSTAUTOGUI_SUPPRESS_WARNINGS")
            .map(|val| val == "1" || val.eq_ignore_ascii_case("true"))
            .unwrap_or(false); // Default: warnings are NOT suppressed
        Ok(Self {
            template: None,
            prepared_data: PreparedData::None,
            prepared_data_stored: HashMap::new(),
            debug: debug,
            template_width: 0,
            template_height: 0,
            keyboard: keyboard,
            mouse: mouse_struct,
            screen: screen,
            match_mode: None,
            max_segments: None,
            region: (0, 0, 0, 0),
            suppress_warnings: suppress_warnings,
        })
    }

    /// initiation of screen, keyboard and mouse that are assigned to new rustautogui struct.
    /// all the other struct fields are initiated as 0 or None
    #[cfg(target_os = "linux")]
    pub fn new(debug: bool) -> Result<Self, &'static str> {
        // on linux, screen display pointer is shared to keyboard and mouse
        // x11 works like that and initiation of individual display objects
        // under each struct wouldnt be preferable
        let screen = Screen::new();
        let keyboard = Keyboard::new(screen.display);
        let mouse_struct: Mouse = Mouse::new(screen.display, screen.root_window);
        // check for env variable to suppress warnings, otherwise set default false value
        let suppress_warnings = env::var("RUSTAUTOGUI_SUPPRESS_WARNINGS")
            .map(|val| val == "1" || val.eq_ignore_ascii_case("true"))
            .unwrap_or(false); // Default: warnings are NOT suppressed
        Ok(Self {
            template: None,
            prepared_data: PreparedData::None,
            prepared_data_stored: HashMap::new(),
            debug: debug,
            template_width: 0,
            template_height: 0,
            keyboard: keyboard,
            mouse: mouse_struct,
            screen: screen,
            match_mode: None,
            max_segments: None,
            region: (0, 0, 0, 0),
            suppress_warnings: suppress_warnings,
        })
    }

    pub fn set_suppress_warnings(&mut self, suppress: bool) {
        self.suppress_warnings = suppress;
    }

    /// changes debug mode
    pub fn change_debug_state(&mut self, state: bool) {
        self.debug = state;
    }

    pub fn get_screen_size(&mut self) -> (i32, i32) {
        self.screen.dimension()
    }

    /// saves screenshot and saves it at provided path
    pub fn save_screenshot(&mut self, path: &str) -> Result<(), String> {
        self.screen.grab_screenshot(path)?;
        Ok(())
    }

    fn check_if_region_out_of_bound(&mut self) -> Result<(), &'static str> {
        let region_x = self.region.0;
        let region_y = self.region.1;
        let region_width = self.region.2;
        let region_height = self.region.3;

        if (region_x + region_width > self.screen.screen_width as u32)
            | (region_y + region_height > self.screen.screen_height as u32)
        {
            return Err("Selected region out of bounds");
        }

        // this is a redundant check since this case should be covered by the
        // next region check, but leaving it
        if (self.template_width > self.screen.screen_width as u32)
            | (self.template_height > self.screen.screen_height as u32)
        {
            return Err("Selected template is larger than detected screen");
        }

        if (self.template_width > region_width) | (self.template_height > region_height) {
            return Err("Selected template is larger than selected search region. ");
        }
        Ok(())
    }

    ////////////////////////////// image functions

    /// Loads template from file on provided path
    pub fn prepare_template_from_file(
        &mut self,
        template_path: &str,
        region: Option<(u32, u32, u32, u32)>,
        match_mode: MatchMode,
        max_segments: Option<u32>,
    ) -> Result<(), String> {

        let template: ImageBuffer<Luma<u8>, Vec<u8>> = imgtools::load_image_bw(template_path)?;
        self.prepare_template_picture_bw(template, region, match_mode, max_segments, None)
    }

    /// prepare from imagebuffer, works only on types RGB/RGBA/Luma
    pub fn prepare_template_from_imagebuffer<P, T>(
        &mut self,
        image: ImageBuffer<P, Vec<T>>,
        region: Option<(u32, u32, u32, u32)>,
        match_mode: MatchMode,
        max_segments: Option<u32>,
    ) -> Result<(), String>
    where
        P: Pixel<Subpixel = T> + 'static,
        T: Primitive + ToPrimitive + 'static,
    {
        let color_scheme = imgtools::check_imagebuffer_color_scheme(&image)?;
        let luma_img = imgtools::convert_t_imgbuffer_to_luma(&image, &color_scheme)?;
        self.prepare_template_picture_bw(luma_img, region, match_mode, max_segments, None)?;
        Ok(())
    }

    /// Only works on encoded images. uses image::load_from_memory() which reads first bytes of image which contain metadata depending on format.
    pub fn prepare_template_from_raw_encoded(
        &mut self,
        img_raw: &[u8],
        region: Option<(u32, u32, u32, u32)>,
        match_mode: MatchMode,
        max_segments: Option<u32>,
    ) -> Result<(), String> {
        let image = image::load_from_memory(img_raw).map_err(|e| {
            let mut err_msg = "Prepare template from raw only works on encoded images. The original error message was \n".to_string();
            err_msg.push_str(e.to_string().as_str());
            err_msg
            })?;
        self.prepare_template_picture_bw(image.to_luma8(), region, match_mode, max_segments, None)?;
        Ok(())
    }

    /// Store template data for multiple image search
    pub fn store_template_from_file(
        &mut self,
        template_path: &str,
        region: Option<(u32, u32, u32, u32)>,
        match_mode: MatchMode,
        max_segments: Option<u32>,
        alias: String,
    ) -> Result<(), String> {
        let template: ImageBuffer<Luma<u8>, Vec<u8>> = imgtools::load_image_bw(template_path)?;
        self.prepare_template_picture_bw(template, region, match_mode, max_segments, Some(alias))
    }

    /// Load template from imagebuffer and store prepared template data for multiple image search
    pub fn store_template_from_imagebuffer<P, T>(
        &mut self,
        image: ImageBuffer<P, Vec<T>>,
        region: Option<(u32, u32, u32, u32)>,
        match_mode: MatchMode,
        max_segments: Option<u32>,
        alias: String,
    ) -> Result<(), String>
    where
        P: Pixel<Subpixel = T> + 'static,
        T: Primitive + ToPrimitive + 'static,
    {
        let color_scheme = imgtools::check_imagebuffer_color_scheme(&image)?;
        let luma_img = imgtools::convert_t_imgbuffer_to_luma(&image, &color_scheme)?;
        self.prepare_template_picture_bw(luma_img, region, match_mode, max_segments, Some(alias))?;
        Ok(())
    }

    /// Load template from encoded raw bytes and store prepared template data for multiple image search
    pub fn store_template_from_raw_encoded(
        &mut self,
        img_raw: &[u8],
        region: Option<(u32, u32, u32, u32)>,
        match_mode: MatchMode,
        max_segments: Option<u32>,
        alias: String,
    ) -> Result<(), String> {
        let image = image::load_from_memory(img_raw).map_err(|e| {
            let mut err_msg = "Prepare template from raw only works on encoded images. The original error message was \n".to_string();
            err_msg.push_str(e.to_string().as_str());
            err_msg
            })?;
        self.prepare_template_picture_bw(
            image.to_luma8(),
            region,
            match_mode,
            max_segments,
            Some(alias),
        )?;
        Ok(())
    }

    // main prepare template picture which takes ImageBuffer Luma u8. all the other variants
    // of load_and prepare call this function
    fn prepare_template_picture_bw(
        &mut self,
        mut template: ImageBuffer<Luma<u8>, Vec<u8>>,
        region: Option<(u32, u32, u32, u32)>,
        match_mode: MatchMode,
        max_segments: Option<u32>,
        alias: Option<String>,
    ) -> Result<(), String> {
        
        //resize and adjust if retina screen is used
        #[cfg(target_os = "macos")]
        {
            template = resize(
                &template,
                template.width() / self.screen.scaling_factor_x as u32,
                template.height() / self.screen.scaling_factor_y as u32,
                Nearest,
            );
        }
        let (template_width, template_height) = template.dimensions();

        self.max_segments = max_segments;
        let region = match region {
            Some(region_tuple) => region_tuple,
            None => {
                let (screen_width, screen_height) = self.screen.dimension();
                (0, 0, screen_width as u32, screen_height as u32)
            }
        };

        self.check_if_region_out_of_bound()?;
        // do the rest of preparation calculations depending on the matchmode
        // FFT pads the image, does fourier transformations,
        // calculates conjugate and inverses transformation on template
        // Segmented creates vector of picture segments with coordinates, dimensions and average pixel value
        let (template_data, match_mode) = match match_mode {
            MatchMode::FFT => {
                let prepared_data =
                    PreparedData::FFT(normalized_x_corr::fft_ncc::prepare_template_picture(
                        &template, &region.2, &region.3,
                    ));
                let match_mode = Some(MatchMode::FFT);
                (prepared_data, match_mode)
            }
            MatchMode::Segmented => {
                let prepared_data: (
                    Vec<(u32, u32, u32, u32, f32)>,
                    Vec<(u32, u32, u32, u32, f32)>,
                    u32,
                    u32,
                    f32,
                    f32,
                    f32,
                    f32,
                    f32,
                    f32,
                ) = normalized_x_corr::fast_segment_x_corr::prepare_template_picture(
                    &template,
                    max_segments,
                    &self.debug,
                );
                // mostly happens due to using too complex image with small max segments value
                if (prepared_data.0.len() == 1) | (prepared_data.1.len() == 1) {
                    return Err(String::from("Error in creating segmented template image. To resolve: either increase the max_segments, use FFT matching mode or use smaller template image"));
                }
                let match_mode = Some(MatchMode::Segmented);
                (PreparedData::Segmented(prepared_data), match_mode)
            }
        };
        // if storing the image , we just save it to Hashmap
        // if not storing, then we change struct attributes to fit the single loaded image search
        match alias {
            Some(name) => {
                self.prepared_data_stored
                    .insert(name, (template_data, region));
            }
            None => {
                self.region = region;
                self.prepared_data = template_data;
                self.match_mode = match_mode;
                // update screen struct
                self.screen.screen_region_width = region.2;
                self.screen.screen_region_height = region.3;
                // update struct values
                self.template_width = template_width;
                self.template_height = template_height;
                // convert to option and store in struct
                self.template = Some(template.clone());
            }
        }

        return Ok(());
    }

    /// change certain settings for prepared template, like region, match_mode or max_segments. If MatchMode is not changed, whole template
    /// recalculation may still be needed if certain other parameters are changed, depending on current MatchMode.
    /// For FFT, changing region starts complete recalculation again, because of change in zero pad image. While changing
    /// max_segments calls recalculation of template for Segmented matchmode
    #[allow(unused_variables)]
    pub fn change_prepared_settings(
        &mut self,
        region: Option<(u32, u32, u32, u32)>,
        match_mode: MatchMode,
        max_segments: Option<u32>,
    ) {
        let template = self.template.clone();
        let template = match template {
            Some(image) => image,
            None => {
                println!("No template loaded! Please use load_and_prepare_template method before changing prepared settings");
                return;
            }
        };

        // unpack region , or set default if none (whole screen)
        let region = match region {
            Some(region_tuple) => region_tuple,
            None => {
                let (screen_width, screen_height) = self.screen.dimension();
                (0, 0, screen_width as u32, screen_height as u32)
            }
        };

        // check if template recalculation is needed
        // if region changed and FFT used, recalculate because of image padding
        // for Segmented, recalculate if max_segments changed
        match match_mode {
            MatchMode::FFT => {
                if self.region == region && self.match_mode == Some(MatchMode::FFT) {
                    if self.debug {
                        println!("Keeping same template data");
                    }
                } else {
                    if self.debug {
                        println!("Recalculating template data");
                    }
                    let prepared_data =
                        PreparedData::FFT(normalized_x_corr::fft_ncc::prepare_template_picture(
                            &template, &region.2, &region.3,
                        ));
                    self.prepared_data = prepared_data;
                    self.match_mode = Some(MatchMode::FFT);
                }
            }
            MatchMode::Segmented => {
                // no need to recalculate if max segments havent changed or if match mode has not changed
                if self.match_mode == Some(MatchMode::Segmented)
                    && self.max_segments == max_segments
                {
                    if self.debug {
                        println!("Keeping same template data");
                    }
                } else {
                    if self.debug {
                        println!("Recalculating template data");
                    }
                    let prepared_data = PreparedData::Segmented(
                        normalized_x_corr::fast_segment_x_corr::prepare_template_picture(
                            &template,
                            None,
                            &self.debug,
                        ),
                    );
                    self.prepared_data = prepared_data;
                    self.match_mode = Some(MatchMode::Segmented);
                }
            }
        };
        self.region = region;
        // set new Screen::region... fields
        self.screen.screen_region_width = region.2;
        self.screen.screen_region_height = region.3;
    }

    /// Searches for prepared template on screen.
    /// On windows only main monitor search is supported, while on linux, all monitors work
    /// more details in README
    #[allow(unused_variables)]
    pub fn find_image_on_screen(
        &mut self,
        precision: f32,
    ) -> Result<Option<Vec<(u32, u32, f64)>>, &'static str> {
        /// searches for image on screen and returns found locations in vector format
        let image: ImageBuffer<Luma<u8>, Vec<u8>> =
            self.screen.grab_screen_image_grayscale(&self.region)?;

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
        self.run_x_corr(image, precision)
    }

    // loops until image is found and returns found values, or until it times out
    pub fn loop_find_image_on_screen(
        &mut self,
        precision: f32,
        timeout: u64,
    ) -> Result<Option<Vec<(u32, u32, f64)>>, &'static str> {
        if (timeout == 0) & (!self.suppress_warnings) {
            eprintln!(
                "Warning: setting a timeout to 0 on a loop find image initiates an infinite loop"
            )
        }

        let timeout_start = std::time::Instant::now();
        let result = loop {
            if (timeout_start.elapsed().as_secs() > timeout) & (timeout > 0) {
                return Err("loop find image timed out. Could not find image");
            }
            let result = self.find_image_on_screen(precision);
            match result.clone()? {
                Some(_) => break result,
                None => continue,
            }
        };
        result
    }

    pub fn find_stored_image_on_screen(
        &mut self,
        precision: f32,
        alias: &String,
    ) -> Result<Option<Vec<(u32, u32, f64)>>, &'static str> {
        let (prepared_data, region) = self
            .prepared_data_stored
            .get(alias)
            .ok_or("No template stored with selected alias")?;
        // save to reset after finished
        let backup = BackupData {
            starting_data: self.prepared_data.clone(),
            starting_region: self.region.clone(),
            starting_match_mode: self.match_mode.clone(),
            starting_template_height: self.template_height.clone(),
            starting_template_width: self.template_width.clone(),
        };

        self.prepared_data = prepared_data.clone();
        self.screen.screen_region_width = region.2;
        self.screen.screen_region_height = region.3;
        self.region = *region;
        self.match_mode = match prepared_data {
            PreparedData::FFT(data) => {
                self.template_width = data.2;
                self.template_height = data.3;
                Some(MatchMode::FFT)
            }
            PreparedData::Segmented(data) => {
                self.template_width = data.2;
                self.template_height = data.3;
                Some(MatchMode::Segmented)
            }
            PreparedData::None => None,
        };
        let points = self.find_image_on_screen(precision)?;
        // reset to starting info
        backup.update_rustautogui(self);

        Ok(points)
    }

    // loops until stored image is found and returns found values, or until it times out
    pub fn loop_find_stored_image_on_screen(
        &mut self,
        precision: f32,
        timeout: u64,
        alias: &String,
    ) -> Result<Option<Vec<(u32, u32, f64)>>, &'static str> {
        if (timeout == 0) & (!self.suppress_warnings) {
            eprintln!(
                "Warning: setting a timeout to 0 on a loop find image initiates an infinite loop"
            )
        }
        let timeout_start = std::time::Instant::now();
        let result = loop {
            if (timeout_start.elapsed().as_secs() > timeout) & (timeout > 0) {
                return Err("loop find image timed out. Could not find image");
            }
            let result = self.find_stored_image_on_screen(precision, alias);
            match result.clone()? {
                Some(_) => break result,
                None => continue,
            }
        };
        result
    }

    pub fn find_stored_image_on_screen_and_move_mouse(
        &mut self,
        precision: f32,
        moving_time: f32,
        alias: &String,
    ) -> Result<Option<Vec<(u32, u32, f64)>>, &'static str> {
        let (prepared_data, region) = self
            .prepared_data_stored
            .get(alias)
            .ok_or("No template stored with selected alias")?;
        // save to reset after finished
        let backup = BackupData {
            starting_data: self.prepared_data.clone(),
            starting_region: self.region.clone(),
            starting_match_mode: self.match_mode.clone(),
            starting_template_height: self.template_height.clone(),
            starting_template_width: self.template_width.clone(),
        };

        self.prepared_data = prepared_data.clone();
        self.region = *region;
        self.screen.screen_region_width = region.2;
        self.screen.screen_region_height = region.3;
        self.match_mode = match prepared_data {
            PreparedData::FFT(data) => {
                self.template_width = data.2;
                self.template_height = data.3;

                Some(MatchMode::FFT)
            }
            PreparedData::Segmented(data) => {
                self.template_width = data.2;
                self.template_height = data.3;

                Some(MatchMode::Segmented)
            }
            PreparedData::None => return Err("No prepared data loaded"),
        };
        let found_points = self.find_image_on_screen_and_move_mouse(precision, moving_time);

        // reset to starting info
        backup.update_rustautogui(self);

        found_points
    }

    /// loops until stored image is found and moves mouse
    pub fn loop_find_stored_image_on_screen_and_move_mouse(
        &mut self,
        precision: f32,
        moving_time: f32,
        timeout: u64,
        alias: &String,
    ) -> Result<Option<Vec<(u32, u32, f64)>>, &'static str> {
        if (timeout == 0) & (!self.suppress_warnings) {
            eprintln!(
                "Warning: setting a timeout to 0 on a loop find image initiates an infinite loop"
            )
        }
        let timeout_start = std::time::Instant::now();
        let result = loop {
            if (timeout_start.elapsed().as_secs() > timeout) & (timeout > 0) {
                return Err("loop find image timed out. Could not find image");
            }
            let result =
                self.find_stored_image_on_screen_and_move_mouse(precision, moving_time, alias);
            match result.clone()? {
                Some(_) => break result,
                None => continue,
            }
        };
        result
    }

    /// executes find_image_on_screen and moves mouse to the middle of the image.
    pub fn find_image_on_screen_and_move_mouse(
        &mut self,
        precision: f32,
        moving_time: f32,
    ) -> Result<Option<Vec<(u32, u32, f64)>>, &'static str> {
        /// finds coordinates of the image on the screen and moves mouse to it. Returns None if no image found
        ///  Best used in loops
        let found_locations = self.find_image_on_screen(precision)?;

        let locations = match found_locations.clone() {
            Some(locations) => locations,
            None => return Ok(None),
        };

        let locations_adjusted: Vec<(u32, u32, f64)> = locations
            .clone()
            .into_iter()
            .map(|(mut x, mut y, corr)| {
                x = x + self.region.0 + (self.template_width / 2);
                y = y + self.region.1 + (self.template_height / 2);
                (x, y, corr)
            })
            .collect();

        let (target_x, target_y, _) = locations_adjusted[0];

        self.move_mouse_to_pos(target_x, target_y, moving_time)?;

        return Ok(Some(locations_adjusted));
    }

    // loops until image is found and returns found values, or until it times out
    pub fn loop_find_image_on_screen_and_move_mouse(
        &mut self,
        precision: f32,
        moving_time: f32,
        timeout: u64,
    ) -> Result<Option<Vec<(u32, u32, f64)>>, &'static str> {
        if (timeout == 0) & (!self.suppress_warnings) {
            eprintln!(
                "Warning: setting a timeout to 0 on a loop find image initiates an infinite loop"
            )
        }
        let timeout_start = std::time::Instant::now();
        let result = loop {
            if (timeout_start.elapsed().as_secs() > timeout) & (timeout > 0) {
                return Err("loop find image timed out. Could not find image");
            }
            let result = self.find_image_on_screen_and_move_mouse(precision, moving_time);
            match result.clone()? {
                Some(_) => break result,
                None => continue,
            }
        };
        result
    }

    fn run_x_corr(
        &mut self,
        image: ImageBuffer<Luma<u8>, Vec<u8>>,
        precision: f32,
    ) -> Result<Option<Vec<(u32, u32, f64)>>, &'static str> {
        let found_locations = match &self.prepared_data {
            PreparedData::FFT(data) => {
                let found_locations = normalized_x_corr::fft_ncc::fft_ncc(&image, &precision, data);
                found_locations
            },
            PreparedData::Segmented(data) => {
                let found_locations: Vec<(u32, u32, f64)> = normalized_x_corr::fast_segment_x_corr::fast_ncc_template_match(&image, &precision, &data, &self.debug, &self.suppress_warnings);
                found_locations
            },
            PreparedData::None => {
                return Err("No template chosen and no template data prepared. Please run load_and_prepare_template before searching image on screen ")
            },

        };

        if found_locations.len() > 0 {
            if self.debug {
                let corrected_found_location: (u32, u32, f64);
                let x = found_locations[0].0 as u32
                    + (self.template_width / 2) as u32
                    + self.region.0 as u32;
                let y = found_locations[0].1 as u32
                    + (self.template_height / 2) as u32
                    + self.region.1 as u32;
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

    //////////////////// Windows Mouse ////////////////////

    /// moves mouse to x, y pixel coordinate
    #[cfg(target_os = "windows")]
    pub fn move_mouse_to_pos(&self, x: u32, y: u32, moving_time: f32) -> Result<(), &'static str> {
        Mouse::move_mouse_to_pos(x as i32, y as i32, moving_time);
        if (x as i32 > self.screen.screen_width) | (y as i32 > self.screen.screen_height) {
            return Err("Out of screen boundaries");
        }
        Ok(())
    }

    /// moves mouse to x, y pixel coordinate
    #[cfg(target_os = "windows")]
    pub fn drag_mouse(&self, x: u32, y: u32, moving_time: f32) -> Result<(), &'static str> {
        if (x as i32 > self.screen.screen_width) | (y as i32 > self.screen.screen_height) {
            return Err("Out of screen boundaries");
        }
        Mouse::drag_mouse(x as i32, y as i32, moving_time);

        Ok(())
    }

    /// executes left mouse click
    #[cfg(target_os = "windows")]
    pub fn left_click(&self) -> Result<(), ()> {
        mouse::platform::Mouse::mouse_click(mouse::MouseClick::LEFT);
        Ok(())
    }

    /// executes middle mouse click
    #[cfg(target_os = "windows")]
    pub fn middle_click(&self) -> Result<(), ()> {
        mouse::platform::Mouse::mouse_click(mouse::MouseClick::MIDDLE);
        Ok(())
    }

    /// executes right mouse click
    #[cfg(target_os = "windows")]
    pub fn right_click(&self) -> Result<(), ()> {
        mouse::platform::Mouse::mouse_click(mouse::MouseClick::RIGHT);
        Ok(())
    }

    /// executes double left mouse click
    #[cfg(target_os = "windows")]
    pub fn double_click(&self) -> Result<(), ()> {
        mouse::platform::Mouse::mouse_click(mouse::MouseClick::LEFT);
        mouse::platform::Mouse::mouse_click(mouse::MouseClick::LEFT);
        Ok(())
    }

    #[cfg(target_os = "windows")]
    pub fn scroll_up(&self) -> Result<(), ()> {
        mouse::platform::Mouse::scroll(mouse::MouseScroll::UP);
        Ok(())
    }

    #[cfg(target_os = "windows")]
    pub fn scroll_down(&self) -> Result<(), ()> {
        mouse::platform::Mouse::scroll(mouse::MouseScroll::DOWN);
        Ok(())
    }

    #[cfg(target_os = "windows")]
    pub fn scroll_left(&self) -> Result<(), ()> {
        mouse::platform::Mouse::scroll(mouse::MouseScroll::LEFT);
        Ok(())
    }

    #[cfg(target_os = "windows")]
    pub fn scroll_right(&self) -> Result<(), ()> {
        mouse::platform::Mouse::scroll(mouse::MouseScroll::RIGHT);
        Ok(())
    }

    //////////////////// MacOS Mouse ////////////////////

    /// moves mouse to x, y pixel coordinate
    #[cfg(target_os = "macos")]
    pub fn move_mouse_to_pos(&self, x: u32, y: u32, moving_time: f32) -> Result<(), &'static str> {
        if (x as i32 > self.screen.screen_width) | (y as i32 > self.screen.screen_height) {
            return Err("Out of screen boundaries");
        }
        Mouse::move_mouse_to_pos(x as i32, y as i32, moving_time)?;
        Ok(())
    }

    #[cfg(target_os = "macos")]
    pub fn drag_mouse(&self, x: u32, y: u32, moving_time: f32) -> Result<(), &'static str> {
        if moving_time < 0.5 {
            if !self.suppress_warnings {
                eprintln!("WARNING:Small moving time values may cause issues on mouse drag");
            }
        }
        if (x as i32 > self.screen.screen_width) | (y as i32 > self.screen.screen_height) {
            return Err("Out of screen boundaries");
        }
        Mouse::drag_mouse(x as i32, y as i32, moving_time)?;

        Ok(())
    }

    /// executes left mouse click
    #[cfg(target_os = "macos")]
    pub fn left_click(&self) -> Result<(), &'static str> {
        mouse::platform::Mouse::mouse_click(mouse::MouseClick::LEFT)?;
        Ok(())
    }

    /// executes right mouse click
    #[cfg(target_os = "macos")]
    pub fn right_click(&self) -> Result<(), &'static str> {
        mouse::platform::Mouse::mouse_click(mouse::MouseClick::RIGHT)?;
        Ok(())
    }

    /// executes middle mouse click
    #[cfg(target_os = "macos")]
    pub fn middle_click(&self) -> Result<(), &'static str> {
        mouse::platform::Mouse::mouse_click(mouse::MouseClick::MIDDLE)?;
        Ok(())
    }

    /// executes double mouse click
    #[cfg(target_os = "macos")]
    pub fn double_click(&self) -> Result<(), &'static str> {
        mouse::platform::Mouse::double_click()?;
        Ok(())
    }

    #[cfg(target_os = "macos")]
    pub fn scroll_up(&self) -> Result<(), &'static str> {
        mouse::platform::Mouse::scroll(mouse::MouseScroll::UP)?;
        Ok(())
    }

    #[cfg(target_os = "macos")]
    pub fn scroll_down(&self) -> Result<(), &'static str> {
        mouse::platform::Mouse::scroll(mouse::MouseScroll::DOWN)?;
        Ok(())
    }

    #[cfg(target_os = "macos")]
    pub fn scroll_left(&self) -> Result<(), &'static str> {
        mouse::platform::Mouse::scroll(mouse::MouseScroll::LEFT)?;
        Ok(())
    }

    #[cfg(target_os = "macos")]
    pub fn scroll_right(&self) -> Result<(), &'static str> {
        mouse::platform::Mouse::scroll(mouse::MouseScroll::RIGHT)?;
        Ok(())
    }

    //////////////////// Linux Mouse ////////////////////

    /// moves mouse to x, y pixel coordinate
    #[cfg(target_os = "linux")]
    pub fn move_mouse_to_pos(&self, x: u32, y: u32, moving_time: f32) -> Result<(), &'static str> {
        if (x as i32 > self.screen.screen_width) | (y as i32 > self.screen.screen_height) {
            return Err("Out of screen boundaries");
        }
        self.mouse
            .move_mouse_to_pos(x as i32, y as i32, moving_time)
    }

    /// moves mouse to x, y pixel coordinate
    #[cfg(target_os = "linux")]
    pub fn drag_mouse(&self, x: u32, y: u32, moving_time: f32) -> Result<(), &'static str> {
        if (x as i32 > self.screen.screen_width) | (y as i32 > self.screen.screen_height) {
            return Err("Out of screen boundaries");
        }
        if moving_time < 0.5 {
            if !self.suppress_warnings {
                eprintln!("WARNING:Small moving time values may cause issues on mouse drag");
            }
        }
        self.mouse.drag_mouse(x as i32, y as i32, moving_time)
    }

    /// executes left mouse click
    #[cfg(target_os = "linux")]
    pub fn left_click(&self) -> Result<(), &'static str> {
        self.mouse.mouse_click(mouse::MouseClick::LEFT)
    }

    /// executes right mouse click
    #[cfg(target_os = "linux")]
    pub fn right_click(&self) -> Result<(), &'static str> {
        self.mouse.mouse_click(mouse::MouseClick::RIGHT)
    }

    /// executes middle mouse click
    #[cfg(target_os = "linux")]
    pub fn middle_click(&self) -> Result<(), &'static str> {
        self.mouse.mouse_click(mouse::MouseClick::MIDDLE)
    }

    /// executes double left mouse click
    #[cfg(target_os = "linux")]
    pub fn double_click(&self) -> Result<(), &'static str> {
        self.mouse.mouse_click(mouse::MouseClick::LEFT)?;
        self.mouse.mouse_click(mouse::MouseClick::LEFT)
    }

    #[cfg(target_os = "linux")]
    pub fn scroll_up(&self) -> Result<(), ()> {
        self.mouse.scroll(mouse::MouseScroll::UP);
        Ok(())
    }

    #[cfg(target_os = "linux")]
    pub fn scroll_down(&self) -> Result<(), ()> {
        self.mouse.scroll(mouse::MouseScroll::DOWN);
        Ok(())
    }

    #[cfg(target_os = "linux")]
    pub fn scroll_left(&self) -> Result<(), ()> {
        self.mouse.scroll(mouse::MouseScroll::LEFT);
        Ok(())
    }

    #[cfg(target_os = "linux")]
    pub fn scroll_right(&self) -> Result<(), ()> {
        self.mouse.scroll(mouse::MouseScroll::RIGHT);
        Ok(())
    }

    //////////////////// Keyboard ////////////////////

    /// accepts string and mimics keyboard key presses for each character in string
    pub fn keyboard_input(&self, input: &str) -> Result<(), &'static str> {
        let input_string = String::from(input);
        for letter in input_string.chars() {
            self.keyboard.send_char(&letter)?;
        }
        Ok(())
    }

    /// executes keyboard command like "return" or "escape"
    pub fn keyboard_command(&self, input: &str) -> Result<(), &'static str> {
        let input_string = String::from(input);
        // return automatically the result of send_command function
        self.keyboard.send_command(&input_string)
    }

    pub fn keyboard_multi_key(
        &self,
        input1: &str,
        input2: &str,
        input3: Option<&str>,
    ) -> Result<(), &'static str> {
        let input3 = match input3 {
            Some(x) => Some(String::from(x)),
            None => None,
        };
        // send automatically result of function
        self.keyboard
            .send_multi_key(&String::from(input1), &String::from(input2), input3)
    }

    /// DEPRECATED
    pub fn load_and_prepare_template(
        &mut self,
        template_path: &str,
        region: Option<(u32, u32, u32, u32)>,
        match_mode: MatchMode,
        max_segments: Option<u32>,
    ) -> Result<(), String> {
        if !self.suppress_warnings {
            eprintln!("Warning: load_and_prepare_template will be deprecated. Consider using prepare_template_from_file");
        }
        let template: ImageBuffer<Luma<u8>, Vec<u8>> = imgtools::load_image_bw(template_path)?;
        self.prepare_template_picture_bw(template, region, match_mode, max_segments, None)
    }
}

#[cfg(any(target_os = "linux", target_os = "windows"))]
impl Drop for RustAutoGui {
    fn drop(&mut self) {
        self.screen.destroy();
    }
}
