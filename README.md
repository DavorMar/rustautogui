# Rustautogui

Rustautogui crate, made after Al Sweigarts library Pyautogui for python. 

Rustautogui allows you to control the mouse and keyboard to automate interactions with other applications. 
The crate works on Windows and Linux, and since version 0.2.0 on macOS.

Main functions:

- capture screen
- find image on screen
- move mouse to pixel coordinate
- click mouse buttons
- input keyboard string
- input keyboard command
- keyboard multiple key press
- find image on screen and move mouse to it
- detect cursor position


Note: this library does not use OpenCV template matching, like python version does. OpenCV has fully optimised template matching, while here multithreading is used but no GPU acceleration or other optimisations yet. For that reason, the speed of the algorithm itself is somewhat slower than OpenCVs template matching, but the whole process of grabbing monitor image, processing it and finding image on it is (or should be in most scenarios) much faster than python counterpart.

The reason for not including OpenCV in this library is because installation of all the bindings and dependencies for rust can be a tiresome and a long process. Shaping the library with prerequisite of user spending multiple hours pre installing everything needed was not the goal. For this reason, template maching has been completely self developed, with algorithms that require less computations to achieve the result.  

## Segmented template matching algorithm

Since version 1.0.0, Rustautogui crate includes another variation of template matching algorithm using Segmented Normalized Cross-Correlation. 
More information: https://arxiv.org/pdf/2502.01286


## Installation

Either run 
`cargo add rustautogui`

or add the crate in your Cargo.toml file like:

`rustautogui = "2.1.1"`

For Linux additionally install run :

`sudo apt-get update`

`sudo apt-get install libx11-dev libxtst-dev`


For macOS: dont forget to give necessary permissions


## Usage:

```rust
use rustautogui;


let mut rustautogui = rustautogui::RustAutoGui::new(debug:false); // create rustautogui instance

rustautogui.load_and_prepare_template(
                               template_path: "template.png",
                               region: Some((0,0,1000,1000)),
                               match_mode: rustautogui::MatchMode::Segmented,
                               max_segments: &Some(3000)
                           ).unwrap(); // load a template image for screen matching
```
Arguments are template path, Option<(x,y,width,height)> region parameter. Option is used because it can be None meaning it will search over whole screen. Matchmode can be MatchMode::Segmented or MatchMode::FFT.
FFT is used in fourier transformation and it may be better if size of template is pretty large. If using smaller template, Segmented matchmode will be preffered and much faster. 
max_segments arguments is only used in Segmented matchmode and its not important for FFT match mode, when you want to set it as &None.
If using Segmented match mode, max_segments can influence speed and precision. If using large and complex pictures(with high pixel variation), max segments determines maximum number of segments
to divide picture into, sacrificing precision while increasing speed. The higher the number of segments, lower the speed but higher precision.
The default value is set to 10000, but can be increased or decreased depending on template. If you want to increase the speed, reduce the max segments by some value and follow the correlation value with debug mode. 

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
let found_locations: Option<Vec<(u32, u32, f64)>> = rustautogui.find_image_on_screen(precision:0.9).unwrap(); // returns pixel coordinates for prepared template
// on screen. Returns list of coordinates that have correlation higher than inserted precision parameter
// must have prepared template before
// returns locations that have correlation higher than precision, ordered from highest to lowest. 
// Mouse moves to highest correlation point
```

```rust
let found_locations: Option<Vec<(u32, u32, f64)>> =  rustautogui.find_image_on_screen_and_move_mouse(precision:0.9, moving_time:1.0).unwrap();
// finds template image on screen and automatically moves mouse
// cursor to the middle of the image. Matches only single
// position with highest correlation value
// must have prepared template before
// returns locations that have correlation higher than precision, ordered from highest to lowest. 
// Mouse moves to highest correlation point
```
IMPORTANT: Difference between linux and windows/macOS when using multiple monitors. On Windows and macOS, search for template image can be done only on the main monitor.
On linux, search can be done on all monitors and  0, 0 coordinates start from top left position. The leftmost monitor will have zero X coordinate, while top most monitor will have Y zero coordinate. 


```rust
rustautogui.get_screen_size(); // returns (x, y) size of display
rustautogui.left_click().unwrap(); // left mouse click
rustautogui.right_click().unwrap(); // right mouse click
rustautogui.double_click().unwrap(); // double left click
rustautogui.keyboard_input(input: "test!@#24").unwrap(); // input string, or better say, do the sequence of key presses
rustautogui.keyboard_command(input:"return").unwrap(); //press a keyboard button 
rustautogui.keyboard_multi_key(input1: "shift", input2:"control", input3: Some("t")).unwrap(); // Executed multiple key press at same time. third argument is optional
rustautogui.change_debug_state(true); // change debugging
rustautogui.scroll_up().unwrap();
rustautogui.scroll_down().unwrap();
rustautogui.scroll_left().unwrap();
rustautogui.scroll_right().unwrap();
rustautogui.drag_mouse(x: 500, y:500, moving_time: 1.0).unwrap(); // executes left click down, move mouse to x, y location, left click up. 
                                                                  //note: use moving time > 0.2
