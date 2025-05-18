// run with cargo test --tests --release -- --nocapture

// #[cfg(feature = "dev")]
// pub mod mouse_tests {
//     use rustautogui;

//     #[test]
//     fn execute_tests() {
//         let mut gui = rustautogui::RustAutoGui::new(false).unwrap();
//         let (s_w, s_h) = gui.get_screen_size();

//         let center_x = (s_w / 2) as u32;
//         let center_y = (s_h / 2) as u32;

//         // makes a square movement around the center
//         gui.move_mouse_to_pos(center_x, center_y, 0.5).unwrap();
//         gui.move_mouse(-(center_x as i32 / 2), 0, 0.5).unwrap();
//         gui.move_mouse(0, -(center_y as i32 / 2), 0.5).unwrap();
//         gui.move_mouse_to(Some(center_x + center_x / 2), None, 0.5)
//             .unwrap();
//         gui.move_mouse_to_pos(center_x + center_x / 2, center_y + center_y / 2, 0.5)
//             .unwrap();
//         gui.move_mouse(-3 * s_w / 4, 0, 1.5).unwrap();
//         gui.move_mouse_to(None, Some(s_h as u32 / 2), 0.5).unwrap();
//         gui.move_mouse(s_w - 1, 0, 1.5).unwrap();
//     }
// }
