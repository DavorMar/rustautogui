// run with cargo test --tests --release -- --nocapture

#[cfg(feature = "dev")]
mod multi_test {
    use rustautogui::{self, imgtools, RustAutoGui};

    #[test]
    fn main_test() {
        let mut gui = rustautogui::RustAutoGui::new(true).unwrap();
        gui.list_devices();
        load_imgs(&mut gui);
        gui.loop_find_stored_image_on_screen_and_move_mouse(0.9, 1.0, 10, "step_0")
            .unwrap();
        gui.left_click().unwrap();
        gui.loop_find_stored_image_on_screen_and_move_mouse(0.9, 0.5, 10, "step_1")
            .unwrap();
        gui.left_click().unwrap();
        gui.loop_find_stored_image_on_screen_and_move_mouse(0.9, 1.5, 10, "step_2")
            .unwrap();
        gui.left_click().unwrap();
        gui.keyboard_input("test!@#45<>/\\|{}[]&*()_+").unwrap();
    }

    fn load_imgs(gui: &mut RustAutoGui) {


        #[cfg(target_os = "windows")]
        let insert = 'w';
        #[cfg(target_os = "macos")]
        let insert = "m";
        #[cfg(target_os = "linux")]
        let insert = 'l';
        let img: image::ImageBuffer<image::Luma<u8>, Vec<u8>> = imgtools::load_image_bw(
            format!("tests/testing_images/gui_tests/step_1_{}.png", insert).as_str(),
        )
        .unwrap();
        gui.store_template_from_imagebuffer_custom(
            img,
            None,
            rustautogui::MatchMode::SegmentedOcl,
            "step_0",
            0.9,
        )
        .unwrap();

        gui.store_template_from_file(
            format!("tests/testing_images/gui_tests/step_2_{}.png", insert).as_str(),
            None,
            rustautogui::MatchMode::SegmentedOclV2,
            "step_1",
        )
        .unwrap();

        gui.store_template_from_file_custom(
            format!("tests/testing_images/gui_tests/step_3_{}.png", insert).as_str(),
            None,
            rustautogui::MatchMode::Segmented,
            "step_2",
            0.8,
        )
        .unwrap();
    }
}
