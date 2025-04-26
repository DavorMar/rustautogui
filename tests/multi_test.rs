#[cfg(feature = "dev")]
mod multi_test {
    use rustautogui::{self, imgtools, RustAutoGui};

    #[test]
    fn main_test() {
        let mut gui = rustautogui::RustAutoGui::new(true).unwrap();
        load_imgs(&mut gui);
        gui.loop_find_stored_image_on_screen_and_move_mouse(0.9, 1.0, 10,"step_0").unwrap();
        gui.left_click().unwrap();
        gui.loop_find_stored_image_on_screen_and_move_mouse(0.9, 0.5, 10, "step_1").unwrap();
        gui.left_click().unwrap();
        gui.loop_find_stored_image_on_screen_and_move_mouse(0.9, 1.5, 10, "step_2").unwrap();
        gui.left_click().unwrap();
        gui.keyboard_input("test!@#45<>/\\|{}[]^&*()_+").unwrap();
    }

    fn load_imgs(gui: &mut RustAutoGui) {
        let (s_w, s_h) = gui.get_screen_size();
        let sw = s_w as u32;
        let sh = s_h as u32;
        
        #[cfg(target_os = "windows")]
        {
            let img: image::ImageBuffer<image::Luma<u8>, Vec<u8>> = imgtools::load_image_bw("tests/testing_images/gui_tests/windows.png").unwrap();
            gui.store_template_from_imagebuffer(
                img,
                Some((0, 500, 500, sh - 500)),
                rustautogui::MatchMode::Segmented,
                "step_0",
            )
            .unwrap();

            gui.store_template_from_file(
                "tests/testing_images/gui_tests/win_settings.png",
                Some((0, 500, 500, sh - 500)),
                rustautogui::MatchMode::SegmentedOclV2,
                "step_1",
            )
            .unwrap();

            gui.store_template_from_file_custom(
                "tests/testing_images/gui_tests/win_find_setting.png",
                None,
                rustautogui::MatchMode::SegmentedOcl,
                "step_2",
                0.8
            )
            .unwrap();
        }
    }
}
