use std::time::Instant;

use rustautogui::{MatchMode, RustAutoGui};
fn main() {
    let mut ag = RustAutoGui::new(false).unwrap();
    dbg!(ag.get_screen_size());
    ag.store_template_from_file(
        "./tests/testing_images/algorithm_tests/Socket_template1.png",
        Some((500, 500, 1500, 1500)),
        MatchMode::Segmented,
        "socket1",
    )
    .unwrap();
    loop {
        let start = Instant::now();
        if let Some(t) = ag.find_stored_image_on_screen(0.6, "socket1").unwrap() {
            println!(
                "found at {} {} frametime {}ms",
                t[0].0,
                t[0].1,
                Instant::now().duration_since(start).as_millis()
            );
        }
    }
}
