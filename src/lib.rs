#![allow(unused_doc_comments)]
use image::{ImageBuffer, Luma};
use rustfft::num_complex::Complex;
pub mod imgtools;
pub mod normalized_x_corr;


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


pub mod keyboard;
pub mod mouse;
pub mod screen;







pub enum PreparedData {
    Segmented((Vec<(u32, u32, u32, u32, f32)>, Vec<(u32, u32, u32, u32, f32)>, u32, u32, f32, f32, f32, f32, f32, f32)),
    FFT((Vec<Complex<f32>>, f32, u32, u32, u32)),
    None
}



#[derive(PartialEq)]
pub enum MatchMode {
    Segmented,
    FFT,
}


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
    #[cfg(target_os = "windows")]
    pub fn new(debug:bool) -> Self{
        let screen = Screen::new();
        let keyboard = Keyboard::new();
        
        let mouse_struct: Mouse = Mouse::new(None, None);
        
        
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
    #[cfg(target_os = "linux")]
    pub fn new(debug:bool) -> Self{
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

    pub fn load_and_prepare_template(&mut self, template_path: &str, region:Option<(u32,u32,u32,u32)>, match_mode:MatchMode, max_segments: &Option<u32>) {
        let template = imgtools::load_image_bw(template_path);
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
                panic!("Segmented correlation has not been implemented yet");
            }
        };
        self.prepared_data = template_data;

    }
    
    #[allow(unused_variables)]
    pub fn change_prepared_settings (&mut self, region:Option<(u32,u32,u32,u32)>, match_mode:MatchMode, max_segments: &Option<u32>) {
        let template = self.template.clone();
        let template = match template {
            Some(image) => image,
            None => panic!("No template loaded! Please use load_and_prepare_template method"),
        };


        let region = match region {
            Some(region_tuple) => {
                region_tuple
            },
            None => {   
                let (screen_width, screen_height) = self.screen.dimension();
                (0, 0, screen_width as u32, screen_height as u32)
            }
        };
        

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
                panic!("Segmented correlation has not been implemented yet")
            }
        };
        self.region = region;
        self.screen.screen_region_width = region.2;
        self.screen.screen_region_height = region.3;
        

    }

    pub fn change_debug_state(&mut self, state:bool) {
        self.debug = state;
    }
    #[allow(unused_variables)]
    pub fn find_image_on_screen(&mut self, precision: f32) -> Option<Vec<(u32, u32, f64)>>{
        /// searches for image on screen and returns found locations in vector format
        let image =self.screen.grab_screen_image_grayscale(&self.region);
        if self.debug {
            image.save("debug/screen_capture.png").unwrap();
        };

        let found_locations = match &self.prepared_data {
            PreparedData::FFT(data) => {
                
                let found_locations = normalized_x_corr::fft_ncc::fft_ncc(&image, &precision, data);
                found_locations
            },
            PreparedData::Segmented(data) => {
                panic!("Segmented correlation is not implemented yet");
            },
            PreparedData::None => {
                panic!("No template data chosen");
            },
        };
        if found_locations.len() > 0 {
            return Some(found_locations);
        } else {
            return None;
        };
    }

    pub fn save_screenshot(&mut self, path:&str) {
        self.screen.grab_screenshot(path);
    }


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


    #[cfg(target_os = "windows")]
    pub fn move_mouse_to_pos(&self, x: i32, y: i32, moving_time: f32) {
        Mouse::move_mouse_to_pos(x, y, moving_time);
    }


    #[cfg(target_os = "linux")]
    pub fn move_mouse_to_pos(&self, x: i32, y: i32, moving_time:f32) {
        self.mouse.move_mouse_to_pos(x , y, moving_time);
    }



    #[cfg(target_os = "windows")]
    pub fn left_click(&self) {
        mouse::windows::Mouse::mouse_click(mouse::Mouseclick::LEFT);
    }

    #[cfg(target_os = "windows")]
    pub fn right_click(&self) {
        mouse::windows::Mouse::mouse_click(mouse::Mouseclick::RIGHT);
    }

    #[cfg(target_os = "windows")]
    pub fn middle_click(&self) {
        mouse::windows::Mouse::mouse_click(mouse::Mouseclick::MIDDLE);
    }

    #[cfg(target_os = "linux")]
    pub fn left_click(&self) {
        self.mouse.mouse_click(mouse::Mouseclick::LEFT);
    }

    #[cfg(target_os = "linux")]
    pub fn right_click(&self) {
        self.mouse.mouse_click(mouse::Mouseclick::RIGHT);
    }

    #[cfg(target_os = "linux")]
    pub fn middle_click(&self) {
        self.mouse.mouse_click(mouse::Mouseclick::MIDDLE);
    }
    
    

    pub fn keyboard_input(&self,input:&str, shifted:&bool) {
        let input_string = String::from(input);
        for letter in input_string.chars() {
            self.keyboard.send_char(&letter, shifted);
        }
    }


    pub fn keyboard_command(&self, input:&str) {
        let input_string = String::from(input);
        self.keyboard.send_command(&input_string);
    }
    
}


impl Drop for RustAutoGui {
    fn drop(&mut self) {
        self.screen.destroy();
    }
}