```
For all the keyboard commands check Keyboard_commands.txt, a roughly written list of possible inputs. If you 
find some keyboard commands missing that you need, please open an issue in order to get it added in next versions. 

Debug mode prints out number of segments in segmented picture, times taken for algorithm run and it saves segmented images. It also creates debug folder in code root, where the images are saved. 

```rust
rustautogui.save_screenshot("test.png").unwrap(); //saves screen screenshot
```

```rust
use rustautogui::mouse;
fn main() {
   mouse::mouse_position::print_mouse_position().unwrap();
}
```
Before 0.3.0 this function popped up window, now it just prints. This was changed to reduce dependencies.
This is a helper function to determine coordinates on screen, helpful when determining region or mouse move target. 


## Warnings options:

Rustautogui may display some warnings. In case you want to turn them off, either run:\
Windows powershell:\
```
   $env:RUSTAUTOGUI_SUPPRESS_WARNINGS="1"    #to turn off warnings\
   $env:RUSTAUTOGUI_SUPPRESS_WARNINGS="0"    #to activate warnings\
```
Windows CMD:\
```
   set RUSTAUTOGUI_SUPPRESS_WARNINGS=1       #to turn off warnings\
   set RUSTAUTOGUI_SUPPRESS_WARNINGS=0       #to activate warnings\
```
Linux/MacOS: \
```
   export RUSTAUTOGUI_SUPPRESS_WARNINGS=1    #to turn off warnings\
   export RUSTAUTOGUI_SUPPRESS_WARNINGS=0    #to activate warnings\
```
or in code: \

```rust
let mut rustautogui = RustAutoGui::new(false).unwrap();
rustautogui.set_suppress_warnings(true);
```

## How does crate work:

On windows, api interacts with winapi, through usage of winapi crate, on linux it interacts with x11 api through usage of x11 crate while on macOS it interacts through usage of core-graphics crate. 
Take note, on Linux there is no support for wayland. If you encounter errors on grabbing XImage or similar, 
the reason is most likely wayland. 
When RustAutoGui instance is created with ::new function, the Screen, Mouse and Keyboard structs are also initialized and stored under RustAutoGui struct.
Screen struct preallocates memory segment for screen image storage. 

Executing find_image_on_screen_function does cross correlation of template and screen. The thing is, cross correlation consists of certain steps that can be precalculated and shouldnt be part of the main correlation process, so it does not slow it down.

For this reason, function load_and_prepare_template is created, so you can preload template image and do the precalculations on it before doing correlation with find_image_on_screen.
That is why, you should preinitialize template at some moment where speed is not crucial in your code, so it is ready for faster template matching when needed. 

### Major changes: 

- 1.0.0 - introduces segmented match mode
- 2.0.0 - removed most of ungraceful exits
- 2.1.0 - fixed on keyboard, some methods arguments / returns changed and will cause code breaking. 


