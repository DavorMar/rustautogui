use winapi::shared::windef::POINT;
use winapi::um::winuser::{SetCursorPos, SendInput, INPUT, INPUT_MOUSE,
     MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP,
     MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP,
     MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP,MOUSEEVENTF_WHEEL, MOUSEEVENTF_HWHEEL
};
use std::time::Instant;
use std::{thread, time};
use std::mem::{zeroed, size_of};
use crate::mouse::{MouseClick, MouseScroll};


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

               if moving_time <= 0.0 {
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
          
     pub fn drag_mouse(x: i32, y:i32, moving_time:f32) {
          let (down, up) = (MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP);
          unsafe {
               
               
               // set up the first input event (mouse down)
               let mut input_down: INPUT = zeroed();
               input_down.type_ = INPUT_MOUSE;
               input_down.u.mi_mut().dwFlags = down;
               SendInput(1, &mut input_down, size_of::<INPUT>() as i32);
               let some_duration = time::Duration::from_millis(80);
               thread::sleep(some_duration);
               Mouse::move_mouse_to_pos(x,y, moving_time);
               
               let some_duration = time::Duration::from_millis(50);
               thread::sleep(some_duration);


               // set up the second input event (mouse up)
               let mut input_up: INPUT = zeroed();
               input_up.type_ = INPUT_MOUSE;
               input_up.u.mi_mut().dwFlags = up;
               // send the input events
               SendInput(2, & mut input_up, size_of::<INPUT>() as i32);
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


     /// click mouse, either left, right or middle "MouseClick::LEFT/RIGHT/MIDDLE enumerator"
     pub fn mouse_click(button:MouseClick){
          let (down, up) = match button { 
               MouseClick::LEFT => {(MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP) },
               MouseClick::RIGHT => {(MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP)},
               MouseClick::MIDDLE => {(MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP)}
               
     
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

 

     pub fn scroll(direction: MouseScroll) {
          let (amount, wheel_direction) = match direction {
               MouseScroll::UP => (120, MOUSEEVENTF_WHEEL),
               MouseScroll::DOWN => (-120, MOUSEEVENTF_WHEEL),
               MouseScroll::LEFT => (-120, MOUSEEVENTF_HWHEEL),
               MouseScroll::RIGHT => (120, MOUSEEVENTF_HWHEEL),
          };
          unsafe {
               let mut scroll_input: INPUT = zeroed();
              
               scroll_input.type_ = INPUT_MOUSE;
               scroll_input.u.mi_mut().dwFlags = wheel_direction;
               scroll_input.u.mi_mut().mouseData = amount as u32;
               SendInput(1, &mut scroll_input, size_of::<INPUT>() as i32);
          }    
     }


}
     
     
     
     
     
     
    



