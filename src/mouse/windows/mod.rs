use winapi::shared::windef::POINT;
use winapi::um::winuser::{SetCursorPos, SendInput, INPUT, INPUT_MOUSE,
     MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP,
     MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP,
     MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP,
};
use std::time::Instant;
use std::mem::{zeroed, size_of};
use x11::xlib::_XDisplay;
use crate::mouse::Mouseclick;


pub struct Mouse {

}
impl Mouse {
     #[allow(unused_variables)]
     pub fn new() -> Mouse {
          Mouse{}
     }


     /// moves mouse to x, y pixel coordinate on screen 
     pub fn move_mouse_to_pos( x:i32,y:i32, moving_time: f32){
          // if no moving time, then instant move is executed
          unsafe {

               if moving_time == 0.0 {
                    SetCursorPos(x, y);
                    return
               }
          };
          // if moving time is included, loop is executed that moves step by step
          let start = Instant::now();
          let start_location = Mouse::get_mouse_position();
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
               
               unsafe {
                    if time_passed_percentage >= 1.0{
                         SetCursorPos(x as i32, y as i32);
                         break
                    }  else {
                         SetCursorPos(new_x as i32, new_y as i32);
                    }             
                    
               }
          }
               
          }
          

     /// returns x, y pixel coordinate of mouse position
     pub fn get_mouse_position() -> (i32, i32) {
          unsafe {
               let mut point = POINT { x: 0, y: 0 };
               winapi::um::winuser::GetCursorPos(&mut point);
               return (point.x, point.y)
          };
          
     }

     /// click mouse, either left, right or middle "Mouseclick::LEFT/RIGHT/MIDDLE enumerator"
     pub fn mouse_click(button:Mouseclick){
          let (down, up) = match button { 
               Mouseclick::LEFT => {(MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP) },
               Mouseclick::RIGHT => {(MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP)},
               Mouseclick::MIDDLE => {(MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP)}
               
     
          };
          unsafe {
               // create an array of INPUT structures
               let mut inputs: [INPUT; 2] = [zeroed(), zeroed()];
               // set up the first input event (mouse down)
               inputs[0].type_ = INPUT_MOUSE;
               inputs[0].u.mi_mut().dwFlags = down;
               // set up the second input event (mouse up)
               inputs[1].type_ = INPUT_MOUSE;
               inputs[1].u.mi_mut().dwFlags = up;
               // send the input events
               SendInput(2, inputs.as_mut_ptr(), size_of::<INPUT>() as i32);
               }
     }
}
     
     
     
     
     
     
    



