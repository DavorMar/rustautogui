
use x11::xlib::*;
use std::time::Instant;
use x11::xtest::*;
use super::Mouseclick;

pub struct Mouse{
    screen: *mut _XDisplay,
    root_window: u64,
}

impl Mouse {
    pub fn new(screen: Option<*mut _XDisplay>, root_window: Option<u64>) -> Self {
        let screen = screen.unwrap();
        let root_window = root_window.unwrap();
        Self {screen:screen, root_window: root_window}
    }

    


    pub fn move_mouse_to_pos(&self, x:i32, y:i32, moving_time:f32) {
        
        unsafe {
            if moving_time == 0.0 {
                XWarpPointer(self.screen, 0, self.root_window, 0, 0, 0, 0, x, y);
                XFlush(self.screen); 
                return  
            }
        }
        let start = Instant::now();
        let start_location = self.get_mouse_position();
        let distance_x = x - start_location.0;
        let distance_y= y -start_location.1;
        loop {
               
            let duration = start.elapsed();
            
            let time_passed_percentage=  duration.as_secs_f32() / moving_time;
            if time_passed_percentage > 10.0 {
                 continue
            }
            let new_x =  start_location.0 as f32 + (time_passed_percentage  * distance_x as f32);     
            let new_y =  start_location.1 as f32 + (time_passed_percentage * distance_y as f32) ;
            println!("{time_passed_percentage}");
            unsafe {
                 if time_passed_percentage >= 1.0{
                        XWarpPointer(self.screen, 0, self.root_window, 0, 0, 0, 0, x, y);
                        XFlush(self.screen); 
                      break
                 }  else {
                        XWarpPointer(self.screen, 0, self.root_window, 0, 0, 0, 0, new_x as i32, new_y as i32);
                        XFlush(self.screen); 
                 }             
                 
            }
       }
    }

    pub fn get_mouse_position(&self) -> (i32,i32) {
        unsafe {
            let mut root_return = 0;
            let mut child_return = 0;
            let mut root_x = 0;
            let mut root_y = 0;
            let mut win_x = 0;
            let mut win_y = 0;
            let mut mask_return = 0;

            let status = XQueryPointer(
                self.screen,
                self.root_window,
                &mut root_return,
                &mut child_return,
                &mut root_x,
                &mut root_y,
                &mut win_x,
                &mut win_y,
                &mut mask_return,
            );

            if status == 0 {
                panic!("Unable to query pointer position");
            }

            (root_x, root_y)
        }
    }


    pub fn mouse_click(&self,  button: Mouseclick) {
        let button =match button {
            Mouseclick::LEFT => 1,
            Mouseclick::MIDDLE => 2,
            Mouseclick::RIGHT => 3,
        };
        // Check if the XTest extension is available
        let mut event_base = 0;
        let mut error_base = 0;
        unsafe {
            if XTestQueryExtension(self.screen, &mut event_base, &mut error_base, &mut event_base, &mut error_base) == 0 {
                eprintln!("XTest extension not available");
                return;
            }
        
            // Press the mouse button
            XTestFakeButtonEvent(self.screen, button, 1, CurrentTime);
            XFlush(self.screen);
        
            // Release the mouse button
            XTestFakeButtonEvent(self.screen, button, 0, CurrentTime);
            XFlush(self.screen);
        }
        
    }
}