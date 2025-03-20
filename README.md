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

`rustautogui = "2.2.0"`

For Linux additionally install run :

`sudo apt-get update`

`sudo apt-get install libx11-dev libxtst-dev`


For macOS: dont forget to give necessary permissions


## Usage:

Since version 2.2.0, RustAutoGui supports loading multiple images in memory and searching for them. Additionally, loading images from memory has been implemented, where previously only loading from disk path was possible. 

Loading an image does certain precalculations required for the template matching process, which allows faster execution of the process itself requiring less computations. 

### Importing rustautogui and creating RustAutoGui instance
```rust
use rustautogui;

let mut rustautogui = rustautogui::RustAutoGui::new(false); // arg: debug
```

### Loading single image into memory

From file, same as load_and_prepare_template which will be deprecated
```rust
rustautogui.prepare_template_from_file( // returns Result<(), String>
   "template.png", // template_path: &str path to the image file on disk 
   Some((0,0,1000,1000)), // region: Option<(u32, u32, u32, u32)>  region of monitor to search (x, y, width, height)
   rustautogui::MatchMode::Segmented, // match_mode: rustautogui::MatchMode search mode (Segmented or FFT)
   &Some(3000) // max_segments: Option<u32> max segments for Segmented match mode. Can be set to None to automatically calculate
).unwrap(); 
```
From ImageBuffer<RGB/RGBA/Luma>
```rust
rustautogui.prepare_template_from_imagebuffer( // returns Result<(), String>
   img_buffer, // image:  ImageBuffer<P, Vec<T>> -- Accepts RGB/RGBA/Luma(black and white)
   None,  // region: Option<(u32, u32, u32, u32)> searches whole screen when None
   rustautogui::MatchMode::FFT, // match_mode: rustautogui::MatchMode search mode (Segmented or FFT)  
   None, // max_segments: Option<u32> no need for segments when doing FFT search
).unwrap();
```
From raw bytes of encoded image
```rust
rustautogui.prepare_template_from_raw_encoded( // returns Result<(), String>
   img_raw // img_raw:  &[u8] - encoded raw bytes
   None,  // region: Option<(u32, u32, u32, u32)> 
   rustautogui::MatchMode::FFT, // match_mode: rustautogui::MatchMode search mode (Segmented or FFT)  
   None, // max_segments: Option<u32> 
).unwrap();

```

max_segments arguments is only used in Segmented matchmode and its not important for FFT match mode, when you want to set it as &None.
If using Segmented match mode, max_segments can influence speed and precision. If using large and complex pictures(with high pixel variation), max segments determines maximum number of segments to divide picture into, sacrificing precision while increasing speed. The higher the number of segments, lower the speed but higher precision.
The default value is set to 30% of template pixel count. If you want to increase the speed, reduce the max segments by some value and follow the correlation value with debug mode. 

If you're looking to maximize speed, your template and region of search should be as small as possible. For instance, if youre looking for image of a button, maybe you dont need a whole button image but just a segment of it. There are various pyautogui tutorials that can explain this. 
For this reason, if already using smaller image to speed up the algorithm, try using segmented match mode which will provide greater speed. If segmented mode is unfit for a certain template, warning will be displayed and switching to FFT is recommended. 

Additionally, template settings can be changed
 
```rust
// change template matching settings, if template is already loaded (does not work for stored images with alias)
rustautogui.change_prepared_settings(
            Some((0,0,1000,1000)), //region: Option<(u32, u32, u32, u32)>
            rustautogui::MatchMode::Segmented, // match_mode: rustautogui::MatchMode search mode (Segmented or FFT)  
            &None // max_segments: Option<u32>
);

```




### Loading multiple images into memory

Functions  work the same as single image loads, with additional parameter of alias for the image. 

Load from file
```rust
rustautogui.load_and_prepare_template( // returns Result<(), String>
   "template.png", // template_path: &str path to the image file on disk 
   Some((0,0,1000,1000)), // region: Option<(u32, u32, u32, u32)>  region of monitor to search (x, y, width, height)
   rustautogui::MatchMode::Segmented, // match_mode: rustautogui::MatchMode search mode (Segmented or FFT)
   &Some(3000) // max_segments: Option<u32> max segments for Segmented match mode. Can be set to None to automatically calculate
).unwrap(); 
```
Load from Imagebuffer
```rust
rustautogui.store_template_from_imagebuffer( // returns Result<(), String>
   img_buffer, // image:  ImageBuffer<P, Vec<T>> -- Accepts RGB/RGBA/Luma(black and white)
   None,  // region: Option<(u32, u32, u32, u32)> 
   rustautogui::MatchMode::FFT, // match_mode: rustautogui::MatchMode search mode (Segmented or FFT)  
   None, // max_segments: Option<u32> 
   "button_image".to_string() // alias: String. Keyword used to select which image to search for
).unwrap();
```
Load from encoded raw bytes
```rust
rustautogui.store_template_from_raw_encoded( // returns Result<(), String>
   img_raw // img_raw:  &[u8] encoded raw bytes
   None,  // region: Option<(u32, u32, u32, u32)> 
   rustautogui::MatchMode::FFT, // match_mode: rustautogui::MatchMode search mode (Segmented or FFT)  
   None, // max_segments: Option<u32> 
   "button_image".to_string() // alias: String. Keyword used to select which image to search for
).unwrap();

```


### Single loaded template search

