#![allow(unused_doc_comments)]
use image::{imageops::{resize, FilterType::Nearest}, ImageBuffer, Luma};
use rustfft::num_complex::Complex;
pub mod imgtools;
pub mod normalized_x_corr;
use std::fs;
use std::path::Path;

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
    Segmented((Vec<(u32, u32, u32, u32, f32)>, Vec<(u32, u32, u32, u32, f32)>, u32, u32, f32, f32, f32, f32, f32, f32)),
    FFT((Vec<Complex<f32>>, f32, u32, u32, u32)),
    None
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
    template: Option<ImageBuffer<Luma<u8>, Vec<u8>>>,
    prepared_data: PreparedData,
    debug:bool,
    template_height:u32,
    template_width:u32,
    keyboard:Keyboard,
    mouse:Mouse,
    screen:Screen,
    match_mode: Option<MatchMode>,
    max_segments: Option<u32>,
    region: (u32,u32,u32,u32),
}
impl RustAutoGui {
    /// initiation of screen, keyboard and mouse that are assigned to new rustautogui struct.
    /// all the other struct fields are initiated as 0 or None
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    pub fn new(debug:bool) -> Self{
        // initiation of screen, keyboard and mouse
        // on windows there is no need to share display pointer accross other structs
        let screen = Screen::new();
        let keyboard = Keyboard::new();
        let mouse_struct: Mouse = Mouse::new();
        Self{
            template:None, 
            prepared_data:PreparedData::None,
            debug:debug,
            template_width:0,
            template_height:0,
            keyboard:keyboard,
            mouse:mouse_struct,
            screen:screen,
            match_mode:None,
            max_segments: None,
            region:(0,0,0,0)
        }
    }

    /// initiation of screen, keyboard and mouse that are assigned to new rustautogui struct.
    /// all the other struct fields are initiated as 0 or None
    #[cfg(target_os = "linux")]
    pub fn new(debug:bool) -> Self{
        // on linux, screen display pointer is shared to keyboard and mouse 
        // x11 works like that and initiation of individual display objects
        // under each struct wouldnt be preferable
        let screen = Screen::new();
        let keyboard = Keyboard::new(screen.display);
        let mouse_struct: Mouse = Mouse::new(Some(screen.display), Some(screen.root_window));
        Self{
            template:None, 
            prepared_data:PreparedData::None,
            debug:debug,
            template_width:0,
            template_height:0,
            keyboard:keyboard,
            mouse:mouse_struct,
            screen:screen,
            match_mode:None,
            max_segments: None,
            region:(0,0,0,0)
        }
    }


    /// Loads template image from provided path and sets all the fields across structs as needed. Depending on match_mode, different template
    /// preparation process is executed. When using FFT, region is also important for zero-pad calculation 
    /// Loading and preparing template is a necessary process before calling find_image_on_screen function
    /// 
    /// creates vector of data stored under PreparedData enumerator, and stored inside struct field self.prepared_data
    pub fn load_and_prepare_template(&mut self, template_path: &str, region:Option<(u32,u32,u32,u32)>, match_mode:MatchMode, max_segments: &Option<u32>) {
        let mut template = imgtools::load_image_bw(template_path);
        #[cfg(target_os = "macos")]
        {
            template = resize(&template, template.width()/ self.screen.scaling_factor_x as u32, template.height()/ self.screen.scaling_factor_y as u32, Nearest);
        }
        let (template_width, template_height) = template.dimensions();
        self.template_width = template_width;
        self.template_height = template_height;
        self.template = Some(template.clone());
        self.max_segments = *max_segments;
        let region = match region {
            Some(region_tuple) => {
                region_tuple
            },
            None => {   
                let (screen_width, screen_height) = self.screen.dimension();
                (0, 0, screen_width as u32, screen_height as u32)
            }
        };
        self.region = region;
        self.screen.screen_region_width = region.2;
        self.screen.screen_region_height = region.3;

        let template_data = match match_mode {
            MatchMode::FFT => {
                let prepared_data = PreparedData::FFT(normalized_x_corr::fft_ncc::prepare_template_picture(&template, &region.2, &region.3));
                self.match_mode = Some(MatchMode::FFT);
                prepared_data
            },
            MatchMode::Segmented => {
                let prepared_data = PreparedData::Segmented(normalized_x_corr::fast_segment_x_corr::prepare_template_picture(&template, max_segments, &self.debug));
                self.match_mode = Some(MatchMode::Segmented);
                prepared_data
            }
        };
        self.prepared_data = template_data;

    }
    
