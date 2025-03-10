use std::{
    thread::sleep,
    time::{Duration, Instant}
};

use core_graphics::event::{
    CGEvent, CGEventType, CGEventTapLocation, CGMouseButton, ScrollEventUnit
};

use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
use core_graphics::geometry::CGPoint;
use crate::mouse::{MouseClick,MouseScroll};




pub struct Mouse {}

impl Mouse{
    pub fn new() -> Self {
        Self{}
    }
    /// moves mouse to x, y pixel coordinate on screen 
    
    pub fn move_mouse_to_pos (x:i32, y:i32 ,moving_time: f32) -> Result<(), &'static str>{
        if moving_time == 0.0 {
            Mouse::move_mouse(x, y)?;
            return Ok(())
        } else {
            
            let start_location = Mouse::get_mouse_position()?;
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
                    Mouse::move_mouse(x, y)?;
                    break
                } else {
                    Mouse::move_mouse(new_x as i32, new_y as i32)?;
                }
            }
            return Ok(())
        }
    }

    // separate private function called by move to pos
    fn move_mouse(x: i32, y: i32) -> Result<(), &'static str> {
        let gc_event_source = CGEventSource::new(CGEventSourceStateID::HIDSystemState);
        let gc_event_source = match gc_event_source {
            Ok(x) => x,
            Err(_) => return Err("Error creating CGEventSource on mouse movement"),
        };
        let event = CGEvent::new_mouse_event(
            gc_event_source,
            CGEventType::MouseMoved,
            CGPoint::new(x as f64, y as f64),
            CGMouseButton::Left,
        );
        match event {
            Ok(xevent) => xevent.post(CGEventTapLocation::HID),
            Err(_) => return Err("Failed creating CGEvent")
            
        };
        
        sleep(Duration::from_millis(20));
        Ok(())
    }

    
    /// Gets the current mouse position.
    pub fn get_mouse_position() -> Result<(i32, i32), &'static str> {
        let gc_event_source = CGEventSource::new(CGEventSourceStateID::HIDSystemState);
        let gc_event_source = match gc_event_source {
            Ok(x) => x,
            Err(_) => return Err("Error creating CGEventSource on mouse movement"),
        };
        let event = CGEvent::new(gc_event_source);
        let event = match event {
            Ok(x)=>x,
            Err(_) => return Err("Failed creating CGevent") 
        };
        let point = event.location();
        Ok((point.x as i32, point.y as i32))
    }

    /// execute left, right or middle mouse click
    pub fn mouse_click(button:MouseClick) -> Result<(), &'static str> {
        let (cg_button, down, up) = match button {
            MouseClick::LEFT => (CGMouseButton::Left, CGEventType::LeftMouseDown, CGEventType::LeftMouseUp),
            MouseClick::RIGHT => (CGMouseButton::Right, CGEventType::RightMouseDown, CGEventType::RightMouseUp),
            MouseClick::MIDDLE => (CGMouseButton::Center, CGEventType::OtherMouseDown, CGEventType::OtherMouseUp),
        };

        // needed as input for where to click
        let mouse_pos = Mouse::get_mouse_position()?;
    
        let cg_event_source = CGEventSource::new(CGEventSourceStateID::HIDSystemState);
        let cg_event_source = match cg_event_source {
            Ok(x) => x,
            Err(_) => return Err("Error creating CGEventSource on mouse movement"),
        };
        
        let click_down = CGEvent::new_mouse_event(
            cg_event_source,
            down,
            CGPoint::new(mouse_pos.0 as f64, mouse_pos.1 as f64),
            cg_button,
        );
        match click_down {
            Ok(x) => x.post(CGEventTapLocation::HID),
            Err(_) => return Err("Failed the mouse click down CGevent")

        }
        
        sleep(Duration::from_millis(20));

        let cg_event_source = CGEventSource::new(CGEventSourceStateID::HIDSystemState);
        let cg_event_source = match cg_event_source {
            Ok(x) => x,
            Err(_) => return Err("Error creating CGEventSource on mouse movement"),
        };

        let click_up = CGEvent::new_mouse_event(
            cg_event_source,
            up,
            CGPoint::new(mouse_pos.0 as f64, mouse_pos.1 as f64),
            cg_button,
        );
        match click_up {
            Ok(x) => x.post(CGEventTapLocation::HID),
            Err(_) => return Err("Failed the mouse click up CGevent"),
        }
        
        sleep(Duration::from_millis(20));
        Ok(())
    }

    pub fn scroll (direction:MouseScroll) -> Result<(), &'static str> {
        let delta = match direction {
            MouseScroll::UP => 10,
            MouseScroll::DOWN => -10,
        };
        let cg_event_source = CGEventSource::new(CGEventSourceStateID::HIDSystemState);
        let cg_event_source = match cg_event_source {
            Ok(x) => x,
            Err(_) => return Err("Error creating CGEventSource on mouse movement"),
        };


        let scroll = CGEvent::new_scroll_event(
            cg_event_source,
            ScrollEventUnit::PIXEL,
            1,
            delta,
            0,
            0
        );
        match scroll {
            Ok(xscroll) => xscroll.post(CGEventTapLocation::HID),
            Err(_) => return Err("Failed creating mouse scroll CGevent"),
        }
        
        Ok(())
    }




    pub fn double_click() -> Result<(), &'static str>{

        let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState);
        let source = match source {
            Ok(x) => x,
            Err(_) => return Err("Failed creating CGEventSource on mouse double click")
        };
        let pos = Mouse::get_mouse_position()?;
        for _ in [0..2] {
            let mouse_down = CGEvent::new_mouse_event(
                source.clone(),
                CGEventType::LeftMouseDown,
                CGPoint::new(pos.0 as f64, pos.1 as f64),
                CGMouseButton::Left,
            );
            match mouse_down {
                Ok(x) => {
                    x.set_integer_value_field(1, 1); // first click
                    x.post(CGEventTapLocation::HID);
                    sleep(Duration::from_millis(10));
                },
                Err(_) => return Err("Failed creating CGevent for mouse click down action")
            };
            
            let mouse_up = CGEvent::new_mouse_event(
                source.clone(),
                CGEventType::LeftMouseUp,
                CGPoint::new(pos.0 as f64, pos.1 as f64),
                CGMouseButton::Left,
            );
            match mouse_up {
                Ok(x) => {
                    x.set_integer_value_field(1, 1); // first click
                    x.post(CGEventTapLocation::HID);
                },
                Err(_) => return Err("Failed creating CGevent for mouse up click")
            };
            
    
            sleep(Duration::from_millis(50)); // Small delay between clicks
        }
        

        Ok(())
    }

    

}