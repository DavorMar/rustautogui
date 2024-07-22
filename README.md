# Rustautogui

Rustautogui crate, made after Al Sweigarts library Pyautogui for python. 

Rustautogui allows you to control the mouse and keyboard to automate interactions with other applications. 
Currently the crate works on Windows and Linux. 

Main functions:

- capture screen
- find image on screen
- move mouse to pixel coordinate
- click mouse button
- input keyboard string
- input keyboard command
- find image on screen and move mouse to it
- detect cursor position


Note: this library does not use OpenCV template matching, like python version does. OpenCV have fully optimised template matching, while here multithreading is used but no GPU acceleration yet. For that reason, finding image on screen may be slower than the python counterpart using OpenCV, but the speed is still satisfying. Once Segmented correlation algorithm is included, speed will be further increased. The goal would be to include also GPU acceleration over time.

The reason for not including OpenCV in this library is because installation of all the bindings and dependencies for rust can be a tiresome and a long process. I did not want to shape the library with prerequisite of user spending multiple hours pre installing everything needed. For this reason, template maching has been completely self developed.  

# Segmented template matching algorithm

Rustautogui crate will include a new variation of template matching algorithm using cross correlation, which is not implemented yet. Currently, a paper has been written on the new algorithm and when it is submitted to arxiv it will be release in this library

For this reason, please follow the updates. Once 1.0 version is released, it will contain new algorithm. Currently, everything code wise is prepared for new algorithm, and code for 1.0 version exists. 
For this reason, if choosing Segmented match mode at this moment, you will get a panic. 

## Installation

Either run 
`cargo add rustautogui`

or add the crate in your Cargo.toml file like:

`rustautogui = "0.1.6"`

For Linux additionally install run :

`sudo apt-get update`

`sudo apt-get install libx11-dev libxtst-dev`


## Usage:

```rust
use rustautogui::RustAutoGui

let mut rustautogui = RustAutoGui::new(debug:false); // create rustautogui instance

rustautogui.load_and_prepare_template(
                               template_path: "template.png",
                               region: Some(0,0,1000,1000),
                               rustautogui::MatchMode::FFT,
                               max_segments:&None
                           ); // load a template image for screen matching
```
Arguments are template path, Option<(x,y,width,height)> region parameter. Option is used because it can be None meaning it wil search over whole screen. Matchmode can be MatchMode::Segmented(once implemented) or MatchMode::FFT.
FFT is used in fourier transformation and it may be better if size of template is pretty large. If using smaller template, Segmented matchmode will be preffered and much faster. 
max_segments arguments is only used in Segmented matchmode and its not important for FFT match mode, so at this moment its better to be set as &None

If you're looking to maximize speed, your template and region of search should be as small as possible. For instance, if youre looking for image of a button, maybe you dont need a whole button image but just a segment of it. There are various pyautogui tutorials that can explain this. 
For this reason, if already using smaller image to speed up the algorithm, go with segmented match mode which will provide greater speed. 

```rust
rustautogui.change_prepared_settings(
                                region:Some((0,0,1000,1000)),
                                rustautogui::MatchMode::FFT,
                                max_segments:&None
); // change template matching settings, if template is already loaded
```

```rust
rustautogui.find_image_on_screen(precision:0.9); // returns pixel coordinates for prepared template
// on screen. Returns list of coordinates that have correlation higher than inserted precision parameter
//must have prepared template before
```

```rust
rustautogui.find_image_on_screen_and_move_mouse(precision:0.9);
// finds template image on screen and automatically moves mouse
// cursor to the middle of the image. Matches only single
//position with highest correlation value
//must have prepared template before
```
IMPORTANT: Difference between linux and windows when using multiple monitors. On Windows, main monitor starts from coordinates 0, 0 to monitor width and height. Any monitor that is left from it will have X values as negative. Monitor above the main one will have negative Y values. Any monitor right and under of the main monitor will have positive X and Y values, greater than monitor width and height.

On linux, 0, 0 coordinates start from top left position. The leftmost monitor will have zero X coordinate, while top most monitor will have Y zero coordinate. 
```rust
rustautogui.left_click(); // left mouse click
rustautogui.right_click(); // right mouse click
rustautogui.keyboard_input(input: "test", shifted:&false); // input string, or better say, do the sequence of key presses
rustautogui.keyboard_command(input:"return"); //press a keyboard button 
rustautogui.change_debug_state(true); // change debugging
```
Debug mode prints out number of segments in segmented picture, times taken for algorithm run and it saves segmented images.

```rust
rustautogui.save_screenshot("test.png"); //saves screen screenshot
```

```rust
use rustautogui::mouse;
fn main() {
   mouse::mouse_position::show_mouse_position_window();
}
```
This is assisting tool that pops up a window that shows mouse coordinates. This is a good utility to determine matching regions when developing. 

## How does crate work:

On windows api interacts with winapi, through usage of winapi crate, while on linux it interacts with x11 api through usage of x11 crate.
When RustAutoGui instance is created with ::new function, the Screen, Mouse and Keyboard structs are also initialized and stored under RustAutoGui struct.
Screen struct preallocates memory segment for screen image storage. 

Executing find_image_on_screen_function does cross correlation of template and screen. The thing is, cross correlation consists of certain steps that can be precalculated and shouldnt be part of the main correlation process, so it does not slow it down.

For this reason, function load_and_prepare_template is created, so you can preload template image and do the precalculations on it before doing correlation with find_image_on_screen.
That is why, you should preinitialize template at some moment where speed is not crucial in your code, so it is ready for faster template matching when needed. 




