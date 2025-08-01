use std::{
    thread::sleep,
    time::{Duration, Instant},
};

use crate::core::mouse::{MouseClick, MouseScroll};
use crate::errors::AutoGuiError;
use core_graphics::{
    event::{CGEvent, CGEventTapLocation, CGEventType, CGMouseButton, ScrollEventUnit},
    event_source::{CGEventSource, CGEventSourceStateID},
    geometry::CGPoint,
};

pub struct Mouse {}

impl Mouse {
    pub fn new() -> Self {
        Self {}
    }
    /// moves mouse to x, y pixel coordinate on screen

    pub fn move_mouse_to_pos(x: i32, y: i32, moving_time: f32) -> Result<(), AutoGuiError> {
        if moving_time <= 0.0 {
            Mouse::move_mouse(x, y)
        } else {
            let start_location = Mouse::get_mouse_position()?;
            let distance_x = x - start_location.0;
            let distance_y = y - start_location.1;
            let start = Instant::now();
            loop {
                let duration = start.elapsed().as_secs_f32();

                let time_passed_percentage = duration / moving_time;
                if time_passed_percentage > 10.0 {
                    continue;
                }
                let new_x = start_location.0 as f32 + (time_passed_percentage * distance_x as f32);
                let new_y = start_location.1 as f32 + (time_passed_percentage * distance_y as f32);
                if time_passed_percentage >= 1.0 {
                    Mouse::move_mouse(x, y)?;
                    break;
                } else {
                    Mouse::move_mouse(new_x as i32, new_y as i32)?;
                }
            }
            Ok(())
        }
    }

    pub fn drag_mouse(x: i32, y: i32, moving_time: f32) -> Result<(), AutoGuiError> {
        let (cg_button, down, up) = (
            CGMouseButton::Left,
            CGEventType::LeftMouseDown,
            CGEventType::LeftMouseUp,
        );
        let drag = CGEventType::LeftMouseDragged;
        // needed as input for where to click
        let mouse_pos = Mouse::get_mouse_position()?;
        // click down
        let cg_event_source =
            CGEventSource::new(CGEventSourceStateID::HIDSystemState).map_err(|_| {
                AutoGuiError::OSFailure(
                    "Error creating CGEventSource on mouse movement".to_string(),
                )
            })?;

        let click_down = CGEvent::new_mouse_event(
            cg_event_source.clone(),
            down,
            CGPoint::new(mouse_pos.0 as f64, mouse_pos.1 as f64),
            cg_button,
        )
        .map_err(|_| AutoGuiError::OSFailure("Failed the mouse click down CGevent".to_string()))?;
        click_down.post(CGEventTapLocation::HID);

        sleep(Duration::from_millis(100));

        // Move mouse with dragging event
        let distance = (((x - mouse_pos.0).pow(2) + (y - mouse_pos.1).pow(2)) as f32).sqrt();
        let steps = distance / 20.0; // Adjust for smoothness

        let dx = (x - mouse_pos.0) as f64 / steps as f64;
        let dy = (y - mouse_pos.1) as f64 / steps as f64;

        for i in 1..=steps as i32 {
            let new_x = mouse_pos.0 as f64 + dx * i as f64;
            let new_y = mouse_pos.1 as f64 + dy * i as f64;

            let drag_event = CGEvent::new_mouse_event(
                cg_event_source.clone(),
                drag, // Use LeftMouseDragged instead of just moving
                CGPoint::new(new_x, new_y),
                cg_button,
            )
            .map_err(|_| AutoGuiError::OSFailure("Failed to create drag CGEvent".to_string()))?;
            drag_event.post(CGEventTapLocation::HID);

            sleep(Duration::from_millis(
                (moving_time * 1000.0 / steps as f32) as u64,
            ));
        }

        //click up
        let mouse_pos = Mouse::get_mouse_position()?;
        let cg_event_source =
            CGEventSource::new(CGEventSourceStateID::HIDSystemState).map_err(|_| {
                AutoGuiError::OSFailure(
                    "Error creating CGEventSource on mouse movement".to_string(),
                )
            })?;

        let click_up = CGEvent::new_mouse_event(
            cg_event_source,
            up,
            CGPoint::new(mouse_pos.0 as f64, mouse_pos.1 as f64),
            cg_button,
        )
        .map_err(|_| AutoGuiError::OSFailure("Failed the mouse click up CGevent".to_string()))?;
        click_up.post(CGEventTapLocation::HID);

        sleep(Duration::from_millis(20));

        Ok(())
    }