Find image and get pixel coordinates
```rust
let found_locations: Option<Vec<(u32, u32, f64)>> = rustautogui.find_image_on_screen(0.9).unwrap(); // arg: precision
// returns pixel locations for prepared template that have correlation higher than precision, ordered from highest correlation to lowest
// Must have prepared template before
```
Find image, get pixel coordinates and move mouse to location
```rust
let found_locations: Option<Vec<(u32, u32, f64)>> =  rustautogui.find_image_on_screen_and_move_mouse(0.9, 1.0).unwrap();
// args: precision , moving_time
// executes find_image_on_screen() and moves mouse to the center of the highest correlation location
```
IMPORTANT: Difference between linux and windows/macOS when using multiple monitors. On Windows and macOS, search for template image can be done only on the main monitor.
On linux, search can be done on all monitors if multiple are used, and  0, 0 coordinates start from top left position. The leftmost monitor will have zero X coordinate, while top most monitor will have Y zero coordinate. 

### Multiple stored templates search

Again, functions are the same, just having alias argument

```rust
rustautogui
        .find_stored_image_on_screen(0.9,  &"test2".to_string()) // precision, alias
        .unwrap();
```
With mouse movement to location
```rust
rustautogui
      .find_stored_image_on_screen_and_move_mouse(0.9, 1.0, &"test2".to_string()) // precision, move time, alias (&String)
      .unwrap();
```


### General functions
Debug mode prints out number of segments in segmented picture, times taken for algorithm run and it saves segmented images. It also creates debug folder in code root, where the images are saved. 

Warnings give useful information which shouldnt pop up frequently
```rust
rustautogui.get_screen_size(); // returns (x, y) size of display
rustautogui.change_debug_state(true); // change debugging
rustautogui.set_suppress_warning(true); // turn off warnings
rustautogui.save_screenshot("test.png").unwrap(); //saves screen screenshot
```

### Mouse functions 
```rust
rustautogui.left_click().unwrap(); // left mouse click
rustautogui.right_click().unwrap(); // right mouse click
rustautogui.double_click().unwrap(); // double left click
rustautogui.middle_click().unwrap(); // double left click
rustautogui.scroll_up().unwrap();
rustautogui.scroll_down().unwrap();
rustautogui.scroll_left().unwrap();
rustautogui.scroll_right().unwrap();
rustautogui.move_mouse_to_pos(1920, 1080, 1.0).unwrap(); // args: x, y, moving_time. Moves mouse to position for certain time
rustautogui.drag_mouse(500, 500, 1.0).unwrap(); // executes left click down, move mouse_to_pos x, y location, left click up. 
//note: use moving time > 0.2, or even higher, depending on distance. Especially important for macOS
```

Below is a helper function to determine coordinates on screen, helpful when determining region or mouse move target when developing
- Before 0.3.0 this function popped up window, now it just prints. This was changed to reduce dependencies.
```rust
use rustautogui::mouse;
fn main() {
   mouse::mouse_position::print_mouse_position().unwrap();
}
```

### Keyboard functions
Currently, only US keyboard is implemented. If you have different layout active, lots of characters will not work correctly
```rust
rustautogui.keyboard_input("test!@#24").unwrap(); // input string, or better say, do the sequence of key presses
rustautogui.keyboard_command("backspace").unwrap(); // press a keyboard button 
rustautogui.keyboard_multi_key("shift", "control", Some("t")).unwrap(); // Executed multiple key press at same time. third argument is optional
```


For all the keyboard commands check Keyboard_commands.md, a table of possible keyboard inputs/commands for each OS. If you 
find some keyboard commands missing that you need, please open an issue in order to get it added in next versions. 









## Warnings options:

Rustautogui may display some warnings. In case you want to turn them off, either run:\
Windows powershell:
```
   $env:RUSTAUTOGUI_SUPPRESS_WARNINGS="1"    #to turn off warnings
   $env:RUSTAUTOGUI_SUPPRESS_WARNINGS="0"    #to activate warnings
```
Windows CMD:
```
   set RUSTAUTOGUI_SUPPRESS_WARNINGS=1       #to turn off warnings
   set RUSTAUTOGUI_SUPPRESS_WARNINGS=0       #to activate warnings
```
Linux/MacOS: 
```
   export RUSTAUTOGUI_SUPPRESS_WARNINGS=1    #to turn off warnings
   export RUSTAUTOGUI_SUPPRESS_WARNINGS=0    #to activate warnings
```
or in code: 

```rust
let mut rustautogui = RustAutoGui::new(false).unwrap();
rustautogui.set_suppress_warnings(true);
```

## How does crate work:

On windows, api interacts with winapi, through usage of winapi crate, on linux it interacts with x11 api through usage of x11 crate while on macOS it interacts through usage of core-graphics crate. 
Take note, on Linux there is no support for wayland. If you encounter errors on grabbing XImage or similar, the reason is most likely wayland. 
When RustAutoGui instance is created with ::new function, the Screen, Mouse and Keyboard structs are also initialized and stored under RustAutoGui struct.
Screen struct preallocates memory segment for screen image storage. 

Executing find_image_on_screen_function does cross correlation of template and screen. Since cross correlation consists of certain steps that can be precalculated, they shouldnt be part of the main correlation process, so it does not get slowed down.

For this reason, function load_and_prepare_template is created, so template image can be preloaded and precalculations can be done on it before doing correlation with find_image_on_screen.
That is why, you should preinitialize template at some moment where speed is not crucial in your code, so it is ready for faster template matching when needed. 

### Major changes: 

- 1.0.0 - introduces segmented match mode
- 2.0.0 - removed most of ungraceful exits
- 2.1.0 - fixed on keyboard, some methods arguments / returns changed and will cause code breaking. 
- 2.2.0 - loading multiple images, loading images from memory


