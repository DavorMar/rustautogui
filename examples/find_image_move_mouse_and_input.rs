fn main() {
    #[cfg(not(feature = "lite"))]
    {
        use rustautogui;
        use rustautogui::imgtools;
        // initialize autogui
        let mut gui = rustautogui::RustAutoGui::new(false).unwrap();

        // displays (x, y) size of screen
        // especially useful on linux where screen from all monitors is grabbed
        gui.get_screen_size();

        {
            // load the image searching for. Region is Option<(startx, starty, width, height)> of search. Matchmode FFT or Segmented (not implemented before 1.0 version), max segments, only important for Segmented match mode
            gui.prepare_template_from_file(
                "test.png",
                Some((0, 0, 500, 300)),
                rustautogui::MatchMode::FFT,
            )
            .unwrap();
        }
        // or another way to prepare template
        {
            let img = imgtools::load_image_rgba("test.png").unwrap(); // just loading this way for example
            gui.prepare_template_from_imagebuffer(
                img,
                Some((0, 0, 700, 500)),
                rustautogui::MatchMode::Segmented,
            )
            .unwrap();
        }

        // or segmented variant with no region
        gui.prepare_template_from_file("test.png", None, rustautogui::MatchMode::FFT)
            .unwrap();

        // change prepare template settings, like region, matchmode or max segments

        // automatically move mouse to found template position, execute movement for 1 second
        gui.find_image_on_screen_and_move_mouse(0.9, 1.0).unwrap();

        //move mouse to position (in this case to bottom left of stanard 1920x1080 monitor), move the mouse for 1 second
        gui.move_mouse_to_pos(1920, 1080, 1.0).unwrap();

        // execute left click, move the mouse to x (500), y (500) position for 1 second, release mouse click
        // used for actions like moving icons or files
        // suggestion: dont use very small moving time values, especially on mac
        gui.drag_mouse(500, 500, 1.0).unwrap();

        // execute mouse left click
        gui.left_click().unwrap();

        // execute mouse right click
        gui.right_click().unwrap();

        // execute mouse middle click
        gui.middle_click().unwrap();

        // execute a double (left) mouse click
        gui.double_click().unwrap();

        // mouse scrolls
        gui.scroll_down(1).unwrap();
        gui.scroll_up(5).unwrap();
        gui.scroll_right(8).unwrap();
        gui.scroll_left(10).unwrap();

        // input keyboard string
        gui.keyboard_input("test.com").unwrap();

        // press a keyboard command, in this case enter(return)
        gui.keyboard_command("return").unwrap();

        // two key press
        gui.keyboard_multi_key("alt", "tab", None).unwrap();

        // three key press
        gui.keyboard_multi_key("shift", "control", Some("e"))
            .unwrap();

        // maybe you would want to loop search until image is found and break the loop then
        loop {
            let pos = gui.find_image_on_screen_and_move_mouse(0.9, 1.0).unwrap();
            match pos {
                Some(_) => break,
                None => (),
            }
        }
    }
}
