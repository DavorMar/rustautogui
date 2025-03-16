#![allow(unused_doc_comments, unused_imports)]
pub mod imgtools;
pub mod normalized_x_corr;

use image::{
    imageops::{resize, FilterType::Nearest},
    ImageBuffer, Luma,
};
use rustfft::num_complex::Complex;

use std::{env, fs, path::Path};

#[cfg(target_os = "windows")]
pub use crate::{
    keyboard::windows::Keyboard,
    mouse::windows::Mouse,
    screen::windows::Screen
};

#[cfg(target_os = "linux")]
pub use crate::{
    keyboard::linux::Keyboard,
    mouse::linux::Mouse, 
    screen::linux::Screen
};

#[cfg(target_os = "macos")]
pub use crate::{
    keyboard::macos::Keyboard, 
    mouse::macos::Mouse, 
    screen::macos::Screen
};

pub mod keyboard;
pub mod mouse;
pub mod screen;

/// Struct of prepared data for each correlation method used
/// Segmented consists of two image vectors and associated mean value, sum of squared deviations, sizes
/// FFT vector consists of template vector converted to frequency domain and conjugated, sum squared deviations, size and padded size
pub enum PreparedData {
    Segmented(
        (
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
        ),
    ),
    FFT((Vec<Complex<f32>>, f32, u32, u32, u32)),
    None,
}

/// Matchmode Segmented correlation and Fourier transform correlation
#[derive(PartialEq)]
pub enum MatchMode {
    Segmented,
    FFT,
}

/// Main struct for Rustautogui
/// Struct gets assigned keyboard, mouse and struct to it implemented functions execute commands from each of assigned substructs
/// executes also correlation algorithms when doing find_image_on_screen
#[allow(dead_code)]
pub struct RustAutoGui {
    // most of the fields are set up in load_and_prepare_template method
    template: Option<ImageBuffer<Luma<u8>, Vec<u8>>>,
    prepared_data: PreparedData,
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

    /// Loads template image from provided path and sets all the fields across structs as needed. Depending on match_mode, different template
    /// preparation process is executed. When using FFT, region is also important for zero-pad calculation
    /// Loading and preparing template is a necessary process before calling find_image_on_screen function
    ///
    /// creates vector of data stored under PreparedData enumerator, and stored inside struct field self.prepared_data
    pub fn load_and_prepare_template(
        &mut self,
        template_path: &str,
        region: Option<(u32, u32, u32, u32)>,
        match_mode: MatchMode,
        max_segments: &Option<u32>,
    ) -> Result<(), String> {
        #[allow(unused_mut)] // allowed because its needed in macos code below
        let mut template = imgtools::load_image_bw(template_path)?;
        #[cfg(target_os = "macos")] 
        //resize and adjust if retina screen is used
        {
            template = resize(
                &template,
                template.width() / self.screen.scaling_factor_x as u32,
                template.height() / self.screen.scaling_factor_y as u32,
                Nearest,
            );
        }
        let (template_width, template_height) = template.dimensions();
        // update struct values
        self.template_width = template_width;
        self.template_height = template_height;
        // convert to option and store in struct
        self.template = Some(template.clone());
        self.max_segments = *max_segments;
        let region = match region {
            Some(region_tuple) => region_tuple,
            None => {
                let (screen_width, screen_height) = self.screen.dimension();
                (0, 0, screen_width as u32, screen_height as u32)
            }
        };
        self.region = region;
        // update screen struct
        self.screen.screen_region_width = region.2;
        self.screen.screen_region_height = region.3;
        self.check_if_region_out_of_bound()?;
        // do the rest of preparation calculations depending on the matchmode
        // FFT pads the image, does fourier transformations,
        // calculates conjugate and inverses transformation on template
        // Segmented creates vector of picture segments with coordinates, dimensions and average pixel value
        let template_data = match match_mode {
            MatchMode::FFT => {
                let prepared_data =
                    PreparedData::FFT(normalized_x_corr::fft_ncc::prepare_template_picture(
                        &template, &region.2, &region.3,
                    ));
                self.match_mode = Some(MatchMode::FFT);
                prepared_data
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
                let prepared_data = PreparedData::Segmented(prepared_data);
                self.match_mode = Some(MatchMode::Segmented);
                prepared_data
            }
        };
        self.prepared_data = template_data;
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
        max_segments: &Option<u32>,
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
                    && self.max_segments == *max_segments
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
                            &None,
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

    /// changes debug mode
    pub fn change_debug_state(&mut self, state: bool) {
        self.debug = state;
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
        let image = self.screen.grab_screen_image_grayscale(&self.region)?;

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

        let found_locations = match &self.prepared_data {
            PreparedData::FFT(data) => {
                
                let found_locations = normalized_x_corr::fft_ncc::fft_ncc(&image, &precision, data);
                found_locations
            },
            PreparedData::Segmented(data) => {
                let found_locations: Vec<(u32, u32, f64)> = normalized_x_corr::fast_segment_x_corr::fast_ncc_template_match(&image, &precision, data, &self.debug, "", &self.suppress_warnings);
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
            let empty_vec: Vec<(u32, u32, f64)> = vec![];
            return Ok(None);
        };
    }

    /// saves screenshot and saves it at provided path
    pub fn save_screenshot(&mut self, path: &str) -> Result<(), String> {
        self.screen.grab_screenshot(path)?;
        Ok(())
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

        let locations_adjusted:Vec<(u32,u32,f64)> = locations.clone().into_iter().map(
            |(mut x, mut y, corr)| {
                x = x + self.region.0 + (self.template_width / 2);
                y = y + self.region.1 + (self.template_height / 2);
                (x, y, corr)
            }
        ).collect();

        let (target_x, target_y, _) = locations_adjusted[0];


        self.move_mouse_to_pos(target_x, target_y, moving_time)?;

        return Ok(Some(locations_adjusted));
    }

    pub fn get_screen_size(&mut self) -> (i32, i32) {
        self.screen.dimension()
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
        if self.debug {
            let (x, y) = Mouse::get_mouse_position()?;
            println!("Mouse moved to position {x}, {y}");
        }
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
            .move_mouse_to_pos(x as i32, y as i32, moving_time)?;
        Ok(())
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
        self.mouse.drag_mouse(x as i32, y as i32, moving_time)?;
        Ok(())
    }

    /// executes left mouse click
    #[cfg(target_os = "linux")]
    pub fn left_click(&self) -> Result<(), &'static str> {
        self.mouse.mouse_click(mouse::MouseClick::LEFT)?;
        Ok(())
    }

    /// executes right mouse click
    #[cfg(target_os = "linux")]
    pub fn right_click(&self) -> Result<(), &'static str> {
        self.mouse.mouse_click(mouse::MouseClick::RIGHT)?;
        Ok(())
    }

    /// executes middle mouse click
    #[cfg(target_os = "linux")]
    pub fn middle_click(&self) -> Result<(), &'static str> {
        self.mouse.mouse_click(mouse::MouseClick::MIDDLE)?;
        Ok(())
    }

    /// executes double left mouse click
    #[cfg(target_os = "linux")]
    pub fn double_click(&self) -> Result<(), &'static str> {
        self.mouse.mouse_click(mouse::MouseClick::LEFT)?;
        self.mouse.mouse_click(mouse::MouseClick::LEFT)?;
        Ok(())
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
}

#[cfg(any(target_os = "linux", target_os = "windows"))]
impl Drop for RustAutoGui {
    fn drop(&mut self) {
        self.screen.destroy();
    }
}
