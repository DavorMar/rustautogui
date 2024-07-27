use std::{
    thread::sleep,
    time::{Duration, Instant}
};

use core_graphics::event::{
    CGEvent, CGEventType, CGEventTapLocation, CGMouseButton, 
};

use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
use core_graphics::geometry::CGPoint;
use crate::mouse::MouseClick;




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

    // separate private function called by move to pos
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

    /// execute left, right or middle mouse click
    pub fn mouse_click(button:MouseClick) {
        let (cg_button, down, up) = match button {
            MouseClick::LEFT => (CGMouseButton::Left, CGEventType::LeftMouseDown, CGEventType::LeftMouseUp),
            MouseClick::RIGHT => (CGMouseButton::Right, CGEventType::RightMouseDown, CGEventType::RightMouseUp),
            MouseClick::MIDDLE => (CGMouseButton::Center, CGEventType::OtherMouseDown, CGEventType::OtherMouseUp),
        };

        // needed as input for where to click
        let mouse_pos = Mouse::get_mouse_position();
    
        
        let click_down = CGEvent::new_mouse_event(
            CGEventSource::new(CGEventSourceStateID::HIDSystemState).unwrap(),
            down,
            CGPoint::new(mouse_pos.0 as f64, mouse_pos.1 as f64),
            cg_button,
        ).unwrap();
        click_down.post(CGEventTapLocation::HID);
        sleep(Duration::from_millis(20));

        
        let click_up = CGEvent::new_mouse_event(
            CGEventSource::new(CGEventSourceStateID::HIDSystemState).unwrap(),
            up,
            CGPoint::new(mouse_pos.0 as f64, mouse_pos.1 as f64),
            cg_button,
        ).unwrap();
        click_up.post(CGEventTapLocation::HID);
        sleep(Duration::from_millis(20));
    }

    pub fn scroll (direction:MouseScroll) {
        let delta = match direction {
            MouseScroll::UP => 10,
            MouseScroll::down => -10,
        };
        let scroll_event = CGEvent::new_scroll_event(
            event_source,
            ScrollEventUnit::PIXEL,
            1, // number of axes
            delta, //value for scroll up or down
        ).unwrap();
        scroll_event.post(CGEventTapLocation::HID);
    }




    pub fn double_click() {
        
        let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState).unwrap();
        let pos = Mouse::get_mouse_position();

        let mouse_down = CGEvent::new_mouse_event(
            source.clone(),
            CGEventType::LeftMouseDown,
            CGPoint::new(pos.0 as f64, pos.1 as f64),
            CGMouseButton::Left,
        ).unwrap();
        mouse_down.set_integer_value_field(1, 1); // first click
        mouse_down.post(CGEventTapLocation::HID);
        sleep(Duration::from_millis(10));
        let mouse_up = CGEvent::new_mouse_event(
            source.clone(),
            CGEventType::LeftMouseUp,
            CGPoint::new(pos.0 as f64, pos.1 as f64),
            CGMouseButton::Left,
        ).unwrap();
        mouse_up.set_integer_value_field(1, 1); // first click
        mouse_up.post(CGEventTapLocation::HID);

        sleep(Duration::from_millis(50)); // Small delay between clicks

        let mouse_down_2 = CGEvent::new_mouse_event(
            source.clone(),
            CGEventType::LeftMouseDown,
            CGPoint::new(pos.0 as f64, pos.1 as f64),
            CGMouseButton::Left,
        ).unwrap();
        mouse_down_2.set_integer_value_field(1, 2); // double click
        mouse_down_2.post(CGEventTapLocation::HID);
        sleep(Duration::from_millis(10));
        let mouse_up_2 = CGEvent::new_mouse_event(
            source,
            CGEventType::LeftMouseUp,
            CGPoint::new(pos.0 as f64, pos.1 as f64),
            CGMouseButton::Left,
        ).unwrap();
        mouse_up_2.set_integer_value_field(1, 2); // double click
        mouse_up_2.post(CGEventTapLocation::HID);
        sleep(Duration::from_millis(10));
    }

    

}