#![allow(unused_doc_comments, unused_imports)]
#![doc = include_str!("../README.md")]

#[cfg(all(feature = "lite", feature = "opencl"))]
compile_error!("Features `lite` and `opencl` cannot be enabled at the same time.");

pub mod core;
#[cfg(not(feature = "lite"))]
pub mod data;
pub mod errors;
pub mod imgtools;
mod rustautogui_impl;

mod imports {
    // main stuff that is used by all features

    pub use std::{collections::HashMap, env};

    // this is in default, not featured in lite
    #[cfg(not(feature = "lite"))]
    pub use crate::core::{
        keyboard::Keyboard,
        mouse::{mouse_position, Mouse, MouseScroll},
        screen::Screen,
        template_match,
    };
    #[cfg(not(feature = "lite"))]
    pub use crate::data::{BackupData, PreparedData, SegmentedData, TemplateMatchingData};
    // opencl stuff
    #[cfg(feature = "opencl")]
    pub use crate::data::{DevicesInfo, OpenClData};
    #[cfg(feature = "opencl")]
    pub use ocl::{enums, Buffer, Context, Kernel, Program, Queue};
}

use crate::errors::*;

#[cfg(not(feature = "lite"))]
use data::SegmentedData; ///////////////////////// REMOVE

pub use core::mouse::mouse_position::print_mouse_position;
pub use core::mouse::MouseClick;
#[cfg(not(feature = "lite"))]
const DEFAULT_ALIAS: &str = "default_rsgui_!#123#!";
#[cfg(not(feature = "lite"))]
const DEFAULT_BCKP_ALIAS: &str = "bckp_tmpl_.#!123!#.";

/// Matchmode Segmented correlation and Fourier transform correlation
#[derive(PartialEq, Debug)]
#[cfg(not(feature = "lite"))]
pub enum MatchMode {
    Segmented,
    FFT,
    #[cfg(feature = "opencl")]
    SegmentedOcl,
    #[cfg(feature = "opencl")]
    SegmentedOclV2,
}
#[cfg(not(feature = "lite"))]
impl Clone for MatchMode {
    fn clone(&self) -> Self {
        match self {
            MatchMode::Segmented => MatchMode::Segmented,
            MatchMode::FFT => MatchMode::FFT,
            #[cfg(feature = "opencl")]
            MatchMode::SegmentedOcl => MatchMode::SegmentedOcl,
            #[cfg(feature = "opencl")]
            MatchMode::SegmentedOclV2 => MatchMode::SegmentedOclV2,
        }
    }
}

/// Main struct for Rustautogui
/// Struct gets assigned keyboard, mouse and struct to it implemented functions execute commands from each of assigned substructs
/// executes also correlation algorithms when doing find_image_on_screen

#[allow(dead_code)]
pub struct RustAutoGui {
    #[cfg(not(feature = "lite"))]
    template_data: imports::TemplateMatchingData,
    debug: bool,
    template_height: u32,
    template_width: u32,
    keyboard: imports::Keyboard,
    mouse: imports::Mouse,
    screen: imports::Screen,

    suppress_warnings: bool,

    #[cfg(feature = "opencl")]
    opencl_data: imports::OpenClData,
}
impl RustAutoGui {
    /// initiation of screen, keyboard and mouse that are assigned to new rustautogui struct.
    /// all the other struct fields are initiated as 0 or None

