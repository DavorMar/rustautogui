# Changelog
All notable changes to this project will be documented in this file.

## [2.5.0] - 2025-05-
### Added 
- Added **OpenCL** implementation of the algorithm. Now you can run the template matching process of GPU to achieve better performance. Two variants of algorithm included
- Adding Opencl expanded MatchMode enum with 2 more variatns: SegmentedOcl and SegmentedOclV2
- Added optimization to segmented template match with automatic threshold detection
- Added *_custom template preparation functions which allow user to input threshold themselves if they dont want automatic detection
- Code cleanup / restructure
- Added feature "lite". This includes much lighter version of library with no template matching possibility, just keyboard and mouse included

## [2.4.0] - 2025-04-05
### Added / Changed / Fixed
- Changed: function drag_mouse to drag_mouse_to_pos. Reason is the new drag_mouse function that moves mouse in relation to current position
- made MouseClick enum public
- Added functions: 
-    - move_mouse() - Moves mouse relative to its current position. -x left, +x right, -y up, +y down
-    - move_mouse_to() -- Accepts Option<x>, Option<y>. None value keeps the same x or y. Usefull for horizontal and vertical movement
-    - drag_mouse() - performs mouse drag action relative to current position. Same rules as in move_mouse function
-    - drag_mouse_to() - same as in mouse_move_to, accepts Options. 
-    - drag_mouse_to_pos() - same as in move_mouse_to_pos(). This is the old drag_mouse() func 
-    - get_mouse_position() - returns Result<(i32, i32)> current mouse position
-    - click() - Mouse click - choose button with MouseClick enum
-    - click_down() - accepts MouseClick enum (does not work on macOS)
-    - click_up() - accepts MouseClick enum (does not work on macOS)
-    - key_down() - executes key press down only 
-    - key_up() - executes key press up only
- move_mouse_to_pos() remains the same, while drag_mouse_to_pos() is new name for the old version of drag_mouse() function

## [2.3.0] - 2025-03-30
### Fixed / Removed
- Rework of Segmented NCC template match. Completely removed argument of Max segments and made it always work robustly, never sacrificing precision for  speed. Additionally fixed part of formulas which will additionally reduce false positives, regardless of max segments. The fix also improves algorithms speed when compared to previous version, if max_segments is not taken into consideration. The speed gain is due to much less checks and verifications in the algorithm. 
- Fixed returned values from find image on screen, where its correctly returning positions adjusted for screen region and template size, where previously that worked only on find image and move mouse
- Removed completely the change prepared template function. 


## [2.2.2] - 2025-03-27
### Fixed
- use `&str` && `&[]` more wide

## [2.2.1] - 2025-03-26
### Fixed
- macOS alias check turned off till fixed


## [2.2.0] - 2025-03-25
### Added / Fixed
- Added ability to store multiple images (stored in Hashmap in struct) and give them alias. Can be stored from path, Imagebuffer or encoded u8 vec
- Added corresponsing find_stored_image_on_screen() and find_stored_image_on_screen_and_move_mouse() which additionaly take alias parameter
- Added prepare_template_from_imagebuffer() which accepts Imagebuffers RGB, RGBa and Luma(black and white)
- Added prepare_template_from_raw_encoded() which can load from encoded u8 vec
- Added search for image with implemented loop
- Added Super/Win key commands for Linux
- Added F1-F20 keys for Linux
- Added another example
- Added custom error type
- Made template search for macOS retina displays more robust. Now 2 variants of template are stored, the provided one and resized one. The search is done for both of them. The reason for this is, it cannot be known if user is providing template image which he got as a snip which needs to be resized, or from a (for instance) downloaded image which does not require resize
- updated to latest versions of dependencies
- Fix: find_image_and_move_mouse now returns vec of all found locations instead of just top location
- Fix: README code examples fixed
- Fix: check for out of bounds on windows mouse move fixed
- imgtools::convert_image_to_bw was renamed to convert_rgba_to_bw
- cleaned up and moved lots of things to private. Only available modules to public now are RustAutoGui, MatchMode, imgtools, normalized_x_corr and function print_mouse_position()
- Win keys currently seem to not work on windows. They will be left in the code and accessible to call, since they issue is most likely not related to code and could be resolved. On keyboard_commands.md they are now labeled as not implemented






## [2.1.1] - 2025-03.16
### Fixed
- added some missing keys for keyboard
- a more detailed list of available key commands is now available in Keyboard_commands.md
- old keyboard_commands.txt file has been removed
- negative movement_times for mouse will not produce errors anymore. Instead, they are treated as 0.0 value

## [2.1.0] - 2025-03.15
### Added/Fixed
- added scroll left / right functionality
- added drag mouse functionalty (click down -> move to location -> click up)
- added get_screen_size() method
- Fix for keyboard. Works on US layout only at the moment. Shifted argument from keyboard_input() method removed
- fixed double click on MacOS
- fix: added another check for region boundaries that should prevent the code to run an assertion, rather returning an error instead.
- fix: find_image_and_move_mouse now returns correct position
- changed move_mouse to accept u32 instead of i32 as x, y parameters
- included warnings


## [2.0.1] - 2025-03.14
### Fixed
- Fixed readme code examples
- fixed Segmented normalized cross correlation doing false matches.

## [2.0.0] - 2025-03.10
### Added/Fixed
- complete rework of the code which will not be compatible with old versions.
- introduced graceful exits, except for some situations like not having x11 activated on linux
- most of methods return Result<> now.

## [1.0.1] - 2025-03.07
### Fixed
- fixed wrong creation of debug folder even when not in debug mode

## [1.0.0] - 2025-02.25
### Added
- Segmented correlation template matching mode included

## [0.3.2] - 2024-09.03
### Fixed
- fix MACOS capture screen on retina


## [0.3.1] - 2024-08.01
### Added
-small optimization to template prepare

## [0.3.0] - 2024-07.27
### Removed
-removed egui and eframe dependencies. Unnecessary and used just to create one window to show mouse position. Simply printing it now.

## [0.2.2] - 2024-07.27
### Added
-scroll up and scroll down functions

## [0.2.1] - 2024-07.26
### Added
-multi key press function

## [0.2.0] - 2024-07-26
### Added
- macOS support
