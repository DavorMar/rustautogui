#![cfg(feature = "opencl")]

use rustautogui;

fn main() {
    //initiate gui
    let mut gui = rustautogui::RustAutoGui::new(false).unwrap();
    // prepare and store template with Opencl matchmode VERSION 2 and custom threshold of 0.8
    gui.store_template_from_file_custom(
        "test.png",
        Some((500, 100, 1000, 700).into()),
        rustautogui::MatchMode::SegmentedOclV2,
        "test_img",
        0.8,
    )
    .unwrap();
    // store another image with Opencl VERSION 1 with automatic determination of threshold
    gui.store_template_from_file(
        "test2.png",
        None,
        rustautogui::MatchMode::SegmentedOcl,
        "test_img2",
    )
    .unwrap();

    // store third template with no opencl, which will run on cpu
    gui.store_template_from_file(
        "test3.png",
        None,
        rustautogui::MatchMode::Segmented,
        "test_img3",
    )
    .unwrap();

    // execute image searches in completely same pattern. Using opencl is determined in preparation phase
    gui.find_stored_image_on_screen(0.9, "test_img").unwrap();
    gui.find_stored_image_on_screen(0.9, "test_img2").unwrap();
    gui.find_stored_image_on_screen(0.9, "test_img3").unwrap();
}
