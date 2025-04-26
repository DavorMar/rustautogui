use crate::core::mouse::{MouseClick, MouseScroll};
use std::mem::{size_of, zeroed};
use std::{thread, time, time::Instant};
use winapi::shared::windef::POINT;
use winapi::um::winuser::{
    SendInput, SetCursorPos, INPUT, INPUT_MOUSE, MOUSEEVENTF_HWHEEL, MOUSEEVENTF_LEFTDOWN,
    MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_RIGHTDOWN,
    MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_WHEEL,
};
#[derive(Debug)]
pub struct Mouse {}
impl Mouse {
    #[allow(unused_variables)]
    pub fn new() -> Mouse {
        Mouse {}
    }

    /// moves mouse to x, y pixel coordinate on screen
    pub fn move_mouse_to_pos(x: i32, y: i32, moving_time: f32) {
        // if no moving time, then instant move is executed
        unsafe {
            if moving_time <= 0.0 {
                SetCursorPos(x, y);
                return;
            }
        };
        // if moving time is included, loop is executed that moves step by step
        let start = Instant::now();
        let start_location = Mouse::get_mouse_position();
        let distance_x = x - start_location.0;
        let distance_y = y - start_location.1;

        loop {
            let duration = start.elapsed().as_secs_f32();

            let time_passed_percentage = duration / moving_time;
            // on first iterations, time passed percentage gets values greater than 10, probably because duration is a
            // very small number. Probably could do if duration < 0.05 or similar
            if time_passed_percentage > 10.0 {
                continue;
            }
            let new_x = start_location.0 as f32 + (time_passed_percentage * distance_x as f32);
            let new_y = start_location.1 as f32 + (time_passed_percentage * distance_y as f32);

            unsafe {
                if time_passed_percentage >= 1.0 {
                    SetCursorPos(x, y);
                    break;
                } else {
                    SetCursorPos(new_x as i32, new_y as i32);
                }
            }
        }
    }

    pub fn drag_mouse(x: i32, y: i32, moving_time: f32) {
        let (down, up) = (MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP);
        unsafe {
            // set up the first input event (mouse down)
            let mut input_down: INPUT = zeroed();
            input_down.type_ = INPUT_MOUSE;
            input_down.u.mi_mut().dwFlags = down;
            SendInput(1, &mut input_down, size_of::<INPUT>() as i32);
            // wait a bit after click down, before moving
            thread::sleep(time::Duration::from_millis(80));
            Mouse::move_mouse_to_pos(x, y, moving_time);
            thread::sleep(time::Duration::from_millis(50));
            // set up the second input event (mouse up)
            let mut input_up: INPUT = zeroed();
            input_up.type_ = INPUT_MOUSE;
            input_up.u.mi_mut().dwFlags = up;
            // send the input events
            SendInput(2, &mut input_up, size_of::<INPUT>() as i32);
        }
    }

    /// returns x, y pixel coordinate of mouse position
    pub fn get_mouse_position() -> (i32, i32) {
        unsafe {
            let mut point = POINT { x: 0, y: 0 };
            winapi::um::winuser::GetCursorPos(&mut point);
            return (point.x, point.y);
        };
    }

    /// click mouse, either left, right or middle "MouseClick::LEFT/RIGHT/MIDDLE enumerator"
    pub fn mouse_click(button: MouseClick) {
        // create event type depending on click type
        let (down, up) = match button {
            MouseClick::LEFT => (MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP),
            MouseClick::RIGHT => (MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP),
            MouseClick::MIDDLE => (MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP),
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

    pub fn mouse_down(button: MouseClick) {
        // create event type depending on click type
        let down = match button {
            MouseClick::LEFT => MOUSEEVENTF_LEFTDOWN,
            MouseClick::RIGHT => MOUSEEVENTF_RIGHTDOWN,
            MouseClick::MIDDLE => MOUSEEVENTF_MIDDLEDOWN,
        };
        unsafe {
            // create an array of INPUT structures
            let mut input: INPUT = zeroed();

            input.type_ = INPUT_MOUSE;
            input.u.mi_mut().dwFlags = down;

            // send the input events
            SendInput(1, &mut input, size_of::<INPUT>() as i32);
        }
    }

    pub fn mouse_up(button: MouseClick) {
        // create event type depending on click type
        let up = match button {
            MouseClick::LEFT => MOUSEEVENTF_LEFTUP,
            MouseClick::RIGHT => MOUSEEVENTF_RIGHTUP,
            MouseClick::MIDDLE => MOUSEEVENTF_MIDDLEUP,
        };
        unsafe {
            // create an array of INPUT structures
            let mut input: INPUT = zeroed();
            // set up thefirstut vent (mous;
            input.type_ = INPUT_MOUSE;
            input.u.mi_mut().dwFlags = up;

            // send the input events
            SendInput(1, &mut input, size_of::<INPUT>() as i32);
        }
    }

    pub fn scroll(direction: MouseScroll, intensity: u32) {
        // direction , H or W wheel, depending on axis scrolled
        let (amount, wheel_direction) = match direction {
            MouseScroll::UP => (120, MOUSEEVENTF_WHEEL),
            MouseScroll::DOWN => (-120, MOUSEEVENTF_WHEEL),
            MouseScroll::LEFT => (-120, MOUSEEVENTF_HWHEEL),
            MouseScroll::RIGHT => (120, MOUSEEVENTF_HWHEEL),
        };
        let amount = amount * intensity as i32;
        unsafe {
            let mut scroll_input: INPUT = zeroed();

            scroll_input.type_ = INPUT_MOUSE;
            scroll_input.u.mi_mut().dwFlags = wheel_direction;
            scroll_input.u.mi_mut().mouseData = amount as u32;
            SendInput(1, &mut scroll_input, size_of::<INPUT>() as i32);
        }
    }
}
