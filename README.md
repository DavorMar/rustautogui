# RustAutoGUI

RustAutoGUI crate, made after Al Sweigarts library PyAutoGUI for python.

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

## Table of contents

- [RustAutoGUI](#rustautogui)
  - [Table of contents](#table-of-contents)
  - [Achievable speed](#achievable-speed)
    - [Why not OpenCV?](#why-not-opencv)
    - [Segmented template matching algorithm](#segmented-template-matching-algorithm)
  - [Installation](#installation)
    - [Lite version](#lite-version)
- [Usage:](#usage)
    - [Import and Initialize RustAutoGui](#import-and-initialize-rustautogui)
  - [Finding image on screen](#finding-image-on-screen)
    - [Loading images into memory](#loading-images-into-memory)
      - [Loading single image into memory](#loading-single-image-into-memory)
      - [Loading multiple images into memory](#loading-multiple-images-into-memory)
    - [Custom Image Preparation \& Storage (\*\_custom Functions)](#custom-image-preparation--storage-_custom-functions)
      - [What's different?](#whats-different)
      - [Why Use Custom Thresholds?\*\*](#why-use-custom-thresholds)
      - [Performance Tips](#performance-tips)
    - [Template matching](#template-matching)
      - [Single loaded template match](#single-loaded-template-match)
      - [Multiple stored templates search](#multiple-stored-templates-search)
    - [MacOS retina display issues:](#macos-retina-display-issues)
    - [Segmented vs FFT matching](#segmented-vs-fft-matching)
  - [General Functions](#general-functions)
  - [Mouse Functions](#mouse-functions)
    - [Mouse Clicks](#mouse-clicks)
    - [Mouse Scrolls](#mouse-scrolls)
    - [Mouse Movements](#mouse-movements)
    - [Mouse Drags](#mouse-drags)
  - [Keyboard Functions](#keyboard-functions)
  - [Warnings Options](#warnings-options)
- [OpenCL](#opencl)
  - [OpenCL Installation](#opencl-installation)
    - [Linux](#linux)
    - [Windows](#windows)
    - [MacOS](#macos)
  - [Please read before using OpenCL ⚠️⚠️](#please-read-before-using-opencl-️️)
  - [V1 vs V2 Algorithms](#v1-vs-v2-algorithms)
- [Other Info](#other-info)
  - [How does crate work](#how-does-crate-work)
  - [Major changes](#major-changes)
  


## Achievable speed
Unlike PyAutoGUI, this library does not use OpenCV for template matching. Instead, it employs a custom multithreaded implementation.
Since version 2.5, OpenCL is included, with more info further down this Readme file.  While OpenCV's template matching algorithm is highly 
optimized, in some cases Segmented algorithm using OpenCL may be even faster. RustAutoGui aims to provide faster overall performance by 
optimizing the entire process of screen capture, processing, and image matching. From tests so far, the performance appears to be ~5x faster
than python counterpart(on Windows). The speed will also vary between operating systems, where Windows outperforms Linux for instance. 


Gif presentation (intentionally captured with phone camera): 

![](testspeed.gif)


### Why not OpenCV?

OpenCV requires complex dependencies and a lengthy setup process in Rust. To keep installation simple and avoid forcing users to spend hours setting up dependencies, RustAutoGui features a fully custom template matching algorithm that minimizes computations while achieving high accuracy.

### Segmented template matching algorithm

RustAutoGUI crate includes another variation of template matching algorithm using Segmented Normalized Cross-Correlation. 
More information: https://arxiv.org/pdf/2502.01286


## Installation

Either run
`cargo add rustautogui`

or add the crate in your Cargo.toml:

`rustautogui = "2.5.0"`

With OpenCL support ( ⚠️ Please read info below before using):

`rustautogui = { version = "2.5.0", features = ["opencl"] }`

Lite Version 

`rustautogui = { version = "2.5.0", features = ["lite"] }`


For Linux additionally run:

`sudo apt-get update`

`sudo apt-get install libx11-dev libxtst-dev`


For macOS: grant necessary permissions in your settings.

### Lite version
Lite version provides just keyboard and mouse function. No template matching code included

# Usage:

Since version 2.2.0, RustAutoGUI supports loading multiple images in memory and searching them. It also allows loading images from memory instead of only from disk.

Loading an image does certain precalculations required for the template matching process, which allows faster execution of the process itself requiring less computations.

### Import and Initialize RustAutoGui
```rust
use rustautogui;

let mut rustautogui = rustautogui::RustAutoGui::new(false); // arg: debug
```

## Finding image on screen

### Loading images into memory

#### Loading single image into memory
---

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





#### Loading multiple images into memory
---

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

### Custom Image Preparation & Storage (*_custom Functions)
---

All standard image functions have corresponding custom variants, identifiable by the _custom suffix (e.g., prepare_template_from_file_custom, store__template_from_imagebuffer_custom).


#### What's different?
---
These _custom functions include an extra threshold parameter. While the default segmented template matching uses an automatic threshold estimation, the custom version gives you manual control over this value.

- Threshold determines how finely the image is segmented:

  - Higher threshold → Finer segmentation → More detailed image

  - Lower threshold → Coarser segmentation → Faster processing

#### Why Use Custom Thresholds?**
---
The automatic thresholding works well in many cases, but:

- It can introduce a slight performance overhead.

- In some scenarios, manual tuning of the threshold can result in significantly faster matching.

- By choosing the right threshold for each image, you can maximize performance.
  
- threshold is only important for Segmented match modes and has no influence on FFT match mode

💡 Internally, the threshold represents the correlation between the fast-segmented image and the original template.


#### Performance Tips
---
- Threshold > 0.85:

  - May slow down the algorithm significantly.
  - Offers diminishing returns in terms of accuracy or speed.

- Threshold < 0.3:

  - Often produces similar results as 0.0 in most cases.

  - Can be a good baseline for experimentation.
- What do we gain with higher threshold? -> Fast template match produces less false positives, giving less work to slow template match process and increasing speed 


*the algorithm does two correlation checks. First with roughly segmented image, with small number of segments, then on second finer segmented image, with higher precision and more segments. Positions found by rough image, which runs very fast, are checked with finer image. Sometimes, rough image is segmented by too small factor and leads to many false positives, which slows down algorithm due to too many checks on finer image*

Example: 
```rust
rustautogui.store_template_from_imagebuffer_custom( // returns Result<(), String>
   img_buffer,
   None, 
   rustautogui::MatchMode::Segmented,
   "button_image",
   0.0 
).unwrap();
```




### Template matching
#### Single loaded template match
---
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
<br><strong>⚠️ Timeout of 0 initiates infinite loop</strong>
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

#### Multiple stored templates search

---

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
<br><strong>⚠️ Timeout of 0 initiates infinite loop</strong>

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
---
Macos retina display functions by digitally doubling the amount of displayed pixels. The original screen size registered by OS is,
for instance, 1400x800. Retina display doubles it to 2800x1600. If a user provides a screengrab, the image will be saved with doubled the amount
of pixels, where it then fails to match template since screen provided by OS api is not doubled.


It can also not be known if user is providing template from a screen grab, or an image thats coming from some other source. For that reason, every template is saved in its original format,
and also resized by half. The template search first searches for resized template, and if it fails then it tries with original. For that reason, users on macOS will experience slower search times than users on other operating systems.

### Segmented vs FFT matching
---

This info does not include OpenCL in comparison. More info about it below. 


It is hard to give a 100% correct answer when to use which algorithm. FFT algorithm is mostly consistent, with no big variances in speed. Segmented on other hand can heavily vary and speed can be up to 10x faster than FFT, but also slower by factor of up to thousands. The best would be for users to test both methods and determine when to use which method. A general advice can be: Use segmented on smaller template images and when template is less visually complex (visual complexity is randomness of pixels in an image, for instance an image that is half white vs half black vs random noise image). 
FFT would probably be better when comparing large template images on a large region, but also when template size approaches image region size. 


Generally, if you're following the idea of maximizing speeds by using as small as possible template images and determining small as possible screen regions, in most cases Segmented will perform faster than FFT. 

Matchmodes enum:
```rust
pub enum MatchMode {
    Segmented,
    FFT,
    SegmentedOcl, // Only with opencl feature enabled
    SegmentedOclV2, // Only with opencl feature enabled
}
```


## General Functions
Debug mode prints out number of segments in segmented picture, times taken for algorithm run and it saves segmented images. It also creates debug folder in code root, where the images are saved.

Warnings give useful information which shouldn't pop up frequently
```rust
rustautogui.get_screen_size(); // returns (x, y) size of display
rustautogui.change_debug_state(true); // change debugging
rustautogui.set_suppress_warning(true); // turn off warnings
rustautogui.save_screenshot("test.png").unwrap(); //saves screen screenshot
```

## Mouse Functions

MouseClick enum used in some functions
```rust
pub enum MouseClick {
    LEFT,
    RIGHT,
    MIDDLE,
}
```
Get current mouse position
```rust
rustautogui.get_mouse_position().unwrap(); // returns (x,y) coordinate of mouse

```
### Mouse Clicks
Mouse clicks functions. Mouse up and down work only on Windows / Linux.
```rust
rustautogui.click(MouseClick::LEFT).unwrap(); // args: button,  choose  click button MouseClick::{LEFT, RIGHT, MIDDLE}
rustautogui.left_click().unwrap(); // left mouse click
rustautogui.right_click().unwrap(); // right mouse click
rustautogui.double_click().unwrap(); // double left click
rustautogui.middle_click().unwrap(); // double left click

// mouse up and mouse down work only on Windows and Linux
rustautogui.mouse_down(MouseClick::RIGHT).unwrap(); // args: button, click button down,  MouseClick::{LEFT, RIGHT, MIDDLE}
rustautogui.mouse_up(MouseClick::RIGHT).unwrap(); // args: button,  click button up MouseClick::{LEFT, RIGHT, MIDDLE}

```
### Mouse Scrolls

```rust
rustautogui.scroll_up().unwrap();
rustautogui.scroll_down().unwrap();
rustautogui.scroll_left().unwrap();
rustautogui.scroll_right().unwrap();
```

### Mouse Movements

```rust
rustautogui.move_mouse_to_pos(1920, 1080, 1.0).unwrap(); // args: x, y, moving_time. Moves mouse to position for certain time
rustautogui.move_mouse_to(Some(500), None, 1.0).unwrap(); // args: x, y, moving_time. Moves mouse to position, but acceps Option
//                                                                                    None Value keeps same position
rustautogui.move_mouse(-50, 120, 1.0).unwrap(); //  args: x, y, moving_time. Moves mouse relative to its current position. 
//                                                                            -x left, +x right, -y up, +y down. 0 maintain position
```

### Mouse Drags
 

For all mouse drag commands, use moving time > 0.2, or even higher, depending on distance. Especially important for macOS

In version 2.4.0 drag_mouse() was renamed to drag_mouse_to_pos(). New drag_mouse() is in relative to its current position

Drag action is: left click down, move mouse to position, left click up. Like when moving icons
```rust
rustautogui.drag_mouse_to_pos(150, 980, 2.0).unwrap(); // args: x, y, moving_time. 
rustautogui.drag_mouse_to(Some(200), Some(400), 1.2).unwrap(); // args: x, y, moving_time. Accepts option. None value keeps current pos.
rustautogui.drag_mouse(500, -500, 1.0).unwrap(); // args: x, y, moving_time. Drags mouse relative to its current position.
//                                                                           Same rules as in move_mouse

```

Below is a helper function to determine coordinates on screen, helpful when determining region or mouse move target when developing

```rust
use rustautogui::print_mouse_position;
fn main() {
   print_mouse_position().unwrap();
}
```

## Keyboard Functions

Currently, only US keyboard is implemented. If you have different layout active, lots of characters will not work correctly
```rust
rustautogui.keyboard_input("test!@#24").unwrap(); // input string, or better say, do the sequence of key presses
rustautogui.keyboard_command("backspace").unwrap(); // press a keyboard button
rustautogui.keyboard_multi_key("shift", "control", Some("t")).unwrap(); // Executed multiple key press at same time. third argument is optional
rustautogui.key_down("backspace").unwrap(); // press a keyboard button down only
rustautogui.key_up("backspace").unwrap(); // press a keyboard button down only
```


For all the keyboard commands check Keyboard_commands.md, a table of possible keyboard inputs/commands for each OS. If you
find some keyboard commands missing that you need, please open an issue in order to get it added in next versions.






## Warnings Options

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

# OpenCL 

To enable OpenCL, as mentioned above, add crate to your Cargo.toml with opencl feature enabled: 

`rustautogui = { version = "2.5.0", features = ["opencl"] }`

## OpenCL Installation

### Linux

1) Install OpenCL ICD:

   `sudo apt install ocl-icd-opencl-dev`

2) If clinfo is not installed:

   `sudo apt install clinfo`


Run clinfo, if no GPU detected, continue. Otherwise you're finished.

3) Install OpenCL drivers

   For Nvidia GPUs:

   `sudo apt install nvidia-opencl-icd`

   For AMD GPUs:

   Follow official ROCm docs for your distro and GPU

   For Intel GPUs:

   `sudo apt install intel-opencl-icd`


### Windows
- install drivers for your graphics card.

### MacOS
- should be immediately ready.


## Please read before using OpenCL ⚠️⚠️


- **OpenCL works only on Segmented match mode**. Running FFT matchmode will fall back to CPU. 

- ⚠️ OpenCL performance highly depends on your GPU. On low-end or integrated GPUs, it may perform worse than CPU processing.

- to utilize opencl, prepare templates with matchmodes SegmentedOcl or SegmentedOclV2

- In case there are multiple GPU devices(often found on mac), the best gpu is chosen my scoring them according to their memory size, clock freq and compute units count
  

To display available devices and change default device:
```rust
gui.list_devices();

gui.change_ocl_device(1); // selects device on index = 1
```
⚠️ Changing device completely resets all the prepared data. 


## V1 vs V2 Algorithms
Your choice between V1 and V2 algorithms can significantly affect performance and reliability, depending on your use case.

⚙️ V1 — Robust & Consistent
- Less sensitive to GPU performance variations.

- Generally slower than V1, but more reliable across different hardware.

- Best used with non-_custom functions, where threshold is automatically determined.

- ✅ Recommended for general use and when you prefer consistency over raw speed.

⚡ V2 — Fast & Flexible
- More sensitive to the template image and the search image used.

- Can be faster than V1, but only when the threshold is well-tuned.

- Best used with _custom functions where you manually set the threshold.

- ⚠️ Using a bad threshold can lead to performance worse than V1.
  
- ⚠️ Not intended for built in or lower tier GPUs

💡 In short:

- Use V1 for safety and automatic tuning.

- Use V2 when optimizing for speed and you're ready to tune thresholds.






# Other Info

## How does crate work

- On Windows, RustAutoGUI interacts with winapi
- on Linux, it uses x11, and Wayland is not supported
- on macOS, it uses core-graphics crate
- OpenCL is utilized through ocl crate


## Major changes
For more details, check CHANGELOG.md

- 1.0.0 - introduces segmented match mode
- 2.0.0 - removed most of panics and crashes
- 2.1.0 - fixed on keyboard, some methods arguments / returns changed and will cause code breaking.
- 2.2.0 - loading multiple images, loading images from memory
- 2.3.0 - rework and improvement on Segmented match mode
- 2.4.0 - many additional functions for mouse and keyboard
- 2.5.0 - OpenCL implementation



