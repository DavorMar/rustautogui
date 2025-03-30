# RustAutoGUI

RustAutoGUI crate, made after Al Sweigarts library PyAutoG for python.

RustAutoGUI allows you to control the mouse and keyboard to automate interactions with other applications.
The crate works on Windows, Linux and Macos.

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
- and more




## Achievable speed
Unlike PyAutoGUI, this library does not use OpenCV for template matching. Instead, it employs a custom multithreaded implementation, though it lacks GPU acceleration. While OpenCV's template matching is highly optimized, RustAutoGui aims to provide faster overall performance by optimizing the entire process of screen capture, processing, and image matching. From tests so far, the performance appears to be ~5x faster than python counterpart. The speed will also vary between operating systems, where Windows outperforms Linux for instance. 


Gif presentation (intentionally captured with phone camera): 

![](testspeed.gif)


### Why not OpenCV?

OpenCV requires complex dependencies and a lengthy setup process in Rust. To keep installation simple and avoid forcing users to spend hours setting up dependencies, RustAutoGui features a fully custom template matching algorithm that minimizes computations while achieving high accuracy.

### Segmented template matching algorithm

Since version 1.0.0, RustAutoGUI crate includes another variation of template matching algorithm using Segmented Normalized Cross-Correlation.
More information: https://arxiv.org/pdf/2502.01286


## Installation

Either run
`cargo add rustautogui`

or add the crate in your Cargo.toml:

`rustautogui = "2.3.0"`

For Linux additionally run:

`sudo apt-get update`

`sudo apt-get install libx11-dev libxtst-dev`


For macOS: grant necessary permissions in your settings.


# Usage:

Since version 2.2.0, RustAutoGUI supports loading multiple images in memory and searching them. It also allows loading images from memory instead of only from disk.

Loading an image does certain precalculations required for the template matching process, which allows faster execution of the process itself requiring less computations.

### Import and Initialize RustAutoGui
```rust
use rustautogui;

let mut rustautogui = rustautogui::RustAutoGui::new(false); // arg: debug
```

## Loading single image into memory

From file, same as load_and_prepare_template which will be deprecated
```rust
rustautogui.prepare_template_from_file( // returns Result<(), String>
   "template.png", // template_path: &str path to the image file on disk
   Some((0,0,1000,1000)), // region: Option<(u32, u32, u32, u32)>  region of monitor to search (x, y, width, height)
   rustautogui::MatchMode::Segmented, // match_mode: rustautogui::MatchMode search mode (Segmented or FFT)
).unwrap();
```
From ImageBuffer<RGB/RGBA/Luma<u8>>
```rust
rustautogui.prepare_template_from_imagebuffer( // returns Result<(), String>
   img_buffer, // image:  ImageBuffer<P, Vec<T>> -- Accepts RGB/RGBA/Luma(black and white)
   None,  // region: Option<(u32, u32, u32, u32)> searches whole screen when None
   rustautogui::MatchMode::FFT, // match_mode: rustautogui::MatchMode search mode (Segmented or FFT)
).unwrap();
```
From raw bytes of encoded image
```rust
rustautogui.prepare_template_from_raw_encoded( // returns Result<(), String>
   img_raw // img_raw:  &[u8] - encoded raw bytes
   None,  // region: Option<(u32, u32, u32, u32)>
   rustautogui::MatchMode::FFT, // match_mode: rustautogui::MatchMode search mode (Segmented or FFT)
).unwrap();

```


## Segmented vs FFT matching
It is hard to give a 100% correct answer when to use which algorithm. FFT algorithm is mostly consistent, with no big variances in speed. Segmented on other hand can heavily vary and speed can be up to 10x faster than FFT, but also slower by factor of up to thousands. The best method would be for users to test both methods and determine when to use which method. A general advice can be: Use segmented on smaller template images and when template is less visually complex (visual complexity is randomness of pixels in an image, for instance an image that is half white vs half black vs random noise image). 
FFT would probably be better when comparing large template images on a large region, but also when template size approaches image region size. 


Generally, if you're following the idea of maximizing speeds by using as small as possible template images and determining small as possible screen regions, in most cases Segmented will perform faster than FFT. 

Matchmodes enum:
```rust
pub enum MatchMode {
   Segmented,
   FFT,
}
```


## Loading multiple images into memory

Functions  work the same as single image loads, with additional parameter of alias for the image.