    // separate private function called by move to pos
    fn move_mouse(x: i32, y: i32) -> Result<(), AutoGuiError> {
        let gc_event_source =
            CGEventSource::new(CGEventSourceStateID::HIDSystemState).map_err(|_| {
                AutoGuiError::OSFailure(
                    "Error creating CGEventSource on mouse movement".to_string(),
                )
            })?;

        let event = CGEvent::new_mouse_event(
            gc_event_source,
            CGEventType::MouseMoved,
            CGPoint::new(x as f64, y as f64),
            CGMouseButton::Left,
        )
        .map_err(|_| AutoGuiError::OSFailure("Failed creating CGEvent".to_string()))?;
        event.post(CGEventTapLocation::HID);

        sleep(Duration::from_millis(20));
        Ok(())
    }

    /// Gets the current mouse position.
    pub fn get_mouse_position() -> Result<(i32, i32), AutoGuiError> {
        let gc_event_source =
            CGEventSource::new(CGEventSourceStateID::HIDSystemState).map_err(|_| {
                AutoGuiError::OSFailure(
                    "Error creating CGEventSource on mouse movement".to_string(),
                )
            })?;
        let event = CGEvent::new(gc_event_source)
            .map_err(|_| AutoGuiError::OSFailure("Failed creating CGevent".to_string()))?;
        let point = event.location();
        Ok((point.x as i32, point.y as i32))
    }

    /// execute left, right or middle mouse click
    pub fn mouse_click(button: MouseClick) -> Result<(), AutoGuiError> {
        let (cg_button, down, up) = match button {
            MouseClick::LEFT => (
                CGMouseButton::Left,
                CGEventType::LeftMouseDown,
                CGEventType::LeftMouseUp,
            ),
            MouseClick::RIGHT => (
                CGMouseButton::Right,
                CGEventType::RightMouseDown,
                CGEventType::RightMouseUp,
            ),
            MouseClick::MIDDLE => (
                CGMouseButton::Center,
                CGEventType::OtherMouseDown,
                CGEventType::OtherMouseUp,
            ),
        };

        // needed as input for where to click
        let mouse_pos = Mouse::get_mouse_position()?;

        let cg_event_source =
            CGEventSource::new(CGEventSourceStateID::HIDSystemState).map_err(|_| {
                AutoGuiError::OSFailure(
                    "Error creating CGEventSource on mouse movement".to_string(),
                )
            })?;
        let click_down = CGEvent::new_mouse_event(
            cg_event_source,
            down,
            CGPoint::new(mouse_pos.0 as f64, mouse_pos.1 as f64),
            cg_button,
        )
        .map_err(|_| AutoGuiError::OSFailure("Failed the mouse click down CGevent".to_string()))?;
        click_down.post(CGEventTapLocation::HID);

        sleep(Duration::from_millis(20));

        let cg_event_source =
            CGEventSource::new(CGEventSourceStateID::HIDSystemState).map_err(|_| {
                AutoGuiError::OSFailure(
                    "Error creating CGEventSource on mouse movement".to_string(),
                )
            })?;

        let click_up = CGEvent::new_mouse_event(
            cg_event_source,
            up,
            CGPoint::new(mouse_pos.0 as f64, mouse_pos.1 as f64),
            cg_button,
        )
        .map_err(|_| AutoGuiError::OSFailure("Failed the mouse click up CGevent".to_string()))?;

        click_up.post(CGEventTapLocation::HID);

        sleep(Duration::from_millis(20));
        Ok(())
    }

    pub fn mouse_down(button: MouseClick) -> Result<(), AutoGuiError> {
        let (cg_button, down) = match button {
            MouseClick::LEFT => (CGMouseButton::Left, CGEventType::LeftMouseDown),
            MouseClick::RIGHT => (CGMouseButton::Right, CGEventType::RightMouseDown),
            MouseClick::MIDDLE => (CGMouseButton::Center, CGEventType::OtherMouseDown),
        };

        // needed as input for where to click
        let mouse_pos = Mouse::get_mouse_position()?;

        let cg_event_source =
            CGEventSource::new(CGEventSourceStateID::HIDSystemState).map_err(|_| {
                AutoGuiError::OSFailure(
                    "Error creating CGEventSource on mouse movement".to_string(),
                )
            })?;
        let click_down = CGEvent::new_mouse_event(
            cg_event_source,
            down,
            CGPoint::new(mouse_pos.0 as f64, mouse_pos.1 as f64),
            cg_button,
        )
        .map_err(|_| AutoGuiError::OSFailure("Failed the mouse click down CGevent".to_string()))?;
        click_down.post(CGEventTapLocation::HID);
        sleep(Duration::from_millis(20));
        Ok(())
    }

