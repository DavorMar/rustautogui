# Changelog
All notable changes to this project will be documented in this file.

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