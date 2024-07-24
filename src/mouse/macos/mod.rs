use std::{
    thread::sleep,
    time::{Duration, Instant}
};
extern crate core_graphics;

use core_graphics::event::{
    CGEvent, CGEventType, CGEventTapLocation, CGMouseButton, 
};

use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
use core_graphics::geometry::CGPoint;
use crate::mouse::Mouseclick;




pub struct Mouse {}

impl Mouse{
    pub fn new() -> Self {
        Self{}
    }
    /// moves mouse to x, y pixel coordinate on screen 
    
    pub fn move_mouse_to_pos (x:i32, y:i32 ,moving_time: f32) {
        if moving_time == 0.0 {
            Mouse::move_mouse(x, y);
        } else {
            
            let start_location = Mouse::get_mouse_position();
            let distance_x = x - start_location.0;
            let distance_y= y -start_location.1;
            let start = Instant::now();
            loop {
                let duration = start.elapsed();
            
                let time_passed_percentage=  duration.as_secs_f32() / moving_time;
                if time_passed_percentage > 10.0 {
                    continue
                }
                let new_x =  start_location.0 as f32 + (time_passed_percentage  * distance_x as f32);     
                let new_y =  start_location.1 as f32 + (time_passed_percentage * distance_y as f32) ;
                if time_passed_percentage >= 1.0 {
                    Mouse::move_mouse(x, y);
                    break
                } else {
                    Mouse::move_mouse(new_x as i32, new_y as i32);
                }
            }
        }
    }

    fn move_mouse(x: i32, y: i32) {
        let event = CGEvent::new_mouse_event(
            CGEventSource::new(CGEventSourceStateID::HIDSystemState).unwrap(),
            CGEventType::MouseMoved,
            CGPoint::new(x as f64, y as f64),
            CGMouseButton::Left,
        ).unwrap();
        event.post(CGEventTapLocation::HID);
        sleep(Duration::from_millis(20));
    }

    
    /// Gets the current mouse position.
    pub fn get_mouse_position() -> (i32 , i32) {
        let event = CGEvent::new(CGEventSource::new(CGEventSourceStateID::HIDSystemState).unwrap()).unwrap();
        let point = event.location();
        (point.x as i32, point.y as i32)

    }
     


}