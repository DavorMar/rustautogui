use rustautogui::RustAutoGui;
use std::thread;
use std::time;

// code that opens chrome, new tab, goes to github, mouse overs star (click commented out), scrolls to the bottom and clicks on terms
// no image files included for examples

fn main() {
    let mut rustautogui = RustAutoGui::new(false).unwrap(); // initialize
    let (screen_w, screen_h) = rustautogui.get_screen_size();

    // load, process and store image of star(can be image of project name text)
    // regions are written in this way to cover any screen size, not hardcoding them
    rustautogui
        .store_template_from_file(
            "/home/davor/Pictures/stars.png",
            Some((
                (0.2 * screen_w as f32) as u32, // start x
                0,                              // start y
                (0.5 * screen_w as f32) as u32, // width
                (0.4 * screen_h as f32) as u32, // height
            )),
            rustautogui::MatchMode::Segmented,
            None,
            "stars".to_string(),
        )
        .unwrap();
    
    // just for example doing single prepare, but you would want to store it also
    // load, process and store image of terms
    rustautogui
        .prepare_template_from_file(
            "/home/davor/Pictures/terms.png",
            Some((
                (0.1 * screen_w as f32) as u32, // start x
                (0.7 * screen_h as f32) as u32, // start y
                (0.5 * screen_w as f32) as u32, // width
                (0.3 * screen_h as f32) as u32, // height
            )),
            rustautogui::MatchMode::Segmented,
            None,
        )
        .unwrap();

    // press windows/super key
    rustautogui.keyboard_command("win").unwrap();

    thread::sleep(time::Duration::from_millis(500));

    // write in chrome
    rustautogui.keyboard_input("chrome").unwrap();

    thread::sleep(time::Duration::from_millis(500));

    // run chrome with clicking return/enter
    rustautogui.keyboard_command("return").unwrap();

    thread::sleep(time::Duration::from_millis(500));

    // open new tab with ctrl+t
    rustautogui.keyboard_multi_key("ctrl", "t", None).unwrap();

    thread::sleep(time::Duration::from_millis(500));

    // input github in url (url input area selected automatically by new tab opening)
    rustautogui
        .keyboard_input("https://github.com/DavorMar/rustautogui")
        .unwrap();

    thread::sleep(time::Duration::from_millis(500));

    // return/enter to open github
    rustautogui.keyboard_command("return").unwrap();

    thread::sleep(time::Duration::from_millis(500));

    // loop till image of star is found or timeout of 15 seconds is hit
    rustautogui
        .loop_find_stored_image_on_screen_and_move_mouse(0.95, 1.0, 15, &"stars".to_string())
        .unwrap();

    thread::sleep(time::Duration::from_millis(500));
    // rustautogui.left_click().unwrap();
    thread::sleep(time::Duration::from_millis(500));

    // scroll down 70 times to the bottom of the page where small terms button is
    for _ in 0..=70 {
        rustautogui.scroll_down().unwrap();
    }

    // loop till image found with timeout of 15 seconds
    rustautogui
        .loop_find_image_on_screen_and_move_mouse(0.95, 1.0, 15)
        .unwrap();

    thread::sleep(time::Duration::from_millis(500));
    rustautogui.left_click().unwrap();
}