    pub fn mouse_up(button: MouseClick) -> Result<(), AutoGuiError> {
        let (cg_button, up) = match button {
            MouseClick::LEFT => (CGMouseButton::Left, CGEventType::LeftMouseUp),
            MouseClick::RIGHT => (CGMouseButton::Right, CGEventType::RightMouseUp),
            MouseClick::MIDDLE => (CGMouseButton::Center, CGEventType::OtherMouseUp),
        };

        // needed as input for where to click
        let mouse_pos = Mouse::get_mouse_position()?;

        let cg_event_source =
            CGEventSource::new(CGEventSourceStateID::HIDSystemState).map_err(|_| {
                AutoGuiError::OSFailure(
                    "Error creating CGEventSource on mouse movement".to_string(),
                )
            })?;

        let click_up = CGEvent::new_mouse_event(
            cg_event_source,
            up,
            CGPoint::new(mouse_pos.0 as f64, mouse_pos.1 as f64),
            cg_button,
        )
        .map_err(|_| AutoGuiError::OSFailure("Failed the mouse click up CGevent".to_string()))?;

        click_up.post(CGEventTapLocation::HID);

        sleep(Duration::from_millis(20));
        Ok(())
    }

    pub fn scroll(direction: MouseScroll, intensity: u32) -> Result<(), AutoGuiError> {
        let delta = match direction {
            MouseScroll::UP => (intensity as i32, 0),
            MouseScroll::DOWN => (-1 * intensity as i32, 0),
            MouseScroll::LEFT => (0, intensity as i32),
            MouseScroll::RIGHT => (0, -1 * intensity as i32),
        };
        let cg_event_source =
            CGEventSource::new(CGEventSourceStateID::HIDSystemState).map_err(|_| {
                AutoGuiError::OSFailure(
                    "Error creating CGEventSource on mouse movement".to_string(),
                )
            })?;

        let scroll = CGEvent::new_scroll_event(
            cg_event_source,
            ScrollEventUnit::PIXEL,
            2,
            delta.0,
            delta.1,
            0,
        )
        .map_err(|_| AutoGuiError::OSFailure("Failed creating mouse scroll CGevent".to_string()))?;
        scroll.post(CGEventTapLocation::HID);
        sleep(Duration::from_millis(20));
        Ok(())
    }

    pub fn double_click() -> Result<(), AutoGuiError> {
        let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState).map_err(|_| {
            AutoGuiError::OSFailure(
                "Failed creating CGEventSource on mouse double click".to_string(),
            )
        })?;
        let pos = Mouse::get_mouse_position()?;

        // needed first to get focus of the window
        Self::mouse_click(MouseClick::LEFT)?;
        sleep(Duration::from_millis(50));

        // MacOs does double click wierldy.
        // good explanation at https://stackoverflow.com/questions/1483657/performing-a-double-click-using-cgeventcreatemouseevent
        // basically, the x.set_integer_value_field defines event as double click. Sending 2 times a left click does not work
        let mouse_down = CGEvent::new_mouse_event(
            source.clone(),
            CGEventType::LeftMouseDown,
            CGPoint::new(pos.0 as f64, pos.1 as f64),
            CGMouseButton::Left,
        )
        .map_err(|_| {
            AutoGuiError::OSFailure(
                "Failed creating CGevent for mouse click down action".to_string(),
            )
        })?;
        mouse_down.set_integer_value_field(1, 2);

        let mouse_up = CGEvent::new_mouse_event(
            source.clone(),
            CGEventType::LeftMouseUp,
            CGPoint::new(pos.0 as f64, pos.1 as f64),
            CGMouseButton::Left,
        )
        .map_err(|_| {
            AutoGuiError::OSFailure("Failed creating CGevent for mouse up click".to_string())
        })?;
        mouse_up.set_integer_value_field(1, 2);

        mouse_down.post(CGEventTapLocation::HID);
        sleep(Duration::from_millis(10));
        mouse_up.post(CGEventTapLocation::HID);
        sleep(Duration::from_millis(50));

        Ok(())
    }
}