Load from file
```rust
rustautogui.store_template_from_file( // returns Result<(), String>
   "template.png", // template_path: &str path to the image file on disk
   Some((0,0,1000,1000)), // region: Option<(u32, u32, u32, u32)>  region of monitor to search (x, y, width, height)
   rustautogui::MatchMode::Segmented, // match_mode: rustautogui::MatchMode search mode (Segmented or FFT)
   "button_image" // alias: &str. Keyword used to select which image to search for
).unwrap();
```
Load from Imagebuffer
```rust
rustautogui.store_template_from_imagebuffer( // returns Result<(), String>
   img_buffer, // image:  ImageBuffer<P, Vec<T>> -- Accepts RGB/RGBA/Luma(black and white)
   None,  // region: Option<(u32, u32, u32, u32)>
   rustautogui::MatchMode::Segmented, // match_mode: rustautogui::MatchMode search mode (Segmented or FFT)
   "button_image" // alias: &str. Keyword used to select which image to search for
).unwrap();
```
Load from encoded raw bytes
```rust
rustautogui.store_template_from_raw_encoded( // returns Result<(), String>
   img_raw // img_raw:  &[u8] encoded raw bytes
   None,  // region: Option<(u32, u32, u32, u32)>
   rustautogui::MatchMode::Segmented, // match_mode: rustautogui::MatchMode search mode (Segmented or FFT)
   "button_image" // alias: &str. Keyword used to select which image to search for
).unwrap();

```


## Single loaded template search

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
IMPORTANT: Difference between linux and windows/macOS when using multiple monitors. On Windows and macOS, search for template image can be done only on the main monitor. On Linux, searches can be done on all monitors if multiple are used, with (0,0) starting from the top-left monitor.

Loop search with timeout. Searches till image is found or timeout in seconds is hit.
<br><strong> Warning: timeout of 0 initiates infinite loop</strong>
```rust
rustautogui
        .loop_find_image_on_screen(0.95, 15) // args: precision, timeout
        .unwrap();
```

```rust
rustautogui
        .loop_find_image_on_screen_and_move_mouse(0.95, 1.0, 15) // args: precision, moving_time and timeout
        .unwrap();
```

## Multiple stored templates search

Again, functions are the same, just having alias argument

```rust
rustautogui
      .find_stored_image_on_screen(0.9,  "test2") // precision, alias
      .unwrap();
```
With mouse movement to location
```rust
rustautogui
      .find_stored_image_on_screen_and_move_mouse(0.9, 1.0, "test2") // precision, moving_time, alias (&str)
      .unwrap();
```
Loop search
<br><strong> Warning: timeout of 0 initiates infinite loop</strong>

```rust
rustautogui
        .loop_find_stored_image_on_screen(0.95, 15, "stars") // precision, timeout, alias
        .unwrap();
```

```rust
rustautogui
        .loop_find_stored_image_on_screen_and_move_mouse(0.95, 1.0, 15, "stars") // precision, moving_time, timeout, alias
        .unwrap();
```

### MacOS retina display issues:
Macos retina display functions by digitally doubling the amount of displayed pixels. The original screen size registered by OS is,
for instance, 1400x800. Retina display doubles it to 2800x1600. If a user provides a screengrab, the image will be saved with doubled the amount
of pixels, where it then fails to match template since screen provided by OS api is not doubled. It can also not be known if user is providing template from a screen grab, or an image thats coming from some other source. For that reason, every template is saved in its original format,
and also resized by half. The template search first searches for resized template, and if it fails then it tries with original. For that reason, users on macOS will experience slower search times than users on other operating systems.


## General functions
Debug mode prints out number of segments in segmented picture, times taken for algorithm run and it saves segmented images. It also creates debug folder in code root, where the images are saved.

Warnings give useful information which shouldn't pop up frequently
```rust
rustautogui.get_screen_size(); // returns (x, y) size of display
rustautogui.change_debug_state(true); // change debugging
rustautogui.set_suppress_warning(true); // turn off warnings
rustautogui.save_screenshot("test.png").unwrap(); //saves screen screenshot
```

## Mouse functions
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
use rustautogui::print_mouse_position;
fn main() {
   print_mouse_position().unwrap();
}
```

## Keyboard functions
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

- On Windows, RustAutoGUI interacts with winapi
- on Linux, it uses x11, and Wayland is not supported
- on macOS, it uses core-graphics crate


## Major changes:

- 1.0.0 - introduces segmented match mode
- 2.0.0 - removed most of panics and crashes
- 2.1.0 - fixed on keyboard, some methods arguments / returns changed and will cause code breaking.
- 2.2.0 - loading multiple images, loading images from memory



## Additional notes
Data stored in prepared template data
```rust
pub enum PreparedData {
    Segmented(
        (
            Vec<(u32, u32, u32, u32, f32)>, // template_segments_fast
            Vec<(u32, u32, u32, u32, f32)>, // template_segments_slow
            u32,                            // template_width
            u32,                            // template_height
            f32,                            // segment_sum_squared_deviations_fast
            f32,                            // segment_sum_squared_deviations_slow
            f32,                            // expected_corr_fast
            f32,                            // expected_corr_slow
            f32,                            // segments_mean_fast
            f32,                            // segments_mean_slow
        ),
    ),
    FFT(
        (
            Vec<Complex<f32>>, // template_conj_freq
            f32,               // template_sum_squared_deviations
            u32,               // template_width
            u32,               // template_height
            u32,               // padded_size
        ),
    ),

    None,
}
```