    pub fn new(debug: bool) -> Result<Self, AutoGuiError> {
        // initiation of screen, keyboard and mouse
        // on windows there is no need to share display pointer accross other structs
        #[cfg(any(target_os = "windows", target_os = "macos"))]
        let screen = imports::Screen::new()?;
        #[cfg(target_os = "linux")]
        let screen = imports::Screen::new();
        let keyboard = imports::Keyboard::new();
        let mouse_struct: imports::Mouse = imports::Mouse::new();
        // check for env variable to suppress warnings, otherwise set default false value
        let suppress_warnings = imports::env::var("RUSTAUTOGUI_SUPPRESS_WARNINGS")
            .map(|val| val == "1" || val.eq_ignore_ascii_case("true"))
            .unwrap_or(false); // Default: warnings are NOT suppressed

        // OCL INITIALIZATION
        #[cfg(feature = "opencl")]
        let opencl_data = Self::setup_opencl(None)?;

        #[cfg(not(feature = "lite"))]
        let template_match_data = imports::TemplateMatchingData {
            template: None,
            prepared_data: imports::PreparedData::None,
            prepared_data_stored: imports::HashMap::new(),
            match_mode: None,
            region: (0, 0, 0, 0),
            alias_used: DEFAULT_ALIAS.to_string(),
        };

        Ok(Self {
            #[cfg(not(feature = "lite"))]
            template_data: template_match_data,
            debug: debug,
            template_width: 0,
            template_height: 0,
            keyboard: keyboard,
            mouse: mouse_struct,
            screen: screen,

            suppress_warnings: suppress_warnings,

            #[cfg(feature = "opencl")]
            opencl_data: opencl_data,
        })
    }

    #[cfg(feature = "opencl")]
    fn setup_opencl(device_id: Option<u32>) -> Result<imports::OpenClData, AutoGuiError> {
        let context = imports::Context::builder().build()?;
        let available_devices = context.devices();
        let device_count = available_devices.len();
        let mut device_list: Vec<imports::DevicesInfo> = Vec::new();
        let mut highest_score = 0;
        let mut best_device_index = 0;
        let mut max_workgroup_size = 0;
        for (i, device) in available_devices.into_iter().enumerate() {
            let device_type = device.info(imports::enums::DeviceInfo::Type)?.to_string();
            let workgroup_size: u32 = device
                .info(imports::enums::DeviceInfo::MaxWorkGroupSize)?
                .to_string()
                .parse()
                .map_err(|_| AutoGuiError::OSFailure("Failed to read GPU data".to_string()))?;
            let global_mem: u64 = device
                .info(imports::enums::DeviceInfo::GlobalMemSize)?
                .to_string()
                .parse()
                .map_err(|_| AutoGuiError::OSFailure("Failed to read GPU data".to_string()))?;
            let compute_units: u32 = device
                .info(imports::enums::DeviceInfo::MaxComputeUnits)?
                .to_string()
                .parse()
                .map_err(|_| AutoGuiError::OSFailure("Failed to read GPU data".to_string()))?;

            let clock_frequency = device
                .info(imports::enums::DeviceInfo::MaxClockFrequency)?
                .to_string()
                .parse()
                .map_err(|_| AutoGuiError::OSFailure("Failed to read GPU data".to_string()))?;
            let device_vendor = device.info(imports::enums::DeviceInfo::Vendor)?.to_string();
            let device_name = device.info(imports::enums::DeviceInfo::Name)?.to_string();
            let global_mem_gb = global_mem / 1_048_576;
            let score = global_mem_gb as u32 * 2 + compute_units * 10 + clock_frequency;
            let gui_device = imports::DevicesInfo::new(
                device,
                i as u32,
                global_mem_gb as u32,
                clock_frequency,
                compute_units,
                device_vendor,
                device_name,
                score,
            );

            device_list.push(gui_device);
            match device_id {
                Some(x) => {
                    if x as usize > device_count {
                        return Err(ocl::Error::from("No device found for the given index"))?;
                    }
                    if i == x as usize {
                        highest_score = score;
                        best_device_index = i;
                        max_workgroup_size = workgroup_size;
                    }
                }
                None => {
                    if score >= highest_score && device_type.contains("GPU") {
                        highest_score = score;
                        best_device_index = i;
                        max_workgroup_size = workgroup_size;
                    }
                }
            }
        }
        let used_device = context.devices()[best_device_index as usize];
        let queue = imports::Queue::new(&context, used_device, None)?;
        let program_source = imports::template_match::opencl_kernel::OCL_KERNEL;
        let program = imports::Program::builder()
            .src(program_source)
            .build(&context)?;

        let opencl_data = imports::OpenClData {
            device_list: device_list,
            ocl_program: program,
            ocl_context: context,
            ocl_queue: queue,
            ocl_buffer_storage: imports::HashMap::new(),
            ocl_kernel_storage: imports::HashMap::new(),
            ocl_workgroup_size: max_workgroup_size,
        };
        Ok(opencl_data)
    }