    /// change certain settings for prepared template, like region, match_mode or max_segments. If MatchMode is not changed, whole template
    /// recalculation may still be needed if certain other parameters are changed, depending on current MatchMode. 
    /// For FFT, changing region starts complete recalculation again, because of change in zero pad image. While changing
    /// max_segments calls recalculation of template for Segmented matchmode
    #[allow(unused_variables)]
    pub fn change_prepared_settings (&mut self, region:Option<(u32,u32,u32,u32)>, match_mode:MatchMode, max_segments: &Option<u32>) {
        let template = self.template.clone();
        let template = match template {
            Some(image) => image,
            None => panic!("No template loaded! Please use load_and_prepare_template method"),
        };

        // unpack region , or set default if none
        let region = match region {
            Some(region_tuple) => {
                region_tuple
            },
            None => {   
                let (screen_width, screen_height) = self.screen.dimension();
                (0, 0, screen_width as u32, screen_height as u32)
            }
        };
        
        // check if template recalculation is needed
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
                    let prepared_data = PreparedData::FFT(normalized_x_corr::fft_ncc::prepare_template_picture(&template, &region.2, &region.3));
                    self.prepared_data = prepared_data;
                    self.match_mode = Some(MatchMode::FFT);
                }
                
            
            },
            MatchMode::Segmented => {
                // no need to recalculate if max segments havent changed or if match mode has not changed
                if self.match_mode == Some(MatchMode::Segmented) && self.max_segments == *max_segments {
                    if self.debug {
                        println!("Keeping same template data");
                    }
                } else {
                    if self.debug {
                        println!("Recalculating template data");
                    }
                    let prepared_data = PreparedData::Segmented(normalized_x_corr::fast_segment_x_corr::prepare_template_picture(&template, &None, &self.debug));
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
    pub fn change_debug_state(&mut self, state:bool) {
        self.debug = state;
    }

    /// Searches for prepared template on screen.
    /// On windows only main monitor search is supported, while on linux, all monitors work
    /// more details in README
    #[allow(unused_variables)]
    pub fn find_image_on_screen(&mut self, precision: f32) -> Option<Vec<(u32, u32, f64)>>{
        /// searches for image on screen and returns found locations in vector format
        let image =self.screen.grab_screen_image_grayscale(&self.region);
        
        if self.debug {
            let debug_path = Path::new("debug");
            if !debug_path.exists() {
                fs::create_dir_all(debug_path).expect("Failed to create 'debug' directory. Please create it manualy in the root folder");
                println!("Created a debug folder in your root for saving segmented template images")
            }
            let error_catch = image.save("debug/screen_capture.png");
            match error_catch {
                Ok(_) => (),
                Err(_) => println!("Create a 'debug' folder in your root folder to save images"),
            }
        };

        let found_locations = match &self.prepared_data {
            PreparedData::FFT(data) => {
                
                let found_locations = normalized_x_corr::fft_ncc::fft_ncc(&image, &precision, data);
                found_locations
            },
            PreparedData::Segmented(data) => {
                let found_locations = normalized_x_corr::fast_segment_x_corr::fast_ncc_template_match(&image, &precision, data, &self.debug, "");
                found_locations
            },
            PreparedData::None => {
                panic!("No template data chosen");
            },
        };
        
        
        
        if found_locations.len() > 0 {
            if self.debug {
                let corrected_found_location:(u32,u32,f64);
                let x = found_locations[0].0 as u32 + (self.template_width /2) as u32 + self.region.0 as u32;
                let y = found_locations[0].1 as u32 + (self.template_height/2) as u32 + self.region.1 as u32;
                let corr = found_locations[0].2;
                corrected_found_location = (x,y,corr);
                
                println!("Location found at x: {}, y {}, corr {} ",corrected_found_location.0, corrected_found_location.1, corrected_found_location.2)
            }
            return Some(found_locations);
            
        } else {
            return None;
        };
    }


    /// saves screenshot and saves it at provided path
    pub fn save_screenshot(&mut self, path:&str) {
        self.screen.grab_screenshot(path);
    }

    /// executes find_image_on_screen and moves mouse to the middle of the image. 
    pub fn find_image_on_screen_and_move_mouse(&mut self, precision: f32, moving_time:f32) -> Option<Vec<(u32, u32, f64)>> {
        /// finds coordinates of the image on the screen and moves mouse to it. Returns None if no image found
        ///  Best used in loops
        let found_locations: Option<Vec<(u32, u32, f64)>> = self.find_image_on_screen(precision);
        let locations = match found_locations.clone() {
            Some(locations) => {locations},
            None => return None
        };
        let top_location = locations[0];
        let x = top_location.0 as i32 + (self.template_width /2) as i32;
        let y = top_location.1 as i32 + (self.template_height/2) as i32;
        self.move_mouse_to_pos(x + self.region.0 as i32,y+self.region.1 as i32, moving_time);
        
        return found_locations;
    }

    /// moves mouse to x, y pixel coordinate
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    pub fn move_mouse_to_pos(&self, x: i32, y: i32, moving_time: f32) {
        Mouse::move_mouse_to_pos(x, y, moving_time);
        if self.debug {
            let (x,y) = Mouse::get_mouse_position();
            println!("Mouse moved to position {x}, {y}");    
        }
        
    }

    /// moves mouse to x, y pixel coordinate
    #[cfg(target_os = "linux")]
    pub fn move_mouse_to_pos(&self, x: i32, y: i32, moving_time:f32) {
        self.mouse.move_mouse_to_pos(x , y, moving_time);
    }
 

    /// executes left mouse click 
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    pub fn left_click(&self) {
        mouse::platform::Mouse::mouse_click(mouse::MouseClick::LEFT);
    }

    /// executes right mouse click 
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    pub fn right_click(&self) {
        mouse::platform::Mouse::mouse_click(mouse::MouseClick::RIGHT);
    }

    /// executes middle mouse click
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    pub fn middle_click(&self) {
        mouse::platform::Mouse::mouse_click(mouse::MouseClick::MIDDLE);
    }

    /// executes double mouse click
    #[cfg(target_os = "macos")]
    pub fn double_click(&self) {
        mouse::platform::Mouse::double_click();
        
    }

    /// executes double left mouse click
    #[cfg(target_os = "windows")]
    pub fn double_click(&self) {
        mouse::platform::Mouse::mouse_click(mouse::MouseClick::LEFT);
        mouse::platform::Mouse::mouse_click(mouse::MouseClick::LEFT);
    }


    /// executes left mouse click 
    #[cfg(target_os = "linux")]
    pub fn left_click(&self) {
        self.mouse.mouse_click(mouse::MouseClick::LEFT);
    }

    /// executes right mouse click
    #[cfg(target_os = "linux")]
    pub fn right_click(&self) {
        self.mouse.mouse_click(mouse::MouseClick::RIGHT);
    }

    /// executes middle mouse click
    #[cfg(target_os = "linux")]
    pub fn middle_click(&self) {
        self.mouse.mouse_click(mouse::MouseClick::MIDDLE);
    }
    
    /// executes double left mouse click
    #[cfg(target_os = "linux")]
    pub fn double_click(&self) {
        self.mouse.mouse_click(mouse::MouseClick::LEFT);
        self.mouse.mouse_click(mouse::MouseClick::LEFT);
    }

    #[cfg(any(target_os = "windows", target_os = "macos"))]
    pub fn scroll_up(&self) {
        mouse::platform::Mouse::scroll(mouse::MouseScroll::UP);
    }
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    pub fn scroll_down(&self) {
        mouse::platform::Mouse::scroll(mouse::MouseScroll::DOWN);
    }

    #[cfg(target_os = "linux")]
    pub fn scroll_up(&self) {
        self.mouse.scroll(mouse::MouseScroll::UP);
    }
    #[cfg(target_os = "linux")]
    pub fn scroll_down(&self) {
        self.mouse.scroll(mouse::MouseScroll::DOWN);
    }
    /// accepts string and mimics keyboard key presses for each character in string
    pub fn keyboard_input(&self,input:&str, shifted:&bool) {
        let input_string = String::from(input);
        for letter in input_string.chars() {
            self.keyboard.send_char(&letter, shifted);
        }
    }

    /// executes keyboard command like "return" or "escape"
    pub fn keyboard_command(&self, input:&str) {
        let input_string = String::from(input);
        self.keyboard.send_command(&input_string);
    }

    pub fn keyboard_multi_key(&self, input1:&str, input2:&str, input3:Option<&str>) {
        let input3 = match input3 {
            Some(x) => {
                Some(String::from(x))
            },
            None => None
        };
        self.keyboard.send_multi_key(&String::from(input1), &String::from(input2), input3);
        
    }



    
    
}

#[cfg(any(target_os = "linux", target_os = "windows"))]
impl Drop for RustAutoGui {
    fn drop(&mut self) {
        self.screen.destroy();
    }
}
