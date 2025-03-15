// use rustautogui;

fn main() {

    // initialize autogui
    let mut gui = rustautogui::RustAutoGui::new(false).unwrap();

    // displays (x, y) size of screen
    // especially useful on linux where screen from all monitors is grabbed
    gui.get_screen_size();
    
    // load the image searching for. Region is Option<(startx, starty, width, height)> of search. Matchmode FFT or Segmented (not implemented before 1.0 version), max segments, only important for Segmented match mode
    gui.load_and_prepare_template("test.png", Some((0,0, 500, 300)), rustautogui::MatchMode::FFT, &None).unwrap();

    // or segmented variant with no region
    gui.load_and_prepare_template("test.png", None, rustautogui::MatchMode::FFT, &Some(5000)).unwrap();
    
    // change prepare template settings, like region, matchmode or max segments

    gui.change_prepared_settings(Some((200, 100, 1000, 500)), rustautogui::MatchMode::FFT, &None);

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
    gui.scroll_down().unwrap();
    gui.scroll_up().unwrap();
    gui.scroll_right().unwrap();
    gui.scroll_left().unwrap();


    // input keyboard string
    gui.keyboard_input("test.com", &false).unwrap();

    // press a keyboard command, in this case enter(return)
    gui.keyboard_command("return").unwrap();




    // maybe you would want to loop search until image is found and break the loop then
    loop {
        let pos = gui.find_image_on_screen_and_move_mouse(0.9, 1.0).unwrap();    
        match pos {
            Some(_) => break,
            None => (),
        }
    }
}