    /// set true to turn off warnings.
    pub fn set_suppress_warnings(&mut self, suppress: bool) {
        self.suppress_warnings = suppress;
    }

    /// changes debug mode. True activates debug
    pub fn change_debug_state(&mut self, state: bool) {
        self.debug = state;
    }

    /// returns screen width and height
    pub fn get_screen_size(&mut self) -> (i32, i32) {
        self.screen.dimension()
    }
    #[cfg(not(feature = "lite"))]
    /// saves screenshot and saves it at provided path
    pub fn save_screenshot(&mut self, path: &str) -> Result<(), AutoGuiError> {
        self.screen.grab_screenshot(path)?;
        Ok(())
    }
    #[cfg(feature = "opencl")]
    pub fn list_devices(&self) {
        for (i, item) in (&self.opencl_data.device_list).iter().enumerate() {
            println!("Device {i}:");
            println!("{}", item.print_device());
            println!("\n");
        }
    }
    #[cfg(feature = "opencl")]
    pub fn change_ocl_device(&mut self, device_index: u32) -> Result<(), AutoGuiError> {
        let new_opencl_data = Self::setup_opencl(Some(device_index))?;
        self.opencl_data = new_opencl_data;

        self.template_data.template = None;
        self.template_data.prepared_data = imports::PreparedData::None;
        self.template_data.prepared_data_stored = imports::HashMap::new();
        self.template_width = 0;
        self.template_height = 0;
        self.template_data.alias_used = DEFAULT_ALIAS.to_string();
        self.template_data.region = (0, 0, 0, 0);
        self.template_data.match_mode = None;

        Ok(())
    }
    #[cfg(not(feature = "lite"))]
    /// checks if region selected out of screen bounds, if template size > screen size (redundant)
    /// and if template size > region size
    fn check_if_region_out_of_bound(
        &mut self,
        template_width: u32,
        template_height: u32,
        region_x: u32,
        region_y: u32,
        region_width: u32,
        region_height: u32,
    ) -> Result<(), AutoGuiError> {
        if (region_x + region_width > self.screen.screen_width as u32)
            | (region_y + region_height > self.screen.screen_height as u32)
        {
            return Err(AutoGuiError::OutOfBoundsError(
                "Region size larger than screen size".to_string(),
            ));
        }

        // this is a redundant check since this case should be covered by the
        // next region check, but leaving it
        if (template_width > (self.screen.screen_width as u32))
            | (template_height > (self.screen.screen_height as u32))
        {
            return Err(AutoGuiError::OutOfBoundsError(
                "Template size larger than screen size".to_string(),
            ));
        }
        #[cfg(not(feature = "lite"))]
        if (template_width > region_width) | (template_height > region_height) {
            return Err(AutoGuiError::OutOfBoundsError(
                "Template size larger than region size".to_string(),
            ));
        }
        #[cfg(not(feature = "lite"))]
        if template_height * template_width == 0 {
            Err(ImageProcessingError::Custom(
                "Template size = 0. Please check loaded template if its correct".to_string(),
            ))?;
        }
        Ok(())
    }
}

#[cfg(target_os = "linux")]
impl Drop for RustAutoGui {
    fn drop(&mut self) {
        self.screen.destroy();
    }
}

#[cfg(not(feature = "lite"))]
#[cfg(target_os = "windows")]
impl Drop for RustAutoGui {
    fn drop(&mut self) {
        self.screen.destroy();
    }
}
