#![allow(unused_doc_comments, unused_imports)]
#![doc = include_str!("../README.md")]

#[cfg(all(feature = "lite", feature = "opencl"))]
compile_error!("Features `lite` and `opencl` cannot be enabled at the same time.");

#[cfg(not(feature = "lite"))]
pub mod data_structs;
pub mod errors;
pub mod imgtools;
mod keyboard;
mod mouse;
mod screen;
#[cfg(not(feature = "lite"))]
pub mod template_match;

mod imports {
    // main stuff that is used by all features
    #[cfg(target_os = "linux")]
    pub use crate::{keyboard::linux::Keyboard, mouse::linux::Mouse, screen::linux::Screen};
    #[cfg(target_os = "macos")]
    pub use crate::{keyboard::macos::Keyboard, mouse::macos::Mouse, screen::macos::Screen};
    #[cfg(target_os = "windows")]
    pub use crate::{keyboard::windows::Keyboard, mouse::windows::Mouse, screen::windows::Screen};
    pub use std::{collections::HashMap, env, fmt, fs, path::Path, str::FromStr};

    // this is in default, not featured in lite
    #[cfg(not(feature = "lite"))]
    pub use crate::data_structs::{BackupData, PreparedData, SegmentedData, TemplateMatchingData};
    #[cfg(not(feature = "lite"))]
    pub use image::{
        imageops::{resize, FilterType::Nearest},
        DynamicImage, GrayImage, ImageBuffer, Luma, Pixel, Primitive, Rgb, Rgba,
    };
    #[cfg(not(feature = "lite"))]
    pub use rustfft::{num_complex::Complex, num_traits::ToPrimitive};

    // opencl stuff
    #[cfg(feature = "opencl")]
    pub use crate::data_structs::{DevicesInfo, GpuMemoryPointers, KernelStorage, OpenClData};
    #[cfg(feature = "opencl")]
    pub use crate::template_match::open_cl::OclVersion;
    #[cfg(feature = "opencl")]
    pub use ocl::{enums, Buffer, Context, Kernel, Program, Queue};
}

use crate::errors::*;

#[cfg(not(feature = "lite"))]
use data_structs::SegmentedData; ///////////////////////// REMOVE

pub use mouse::mouse_position::print_mouse_position;
pub use mouse::MouseClick;
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
    #[cfg(not(feature="lite"))]
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
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    pub fn new(debug: bool) -> Result<Self, AutoGuiError> {
        // initiation of screen, keyboard and mouse
        // on windows there is no need to share display pointer accross other structs
        let screen = imports::Screen::new()?;
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
        let template_match_data= imports::TemplateMatchingData {    
            template: None,
            prepared_data: imports::PreparedData::None,
            prepared_data_stored: imports::HashMap::new(),
            match_mode: None,
            region: (0, 0, 0, 0),
            alias_used: DEFAULT_ALIAS.to_string(),
        };

        Ok(Self {
            #[cfg(not(feature="lite"))]
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

    /// initiation of screen, keyboard and mouse that are assigned to new rustautogui struct.
    /// all the other struct fields are initiated as 0 or None
    #[cfg(target_os = "linux")]
    pub fn new(debug: bool) -> Result<Self, AutoGuiError> {
        // on linux, screen display pointer is shared to keyboard and mouse
        // x11 works like that and initiation of individual display objects
        // under each struct wouldnt be preferable
        
        
        let screen = imports::Screen::new();
        let keyboard = imports::Keyboard::new(screen.display);
        let mouse_struct = imports::Mouse::new(screen.display, screen.root_window);
        // check for env variable to suppress warnings, otherwise set default false value
        let suppress_warnings = imports::env::var("RUSTAUTOGUI_SUPPRESS_WARNINGS")
            .map(|val| val == "1" || val.eq_ignore_ascii_case("true"))
            .unwrap_or(false); // Default: warnings are NOT suppressed

        // OCL INITIALIZATION
        #[cfg(feature = "opencl")]
        let opencl_data = Self::setup_opencl(None)?;

        #[cfg(not(feature = "lite"))]
        let template_match_data= imports::TemplateMatchingData {    
            template: None,
            prepared_data: imports::PreparedData::None,
            prepared_data_stored: imports::HashMap::new(),
            match_mode: None,
            region: (0, 0, 0, 0),
            alias_used: DEFAULT_ALIAS.to_string(),
        };

        Ok(Self {
            #[cfg(not(feature="lite"))]
            template_data: template_match_data,
            debug: debug,
            template_width: 0,
            template_height: 0,
            keyboard: keyboard,
            mouse: mouse_struct,
            screen: screen,
            
            suppress_warnings: suppress_warnings,
            
            #[cfg(feature = "opencl")]
            opencl_data,
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
        let program_source = crate::template_match::opencl_kernel::OCL_KERNEL;
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

    ///////////////////////// prepare single template functions //////////////////////////
    #[cfg(not(feature = "lite"))]
    /// main prepare template picture which takes ImageBuffer Luma u8. all the other variants
    /// of prepare/store funtions call this function
    #[allow(unused_mut)]
    fn prepare_template_picture_bw(
        &mut self,
        mut template: imports::ImageBuffer<imports::Luma<u8>, Vec<u8>>,
        region: Option<(u32, u32, u32, u32)>,
        match_mode: MatchMode,
        alias: Option<&str>,
        user_threshold: Option<f32>,
    ) -> Result<(), AutoGuiError> {
        // resize and adjust if retina screen is used
        // prepare additionally backup template for 2 screen size variants
        // issue comes from retina having digitally doubled the amount of displayed pixels while
        // API returns screen image with original size
        // for instance the screen is 1400x800 but if snip of screen is taken, output image will be 2800x1600
        // for that reason, we cannot be sure which variant of image will be searched for, so image search will search first
        // for resized variant and if not found, then non scaled variant
        // since this recursively initiates construction of another backup prepared template for macos
        // we dont want to back up the backup
        #[cfg(target_os = "macos")]
        {
            self.prepare_macos_backup(&match_mode, template.clone(), region, alias)?;
            match alias {
                Some(a) => {
                    if a.contains(DEFAULT_BCKP_ALIAS) { //skip
                    } else {
                        template = imports::resize(
                            &template,
                            template.width() / self.screen.screen_data.scaling_factor_x as u32,
                            template.height() / self.screen.screen_data.scaling_factor_y as u32,
                            imports::Nearest,
                        );
                    }
                }
                None => {
                    template = imports::resize(
                        &template,
                        template.width() / self.screen.screen_data.scaling_factor_x as u32,
                        template.height() / self.screen.screen_data.scaling_factor_y as u32,
                        imports::Nearest,
                    );
                }
            }
        }
        let (template_width, template_height) = template.dimensions();

        // if no region provided, grab whole screen
        let region = match region {
            Some(region_tuple) => region_tuple,
            None => {
                let (screen_width, screen_height) = self.screen.dimension();
                (0, 0, screen_width as u32, screen_height as u32)
            }
        };
        self.check_if_region_out_of_bound(
            template_width,
            template_height,
            region.0,
            region.1,
            region.2,
            region.3,
        )?;

        // do the rest of preparation calculations depending on the matchmode
        // FFT pads the image, does fourier transformations,
        // calculates conjugate and inverses transformation on template
        // Segmented creates vector of picture segments with coordinates, dimensions and average pixel value
        let (template_data, match_mode_option) = match match_mode.clone() {
            MatchMode::FFT => {
                let prepared_data =
                    imports::PreparedData::FFT(template_match::fft_ncc::prepare_template_picture(
                        &template, region.2, region.3,
                    ));
                let match_mode = Some(MatchMode::FFT);
                (prepared_data, match_mode)
            }

            MatchMode::Segmented => {
                let prepared_data: imports::PreparedData =
                    template_match::segmented_ncc::prepare_template_picture(
                        &template,
                        &self.debug,
                        user_threshold,
                    );
                if let imports::PreparedData::Segmented(ref segmented) = prepared_data {
                    // mostly happens due to using too complex image with small max segments value
                    if (segmented.template_segments_fast.len() == 1)
                        | (segmented.template_segments_slow.len() == 1)
                    {
                        Err(ImageProcessingError::new("Error in creating segmented template image. To resolve: either increase the max_segments, use FFT matching mode or use smaller template image"))?;
                    }
                }

                let match_mode = Some(MatchMode::Segmented);

                (prepared_data, match_mode)
            }

            #[cfg(feature = "opencl")]
            matchmode_val @ MatchMode::SegmentedOcl | matchmode_val @ MatchMode::SegmentedOclV2 => {
                let prepared_data: imports::PreparedData =
                    template_match::segmented_ncc::prepare_template_picture(
                        &template,
                        &self.debug,
                        user_threshold,
                    );
                let prepared_data: imports::SegmentedData =
                    if let imports::PreparedData::Segmented(segmented) = prepared_data {
                        // mostly happens due to using too complex image with small max segments value
                        if (segmented.template_segments_fast.len() == 1)
                            | (segmented.template_segments_slow.len() == 1)
                        {
                            Err(ImageProcessingError::new("Error in creating segmented template image. To resolve: either increase the max_segments, use FFT matching mode or use smaller template image"))?;
                        }
                        segmented
                    } else {
                        return Err(ImageProcessingError::new("Wrong data prepared  / stored."))?;
                    };
                let match_mode = Some(matchmode_val);
                {
                    let ocl_buffer_data = imports::GpuMemoryPointers::new(
                        region.2,
                        region.3,
                        template_width,
                        template_height,
                        &self.opencl_data.ocl_queue,
                        &prepared_data.template_segments_slow,
                        &prepared_data.template_segments_fast,
                    )?;

                    let kernels = imports::KernelStorage::new(
                        &ocl_buffer_data,
                        &self.opencl_data.ocl_program,
                        &self.opencl_data.ocl_queue,
                        region.2,
                        region.3,
                        template_width,
                        template_height,
                        prepared_data.template_segments_fast.len() as u32,
                        prepared_data.template_segments_slow.len() as u32,
                        prepared_data.segments_mean_fast,
                        prepared_data.segments_mean_slow,
                        prepared_data.segment_sum_squared_deviations_fast,
                        prepared_data.segment_sum_squared_deviations_slow,
                        prepared_data.expected_corr_fast,
                        self.opencl_data.ocl_workgroup_size as usize,
                    )?;
                    match alias {
                        Some(name) => {
                            self.opencl_data
                                .ocl_buffer_storage
                                .insert(name.into(), ocl_buffer_data);

                            self.opencl_data
                                .ocl_kernel_storage
                                .insert(name.into(), kernels);
                        }
                        None => {
                            self.opencl_data
                                .ocl_buffer_storage
                                .insert(DEFAULT_ALIAS.into(), ocl_buffer_data);
                            self.opencl_data
                                .ocl_kernel_storage
                                .insert(DEFAULT_ALIAS.into(), kernels);
                        }
                    }
                }

                (imports::PreparedData::Segmented(prepared_data), match_mode)
            }
        };

        // Alias Some -> storing the image , we just save it to Hashmap
        // Alias None -> not storing, then we change struct attributes to fit the single loaded image search
        match alias {
            Some(name) => {
                self.template_data.prepared_data_stored
                    .insert(name.into(), (template_data, region, match_mode));
            }
            None => {
                self.template_data.region = region;
                self.template_data.prepared_data = template_data;
                self.template_data.match_mode = match_mode_option;
                // update screen struct
                self.screen.screen_data.screen_region_width = region.2;
                self.screen.screen_data.screen_region_height = region.3;
                // update struct values
                self.template_width = template_width;
                self.template_height = template_height;
                // convert to option and store in struct
                self.template_data.template = Some(template.clone());
            }
        }
        return Ok(());
    }

    #[cfg(not(feature = "lite"))]
    // prepares also unscaled variant of image if retina display is on
    // since it is recursively calling again preparation of template with another alias
    // checks are made on alias_name to not run infinitely preparations of backups of backups
    #[cfg(target_os = "macos")]
    fn prepare_macos_backup(
        &mut self,
        match_mode: &MatchMode,
        template: imports::ImageBuffer<imports::Luma<u8>, Vec<u8>>,
        region: Option<(u32, u32, u32, u32)>,
        alias: Option<&str>,
    ) -> Result<(), AutoGuiError> {
        {
            if ((self.screen.screen_data.scaling_factor_x > 1.0)
                | (self.screen.screen_data.scaling_factor_y > 1.0))
                & (match alias {
                    Some(a) => !a.contains(DEFAULT_BCKP_ALIAS),
                    None => true,
                })
            // if conditions are met, prepare the backup
            {
                let bckp_template = template.clone();
                // matching alias to see is it regular single template load
                // or storing template with alias
                // where names for backups differ
                let backup_alias = match alias.map(ToString::to_string) {
                    Some(mut a) => {
                        a.push('_');
                        a.push_str(DEFAULT_BCKP_ALIAS);
                        a
                    }
                    None => DEFAULT_BCKP_ALIAS.to_string(),
                };
                // store the backup template that doesnt have resize
                // later on when doing template matching, itll first try to match
                // resized one, and if it doesnt work then it tries the original from backup
                self.store_template_from_imagebuffer(
                    bckp_template,
                    region,
                    match_mode.clone(),
                    &backup_alias,
                )?;
            };
        }

        Ok(())
    }
    #[cfg(not(feature = "lite"))]
    #[allow(dead_code)]
    fn check_alias_name(alias: &str) -> Result<(), ImageProcessingError> {
        if (alias.contains(DEFAULT_ALIAS)) | (alias.contains(DEFAULT_BCKP_ALIAS)) {
            return Err(ImageProcessingError::new(
                "Please do not use built in default alias names",
            ));
        }

        Ok(())
    }
    #[cfg(not(feature = "lite"))]
    /// Loads template from file on provided path
    pub fn prepare_template_from_file(
        &mut self,
        template_path: &str,
        region: Option<(u32, u32, u32, u32)>,
        match_mode: MatchMode,
    ) -> Result<(), AutoGuiError> {
        let template: imports::ImageBuffer<imports::Luma<u8>, Vec<u8>> =
            imgtools::load_image_bw(template_path)?;
        self.prepare_template_picture_bw(template, region, match_mode, None, None)
    }
    #[cfg(not(feature = "lite"))]
    /// Loads template from file on provided path
    pub fn prepare_template_from_file_custom(
        &mut self,
        template_path: &str,
        region: Option<(u32, u32, u32, u32)>,
        match_mode: MatchMode,
        threshold: f32,
    ) -> Result<(), AutoGuiError> {
        let template: imports::ImageBuffer<imports::Luma<u8>, Vec<u8>> =
            imgtools::load_image_bw(template_path)?;
        self.prepare_template_picture_bw(template, region, match_mode, None, Some(threshold))
    }
    #[cfg(not(feature = "lite"))]
    /// prepare from imagebuffer, works only on types RGB/RGBA/Luma
    pub fn prepare_template_from_imagebuffer<P, T>(
        &mut self,
        image: imports::ImageBuffer<P, Vec<T>>,
        region: Option<(u32, u32, u32, u32)>,
        match_mode: MatchMode,
    ) -> Result<(), AutoGuiError>
    where
        P: imports::Pixel<Subpixel = T> + 'static,
        T: imports::Primitive + imports::ToPrimitive + 'static,
    {
        let color_scheme = imgtools::check_imagebuffer_color_scheme(&image)?;
        let luma_img = imgtools::convert_t_imgbuffer_to_luma(&image, color_scheme)?;
        self.prepare_template_picture_bw(luma_img, region, match_mode, None, None)?;
        Ok(())
    }

    #[cfg(not(feature = "lite"))]
    pub fn prepare_template_from_imagebuffer_custom<P, T>(
        &mut self,
        image: imports::ImageBuffer<P, Vec<T>>,
        region: Option<(u32, u32, u32, u32)>,
        match_mode: MatchMode,
        threshold: f32,
    ) -> Result<(), AutoGuiError>
    where
        P: imports::Pixel<Subpixel = T> + 'static,
        T: imports::Primitive + imports::ToPrimitive + 'static,
    {
        let color_scheme = imgtools::check_imagebuffer_color_scheme(&image)?;
        let luma_img = imgtools::convert_t_imgbuffer_to_luma(&image, color_scheme)?;
        self.prepare_template_picture_bw(luma_img, region, match_mode, None, Some(threshold))?;
        Ok(())
    }

    #[cfg(not(feature = "lite"))]
    /// Only works on encoded images. uses image::load_from_memory() which reads first bytes of image which contain metadata depending on format.
    pub fn prepare_template_from_raw_encoded(
        &mut self,
        img_raw: &[u8],
        region: Option<(u32, u32, u32, u32)>,
        match_mode: MatchMode,
    ) -> Result<(), AutoGuiError> {
        let image = image::load_from_memory(img_raw)?;
        self.prepare_template_picture_bw(image.to_luma8(), region, match_mode, None, None)
    }

    #[cfg(not(feature = "lite"))]
    /// Only works on encoded images. uses image::load_from_memory() which reads first bytes of image which contain metadata depending on format.
    pub fn prepare_template_from_raw_encoded_custom(
        &mut self,
        img_raw: &[u8],
        region: Option<(u32, u32, u32, u32)>,
        match_mode: MatchMode,
        threshold: f32,
    ) -> Result<(), AutoGuiError> {
        let image = image::load_from_memory(img_raw)?;
        self.prepare_template_picture_bw(
            image.to_luma8(),
            region,
            match_mode,
            None,
            Some(threshold),
        )
    }

    ///////////////////////// store single template functions //////////////////////////
    #[cfg(not(feature = "lite"))]
    /// Store template data for multiple image search
    pub fn store_template_from_file(
        &mut self,
        template_path: &str,
        region: Option<(u32, u32, u32, u32)>,
        match_mode: MatchMode,
        alias: &str,
    ) -> Result<(), AutoGuiError> {
        // RustAutoGui::check_alias_name(&alias)?;
        let template: imports::ImageBuffer<imports::Luma<u8>, Vec<u8>> =
            imgtools::load_image_bw(template_path)?;
        self.prepare_template_picture_bw(template, region, match_mode, Some(alias), None)
    }

    #[cfg(not(feature = "lite"))]
    /// Store template data for multiple image search
    pub fn store_template_from_file_custom(
        &mut self,
        template_path: &str,
        region: Option<(u32, u32, u32, u32)>,
        match_mode: MatchMode,
        alias: &str,
        threshold: f32,
    ) -> Result<(), AutoGuiError> {
        // RustAutoGui::check_alias_name(&alias)?;
        let template: imports::ImageBuffer<imports::Luma<u8>, Vec<u8>> =
            imgtools::load_image_bw(template_path)?;
        self.prepare_template_picture_bw(template, region, match_mode, Some(alias), Some(threshold))
    }
    #[cfg(not(feature = "lite"))]
    /// Load template from imagebuffer and store prepared template data for multiple image search
    pub fn store_template_from_imagebuffer<P, T>(
        &mut self,
        image: imports::ImageBuffer<P, Vec<T>>,
        region: Option<(u32, u32, u32, u32)>,
        match_mode: MatchMode,
        alias: &str,
    ) -> Result<(), AutoGuiError>
    where
        P: imports::Pixel<Subpixel = T> + 'static,
        T: imports::Primitive + imports::ToPrimitive + 'static,
    {
        // RustAutoGui::check_alias_name(&alias)?;
        let color_scheme = imgtools::check_imagebuffer_color_scheme(&image)?;
        let luma_img = imgtools::convert_t_imgbuffer_to_luma(&image, color_scheme)?;
        self.prepare_template_picture_bw(luma_img, region, match_mode, Some(alias), None)
    }

    #[cfg(not(feature = "lite"))]
    /// Load template from imagebuffer and store prepared template data for multiple image search
    pub fn store_template_from_imagebuffer_custom<P, T>(
        &mut self,
        image: imports::ImageBuffer<P, Vec<T>>,
        region: Option<(u32, u32, u32, u32)>,
        match_mode: MatchMode,
        alias: &str,
        threshold: f32,
    ) -> Result<(), AutoGuiError>
    where
        P: imports::Pixel<Subpixel = T> + 'static,
        T: imports::Primitive + imports::ToPrimitive + 'static,
    {
        // RustAutoGui::check_alias_name(&alias)?;
        let color_scheme = imgtools::check_imagebuffer_color_scheme(&image)?;
        let luma_img = imgtools::convert_t_imgbuffer_to_luma(&image, color_scheme)?;
        self.prepare_template_picture_bw(luma_img, region, match_mode, Some(alias), Some(threshold))
    }
    #[cfg(not(feature = "lite"))]
    /// Load template from encoded raw bytes and store prepared template data for multiple image search
    pub fn store_template_from_raw_encoded(
        &mut self,
        img_raw: &[u8],
        region: Option<(u32, u32, u32, u32)>,
        match_mode: MatchMode,
        alias: &str,
    ) -> Result<(), AutoGuiError> {
        // RustAutoGui::check_alias_name(&alias)?;
        let image = image::load_from_memory(img_raw)?;
        self.prepare_template_picture_bw(image.to_luma8(), region, match_mode, Some(alias), None)?;
        Ok(())
    }
    #[cfg(not(feature = "lite"))]
    pub fn store_template_from_raw_encoded_custom(
        &mut self,
        img_raw: &[u8],
        region: Option<(u32, u32, u32, u32)>,
        match_mode: MatchMode,
        alias: &str,
        threshold: f32,
    ) -> Result<(), AutoGuiError> {
        // RustAutoGui::check_alias_name(&alias)?;
        let image = image::load_from_memory(img_raw)?;
        self.prepare_template_picture_bw(
            image.to_luma8(),
            region,
            match_mode,
            Some(alias),
            Some(threshold),
        )?;
        Ok(())
    }
    /// Searches for prepared template on screen.
    /// On windows only main monitor search is supported, while on linux, all monitors work.
    /// more details in README
    #[cfg(not(feature = "lite"))]
    #[allow(unused_variables)]
    pub fn find_image_on_screen(
        &mut self,
        precision: f32,
    ) -> Result<Option<Vec<(u32, u32, f32)>>, AutoGuiError> {
        /// searches for image on screen and returns found locations in vector format
        let image: imports::ImageBuffer<imports::Luma<u8>, Vec<u8>> =
            self.screen.grab_screen_image_grayscale(&self.template_data.region)?;

        if self.debug {
            let debug_path = imports::Path::new("debug");
            if !debug_path.exists() {
                match imports::fs::create_dir_all(debug_path) {
                    Ok(_) => {
                        println!("Created a debug folder in your root for saving segmented template images");
                        match image.save("debug/screen_capture.png") {
                            Ok(_) => (),
                            Err(x) => println!("{}", x.to_string()),
                        };
                    }
                    Err(x) => {
                        println!("Failed to create debug folder");
                        println!("{}", x.to_string());
                    }
                };
            }
        };

        #[cfg(target_os = "macos")]
        let locations = match self.run_macos_xcorr_with_backup(image, precision)? {
            Some(x) => x,
            None => return Ok(None),
        };
        #[cfg(not(target_os = "macos"))]
        let locations = match self.run_x_corr(image, precision)? {
            Some(x) => x,
            None => return Ok(None),
        };

        let locations_ajusted: Vec<(u32, u32, f32)> = locations
            .iter()
            .map(|(mut x, mut y, corr)| {
                x = x + self.template_data.region.0 + (self.template_width / 2);
                y = y + self.template_data.region.1 + (self.template_height / 2);
                (x, y, *corr)
            })
            .collect();

        return Ok(Some(locations_ajusted));
    }

    // for macOS with retina display, two runs are made. One for resized template
    // and if not found , then second for normal sized template
    // since the function recursively calls find_stored_image_on_screen -> run_macos_xcorr_with_backup
    // covers are made to not run it for backup aswell
    #[cfg(not(feature = "lite"))]
    #[cfg(target_os = "macos")]
    fn run_macos_xcorr_with_backup(
        &mut self,
        image: imports::ImageBuffer<imports::Luma<u8>, Vec<u8>>,
        precision: f32,
    ) -> Result<Option<Vec<(u32, u32, f32)>>, AutoGuiError> {
        let first_match = self.run_x_corr(image, precision);
        // if retina and if this is not already a recursively ran backup
        if ((self.screen.screen_data.scaling_factor_x > 1.0)
            | (self.screen.screen_data.scaling_factor_y > 1.0))
            & (!self.alias_used.contains(DEFAULT_BCKP_ALIAS))
        {
            match first_match? {
                Some(result) => return Ok(Some(result)),
                None => {
                    let mut bckp_alias = String::new();

                    // if its not a single image search, create a alias_backup hash
                    if self.alias_used != DEFAULT_ALIAS.to_string() {
                        bckp_alias.push_str(self.alias_used.as_str());
                        bckp_alias.push('_');
                    }
                    bckp_alias.push_str(DEFAULT_BCKP_ALIAS);
                    // this recursively searches again for backup
                    return self.find_stored_image_on_screen(precision, &bckp_alias);
                }
            }
        }
        first_match
    }
    #[cfg(not(feature = "lite"))]
    /// loops until image is found and returns found values, or until it times out
    pub fn loop_find_image_on_screen(
        &mut self,
        precision: f32,
        timeout: u64,
    ) -> Result<Option<Vec<(u32, u32, f32)>>, AutoGuiError> {
        if (timeout == 0) & (!self.suppress_warnings) {
            eprintln!(
                "Warning: setting a timeout to 0 on a loop find image initiates an infinite loop"
            )
        }

        let timeout_start = std::time::Instant::now();
        loop {
            if (timeout_start.elapsed().as_secs() > timeout) & (timeout > 0) {
                Err(ImageProcessingError::new(
                    "loop find image timed out. Could not find image",
                ))?;
            }
            let result = self.find_image_on_screen(precision)?;
            match result {
                Some(r) => return Ok(Some(r)),
                None => continue,
            }
        }
    }
    #[cfg(not(feature = "lite"))]
    /// find image stored under provided alias
    pub fn find_stored_image_on_screen(
        &mut self,
        precision: f32,
        alias: &str,
    ) -> Result<Option<Vec<(u32, u32, f32)>>, AutoGuiError> {
        let (prepared_data, region, match_mode) =
            self.template_data.prepared_data_stored
                .get(alias)
                .ok_or(AutoGuiError::AliasError(
                    "No template stored with selected alias".to_string(),
                ))?;
        // save to reset after finished
        let backup = imports::BackupData {
            starting_data: self.template_data.prepared_data.clone(),
            starting_region: self.template_data.region.clone(),
            starting_match_mode: self.template_data.match_mode.clone(),
            starting_template_height: self.template_height.clone(),
            starting_template_width: self.template_width.clone(),
            starting_alias_used: self.template_data.alias_used.clone(),
        };

        self.template_data.alias_used = alias.into();
        self.template_data.prepared_data = prepared_data.clone();
        self.screen.screen_data.screen_region_width = region.2;
        self.screen.screen_data.screen_region_height = region.3;
        self.template_data.region = *region;
        self.template_data.match_mode = Some(match_mode.clone());
        match prepared_data {
            imports::PreparedData::FFT(data) => {
                self.template_width = data.template_width;
                self.template_height = data.template_height;
            }
            imports::PreparedData::Segmented(data) => {
                self.template_width = data.template_width;
                self.template_height = data.template_height;
            }
            imports::PreparedData::None => {
                Err(ImageProcessingError::new("No prepared data loaded"))?
            }
        };
        let points = self.find_image_on_screen(precision)?;
        // reset to starting info
        backup.update_rustautogui(self);

        Ok(points)
    }

    #[cfg(not(feature = "lite"))]
    /// loops until stored image is found and returns found values, or until it times out
    pub fn loop_find_stored_image_on_screen(
        &mut self,
        precision: f32,
        timeout: u64,
        alias: &str,
    ) -> Result<Option<Vec<(u32, u32, f32)>>, AutoGuiError> {
        if (timeout == 0) & (!self.suppress_warnings) {
            eprintln!(
                "Warning: setting a timeout to 0 on a loop find image initiates an infinite loop"
            )
        }
        let timeout_start = std::time::Instant::now();
        loop {
            if (timeout_start.elapsed().as_secs() > timeout) & (timeout > 0) {
                Err(ImageProcessingError::new(
                    "loop find image timed out. Could not find image",
                ))?;
            }
            let result = self.find_stored_image_on_screen(precision, alias)?;
            match result {
                Some(r) => return Ok(Some(r)),
                None => continue,
            }
        }
    }
    #[cfg(not(feature = "lite"))]
    /// searches for image stored under provided alias and moves mouse to position
    pub fn find_stored_image_on_screen_and_move_mouse(
        &mut self,
        precision: f32,
        moving_time: f32,
        alias: &str,
    ) -> Result<Option<Vec<(u32, u32, f32)>>, AutoGuiError> {
        let (prepared_data, region, match_mode) =
            self.template_data.prepared_data_stored
                .get(alias)
                .ok_or(AutoGuiError::AliasError(
                    "No template stored with selected alias".to_string(),
                ))?;
        // save to reset after finished
        let backup = imports::BackupData {
            starting_data: self.template_data.prepared_data.clone(),
            starting_region: self.template_data.region.clone(),
            starting_match_mode: self.template_data.match_mode.clone(),
            starting_template_height: self.template_height.clone(),
            starting_template_width: self.template_width.clone(),
            starting_alias_used: self.template_data.alias_used.clone(),
        };
        self.template_data.alias_used = alias.into();
        self.template_data.prepared_data = prepared_data.clone();
        self.template_data.region = *region;
        self.screen.screen_data.screen_region_width = region.2;
        self.screen.screen_data.screen_region_height = region.3;
        self.template_data.match_mode = Some(match_mode.clone());
        match prepared_data {
            imports::PreparedData::FFT(data) => {
                self.template_width = data.template_width;
                self.template_height = data.template_height;
            }
            imports::PreparedData::Segmented(data) => {
                self.template_width = data.template_width;
                self.template_height = data.template_height;
            }
            imports::PreparedData::None => {
                Err(ImageProcessingError::new("No prepared data loaded"))?
            }
        };
        let found_points = self.find_image_on_screen_and_move_mouse(precision, moving_time);

        // reset to starting info
        backup.update_rustautogui(self);

        found_points
    }
    #[cfg(not(feature = "lite"))]
    /// loops until stored image is found and moves mouse
    pub fn loop_find_stored_image_on_screen_and_move_mouse(
        &mut self,
        precision: f32,
        moving_time: f32,
        timeout: u64,
        alias: &str,
    ) -> Result<Option<Vec<(u32, u32, f32)>>, AutoGuiError> {
        if (timeout == 0) & (!self.suppress_warnings) {
            eprintln!(
                "Warning: setting a timeout to 0 on a loop find image initiates an infinite loop"
            )
        }
        let timeout_start = std::time::Instant::now();
        loop {
            if (timeout_start.elapsed().as_secs() > timeout) & (timeout > 0) {
                Err(ImageProcessingError::new(
                    "loop find image timed out. Could not find image",
                ))?;
            }
            let result =
                self.find_stored_image_on_screen_and_move_mouse(precision, moving_time, alias)?;
            match result {
                Some(r) => return Ok(Some(r)),
                None => continue,
            }
        }
    }
    #[cfg(not(feature = "lite"))]
    /// executes find_image_on_screen and moves mouse to the middle of the image.
    pub fn find_image_on_screen_and_move_mouse(
        &mut self,
        precision: f32,
        moving_time: f32,
    ) -> Result<Option<Vec<(u32, u32, f32)>>, AutoGuiError> {
        /// finds coordinates of the image on the screen and moves mouse to it. Returns None if no image found
        ///  Best used in loops
        let found_locations = self.find_image_on_screen(precision)?;

        let locations = match found_locations.clone() {
            Some(locations) => locations,
            None => return Ok(None),
        };

        let (target_x, target_y, _) = locations[0];

        self.move_mouse_to_pos(target_x, target_y, moving_time)?;

        return Ok(Some(locations));
    }
    #[cfg(not(feature = "lite"))]
    /// loops until image is found and returns found values, or until it times out
    pub fn loop_find_image_on_screen_and_move_mouse(
        &mut self,
        precision: f32,
        moving_time: f32,
        timeout: u64,
    ) -> Result<Option<Vec<(u32, u32, f32)>>, AutoGuiError> {
        if (timeout == 0) & (!self.suppress_warnings) {
            eprintln!(
                "Warning: setting a timeout to 0 on a loop find image initiates an infinite loop"
            )
        }
        let timeout_start = std::time::Instant::now();
        loop {
            if (timeout_start.elapsed().as_secs() > timeout) & (timeout > 0) {
                Err(ImageProcessingError::new(
                    "loop find image timed out. Could not find image",
                ))?;
            }
            let result = self.find_image_on_screen_and_move_mouse(precision, moving_time)?;
            match result {
                Some(e) => return Ok(Some(e)),
                None => continue,
            }
        }
    }

    #[cfg(not(feature = "lite"))]
    fn run_x_corr(
        &mut self,
        image: imports::ImageBuffer<imports::Luma<u8>, Vec<u8>>,
        precision: f32,
    ) -> Result<Option<Vec<(u32, u32, f32)>>, AutoGuiError> {
        let match_mode = self.template_data.match_mode.clone().ok_or(ImageProcessingError::new("No template chosen and no template data prepared. Please run load_and_prepare_template before searching image on screen"))?;
        let start = std::time::Instant::now();
        let found_locations: Vec<(u32, u32, f32)> = match match_mode {
            MatchMode::FFT => {
                println!("Running FFT mode");
                let data = match &self.template_data.prepared_data {
                    imports::PreparedData::FFT(data) => data,
                    _ => Err(ImageProcessingError::new(
                        "error in prepared data type. Matchmode does not match prepare data type",
                    ))?,
                };
                let found_locations: Vec<(u32, u32, f64)> =
                    template_match::fft_ncc::fft_ncc(&image, precision, data);
                found_locations
                    .into_iter()
                    .map(|(x, y, value)| (x, y, value as f32))
                    .collect()
            }
            MatchMode::Segmented => {
                println!("Running Segmented mode");
                let data = match &self.template_data.prepared_data {
                    imports::PreparedData::Segmented(data) => data,
                    _ => Err(ImageProcessingError::new(
                        "error in prepared data type. Matchmode does not match prepare data type",
                    ))?,
                };
                template_match::segmented_ncc::fast_ncc_template_match(
                    &image,
                    precision,
                    data,
                    &self.debug,
                )
            }
            #[cfg(feature = "opencl")]
            MatchMode::SegmentedOcl => {
                let data = match &self.template_data.prepared_data {
                    imports::PreparedData::Segmented(data) => data,
                    _ => Err(ImageProcessingError::new(
                        "error in prepared data type. Matchmode does not match prepare data type",
                    ))?,
                };
                let gpu_memory_pointers = self
                    .opencl_data
                    .ocl_buffer_storage
                    .get(&self.template_data.alias_used)
                    .ok_or(ImageProcessingError::new("Error , no OCL data prepared"))?;
                template_match::open_cl::gui_opencl_ncc_template_match(
                    &self.opencl_data.ocl_queue,
                    &self.opencl_data.ocl_program,
                    self.opencl_data.ocl_workgroup_size,
                    &self.opencl_data.ocl_kernel_storage[&self.template_data.alias_used],
                    gpu_memory_pointers,
                    precision,
                    &image,
                    data,
                    imports::OclVersion::V1,
                )?
            }
            #[cfg(feature = "opencl")]
            MatchMode::SegmentedOclV2 => {
                let data = match &self.template_data.prepared_data {
                    imports::PreparedData::Segmented(data) => data,
                    _ => Err(ImageProcessingError::new(
                        "error in prepared data type. Matchmode does not match prepare data type",
                    ))?,
                };
                let gpu_memory_pointers = self
                    .opencl_data
                    .ocl_buffer_storage
                    .get(&self.template_data.alias_used)
                    .ok_or(ImageProcessingError::new("Error , no OCL data prepared"))?;
                template_match::open_cl::gui_opencl_ncc_template_match(
                    &self.opencl_data.ocl_queue,
                    &self.opencl_data.ocl_program,
                    self.opencl_data.ocl_workgroup_size,
                    &self.opencl_data.ocl_kernel_storage[&self.template_data.alias_used],
                    gpu_memory_pointers,
                    precision,
                    &image,
                    data,
                    imports::OclVersion::V2,
                )?
            }
        };
        if found_locations.len() > 0 {
            if self.debug {
                let corrected_found_location: (u32, u32, f32);
                let x = found_locations[0].0 as u32
                    + (self.template_width / 2) as u32
                    + self.template_data.region.0 as u32;
                let y = found_locations[0].1 as u32
                    + (self.template_height / 2) as u32
                    + self.template_data.region.1 as u32;
                let corr = found_locations[0].2;
                corrected_found_location = (x, y, corr);

                println!(
                    "Location found at x: {}, y {}, corr {} ",
                    corrected_found_location.0,
                    corrected_found_location.1,
                    corrected_found_location.2
                )
            }
            return Ok(Some(found_locations));
        } else {
            return Ok(None);
        };
    }

    //////////////////////////////// MOUSE ////////////////////////////////////////

    pub fn get_mouse_position(&self) -> Result<(i32, i32), AutoGuiError> {
        #[cfg(target_os = "linux")]
        return self.mouse.get_mouse_position();
        #[cfg(target_os = "windows")]
        return Ok(imports::Mouse::get_mouse_position());
        #[cfg(target_os = "macos")]
        return imports::Mouse::get_mouse_position();
    }

    /// Move mouse to x,y pixel coordinate
    pub fn move_mouse_to_pos(&self, x: u32, y: u32, moving_time: f32) -> Result<(), AutoGuiError> {
        if (x as i32 > self.screen.screen_width) | (y as i32 > self.screen.screen_height) {
            return Err(AutoGuiError::OutOfBoundsError(format!(
                "Out of bounds at positions x,y :{}, {}",
                x, y
            )));
        }

        #[cfg(target_os = "windows")]
        {
            imports::Mouse::move_mouse_to_pos(x as i32, y as i32, moving_time);
            Ok(())
        }
        #[cfg(target_os = "linux")]
        return self
            .mouse
            .move_mouse_to_pos(x as i32, y as i32, moving_time);
        #[cfg(target_os = "macos")]
        return imports::Mouse::move_mouse_to_pos(x as i32, y as i32, moving_time);
    }

    /// Very similar to move mouse to pos, but takes Option<x> and Option<y>, where None value just keeps the current mouse x or y value
    /// So in case you want to more easily move mouse horizontally or vertically
    pub fn move_mouse_to(
        &self,
        x: Option<u32>,
        y: Option<u32>,
        moving_time: f32,
    ) -> Result<(), AutoGuiError> {
        let (pos_x, pos_y) = self.get_mouse_position()?;

        let x = if let Some(x) = x { x as i32 } else { pos_x };

        let y = if let Some(y) = y { y as i32 } else { pos_y };

        if (x > self.screen.screen_width) | (y > self.screen.screen_height) {
            return Err(AutoGuiError::OutOfBoundsError(format!(
                "Out of bounds at positions x,y :{}, {}",
                x, y
            )));
        }

        #[cfg(target_os = "windows")]
        {
            imports::Mouse::move_mouse_to_pos(x, y, moving_time);
            Ok(())
        }
        #[cfg(target_os = "linux")]
        return self.mouse.move_mouse_to_pos(x, y, moving_time);
        #[cfg(target_os = "macos")]
        return imports::Mouse::move_mouse_to_pos(x, y, moving_time);
    }

    /// Move mouse in relative position. Accepts both positive and negative values, where negative X moves left, positive moves right
    /// and negative Y moves up, positive down
    pub fn move_mouse(&self, x: i32, y: i32, moving_time: f32) -> Result<(), AutoGuiError> {
        let (pos_x, pos_y) = self.get_mouse_position()?;

        let x = x + pos_x;
        let y = y + pos_y;

        if (x > self.screen.screen_width) | (y > self.screen.screen_height) | (x < 0) | (y < 0) {
            return Err(AutoGuiError::OutOfBoundsError(
                format!("Out of bounds at positions x,y :{}, {}", x, y), // "Mouse movement out of screen boundaries".to_string(),
            ));
        }

        #[cfg(target_os = "windows")]
        {
            imports::Mouse::move_mouse_to_pos(x, y, moving_time);
            Ok(())
        }
        #[cfg(target_os = "linux")]
        return self.mouse.move_mouse_to_pos(x, y, moving_time);
        #[cfg(target_os = "macos")]
        return imports::Mouse::move_mouse_to_pos(x, y, moving_time);
    }

    /// executes left click down, move to position relative to current position, left click up
    pub fn drag_mouse(&self, x: i32, y: i32, moving_time: f32) -> Result<(), AutoGuiError> {
        let (pos_x, pos_y) = self.get_mouse_position()?;

        let x = x + pos_x;
        let y = y + pos_y;
        if (x > self.screen.screen_width) | (y > self.screen.screen_height) | (x < 0) | (y < 0) {
            return Err(AutoGuiError::OutOfBoundsError(
                format!("Out of bounds at positions x,y :{}, {}", x, y), // "Mouse movement out of screen boundaries".to_string(),
            ));
        };
        #[cfg(target_os = "windows")]
        {
            imports::Mouse::drag_mouse(x as i32, y as i32, moving_time);

            Ok(())
        }
        #[cfg(target_os = "macos")]
        {
            if moving_time < 0.5 && !self.suppress_warnings {
                eprintln!("WARNING:Small moving time values may cause issues on mouse drag");
            }
            return imports::Mouse::drag_mouse(x as i32, y as i32, moving_time);
        }
        #[cfg(target_os = "linux")]
        {
            if moving_time < 0.5 {
                if !self.suppress_warnings {
                    eprintln!("WARNING:Small moving time values may cause issues on mouse drag");
                }
            }
            return self.mouse.drag_mouse(x as i32, y as i32, moving_time);
        }
    }

    /// Moves to position x,y. None values maintain current position. Useful for vertical and horizontal movement
    pub fn drag_mouse_to(
        &self,
        x: Option<u32>,
        y: Option<u32>,
        moving_time: f32,
    ) -> Result<(), AutoGuiError> {
        let (pos_x, pos_y) = self.get_mouse_position()?;

        let x = if let Some(x) = x { x as i32 } else { pos_x };

        let y = if let Some(y) = y { y as i32 } else { pos_y };

        if (x > self.screen.screen_width) | (y > self.screen.screen_height) {
            return Err(AutoGuiError::OutOfBoundsError(format!(
                "Out of bounds at positions x,y :{}, {}",
                x, y
            )));
        }
        #[cfg(target_os = "windows")]
        {
            imports::Mouse::drag_mouse(x as i32, y as i32, moving_time);

            Ok(())
        }
        #[cfg(target_os = "macos")]
        {
            if moving_time < 0.5 && !self.suppress_warnings {
                eprintln!("WARNING:Small moving time values may cause issues on mouse drag");
            }
            return imports::Mouse::drag_mouse(x as i32, y as i32, moving_time);
        }
        #[cfg(target_os = "linux")]
        {
            if moving_time < 0.5 {
                if !self.suppress_warnings {
                    eprintln!("WARNING:Small moving time values may cause issues on mouse drag");
                }
            }
            return self.mouse.drag_mouse(x as i32, y as i32, moving_time);
        }
    }

    /// moves mouse to x, y pixel coordinate
    pub fn drag_mouse_to_pos(&self, x: u32, y: u32, moving_time: f32) -> Result<(), AutoGuiError> {
        if (x as i32 > self.screen.screen_width) | (y as i32 > self.screen.screen_height) {
            return Err(AutoGuiError::OutOfBoundsError(
                "Drag Mouse out of screen boundaries".to_string(),
            ));
        }

        #[cfg(target_os = "windows")]
        {
            imports::Mouse::drag_mouse(x as i32, y as i32, moving_time);

            Ok(())
        }
        #[cfg(target_os = "macos")]
        {
            if moving_time < 0.5 && !self.suppress_warnings {
                eprintln!("WARNING:Small moving time values may cause issues on mouse drag");
            }
            return imports::Mouse::drag_mouse(x as i32, y as i32, moving_time);
        }
        #[cfg(target_os = "linux")]
        {
            if moving_time < 0.5 {
                if !self.suppress_warnings {
                    eprintln!("WARNING:Small moving time values may cause issues on mouse drag");
                }
            }
            return self.mouse.drag_mouse(x as i32, y as i32, moving_time);
        }
    }

    /// Mouse click. Choose button Mouseclick::{LEFT,RIGHT,MIDDLE}
    pub fn click(&self, button: MouseClick) -> Result<(), AutoGuiError> {
        #[cfg(target_os = "linux")]
        return self.mouse.mouse_click(button);
        #[cfg(target_os = "windows")]
        return Ok(mouse::platform::Mouse::mouse_click(button));
        #[cfg(target_os = "macos")]
        return mouse::platform::Mouse::mouse_click(button);
    }

    /// executes left mouse click
    pub fn left_click(&self) -> Result<(), AutoGuiError> {
        #[cfg(target_os = "linux")]
        return self.mouse.mouse_click(mouse::MouseClick::LEFT);
        #[cfg(target_os = "windows")]
        return Ok(mouse::platform::Mouse::mouse_click(mouse::MouseClick::LEFT));
        #[cfg(target_os = "macos")]
        return mouse::platform::Mouse::mouse_click(mouse::MouseClick::LEFT);
    }

    /// executes right mouse click
    pub fn right_click(&self) -> Result<(), AutoGuiError> {
        #[cfg(target_os = "linux")]
        return self.mouse.mouse_click(mouse::MouseClick::RIGHT);
        #[cfg(target_os = "macos")]
        return mouse::platform::Mouse::mouse_click(mouse::MouseClick::RIGHT);
        #[cfg(target_os = "windows")]
        return Ok(mouse::platform::Mouse::mouse_click(
            mouse::MouseClick::RIGHT,
        ));
    }

    /// executes middle mouse click
    pub fn middle_click(&self) -> Result<(), AutoGuiError> {
        #[cfg(target_os = "linux")]
        return self.mouse.mouse_click(mouse::MouseClick::MIDDLE);
        #[cfg(target_os = "windows")]
        return Ok(mouse::platform::Mouse::mouse_click(
            mouse::MouseClick::MIDDLE,
        ));
        #[cfg(target_os = "macos")]
        return mouse::platform::Mouse::mouse_click(mouse::MouseClick::MIDDLE);
    }

    /// executes double left mouse click
    pub fn double_click(&self) -> Result<(), AutoGuiError> {
        #[cfg(target_os = "linux")]
        {
            self.mouse.mouse_click(mouse::MouseClick::LEFT)?;
            return self.mouse.mouse_click(mouse::MouseClick::LEFT);
        }
        #[cfg(target_os = "windows")]
        {
            mouse::platform::Mouse::mouse_click(mouse::MouseClick::LEFT);
            mouse::platform::Mouse::mouse_click(mouse::MouseClick::LEFT);
            return Ok(());
        }
        #[cfg(target_os = "macos")]
        return mouse::platform::Mouse::double_click();
    }

    pub fn click_down(&self, button: MouseClick) -> Result<(), AutoGuiError> {
        #[cfg(target_os = "linux")]
        return self.mouse.mouse_down(button);
        #[cfg(target_os = "macos")]
        return mouse::platform::Mouse::mouse_down(button);
        #[cfg(target_os = "windows")]
        return Ok(mouse::platform::Mouse::mouse_down(button));
    }
    pub fn click_up(&self, button: MouseClick) -> Result<(), AutoGuiError> {
        #[cfg(target_os = "linux")]
        return self.mouse.mouse_up(button);
        #[cfg(target_os = "macos")]
        return mouse::platform::Mouse::mouse_up(button);
        #[cfg(target_os = "windows")]
        return Ok(mouse::platform::Mouse::mouse_up(button));
    }

    pub fn scroll_up(&self, intensity: u32) -> Result<(), AutoGuiError> {
        #[cfg(target_os = "linux")]
        return Ok(self.mouse.scroll(mouse::MouseScroll::UP, intensity));
        #[cfg(target_os = "windows")]
        return Ok(mouse::platform::Mouse::scroll(
            mouse::MouseScroll::UP,
            intensity,
        ));
        #[cfg(target_os = "macos")]
        return mouse::platform::Mouse::scroll(mouse::MouseScroll::UP, intensity);
    }

    pub fn scroll_down(&self, intensity: u32) -> Result<(), AutoGuiError> {
        #[cfg(target_os = "linux")]
        return Ok(self.mouse.scroll(mouse::MouseScroll::DOWN, intensity));
        #[cfg(target_os = "windows")]
        return Ok(mouse::platform::Mouse::scroll(
            mouse::MouseScroll::DOWN,
            intensity,
        ));
        #[cfg(target_os = "macos")]
        return mouse::platform::Mouse::scroll(mouse::MouseScroll::DOWN, intensity);
    }

    pub fn scroll_left(&self, intensity: u32) -> Result<(), AutoGuiError> {
        #[cfg(target_os = "linux")]
        return Ok(self.mouse.scroll(mouse::MouseScroll::LEFT, intensity));
        #[cfg(target_os = "windows")]
        return Ok(mouse::platform::Mouse::scroll(
            mouse::MouseScroll::LEFT,
            intensity,
        ));
        #[cfg(target_os = "macos")]
        return mouse::platform::Mouse::scroll(mouse::MouseScroll::LEFT, intensity);
    }

    pub fn scroll_right(&self, intensity: u32) -> Result<(), AutoGuiError> {
        #[cfg(target_os = "linux")]
        return Ok(self.mouse.scroll(mouse::MouseScroll::RIGHT, intensity));
        #[cfg(target_os = "windows")]
        return Ok(mouse::platform::Mouse::scroll(
            mouse::MouseScroll::RIGHT,
            intensity,
        ));
        #[cfg(target_os = "macos")]
        return mouse::platform::Mouse::scroll(mouse::MouseScroll::RIGHT, intensity);
    }

    //////////////////// Keyboard ////////////////////

    /// accepts string and mimics keyboard key presses for each character in string
    pub fn keyboard_input(&self, input: &str) -> Result<(), AutoGuiError> {
        let input_string = String::from(input);
        for letter in input_string.chars() {
            self.keyboard.send_char(&letter)?;
        }
        Ok(())
    }

    /// executes keyboard command like "return" or "escape"
    pub fn keyboard_command(&self, input: &str) -> Result<(), AutoGuiError> {
        let input_string = String::from(input);
        // return automatically the result of send_command function
        self.keyboard.send_command(&input_string)
    }

    pub fn keyboard_multi_key(
        &self,
        input1: &str,
        input2: &str,
        input3: Option<&str>,
    ) -> Result<(), AutoGuiError> {
        let input3 = match input3 {
            Some(x) => Some(String::from(x)),
            None => None,
        };
        // send automatically result of function
        self.keyboard.send_multi_key(input1, input2, input3)
    }

    pub fn key_down(&self, key: &str) -> Result<(), AutoGuiError> {
        self.keyboard.key_down(key)
    }

    pub fn key_up(&self, key: &str) -> Result<(), AutoGuiError> {
        self.keyboard.key_up(key)
    }
    #[cfg(not(feature = "lite"))]
    /// DEPRECATED
    #[deprecated(since = "2.2.0", note = "Renamed to prepare_template_from_file.")]
    pub fn load_and_prepare_template(
        &mut self,
        template_path: &str,
        region: Option<(u32, u32, u32, u32)>,
        match_mode: MatchMode,
    ) -> Result<(), AutoGuiError> {
        let template: imports::ImageBuffer<imports::Luma<u8>, Vec<u8>> =
            imgtools::load_image_bw(template_path)?;
        self.prepare_template_picture_bw(template, region, match_mode, None, None)
    }
}

#[cfg(target_os = "linux")]
impl Drop for RustAutoGui {
    fn drop(&mut self) {
        self.screen.destroy();
    }
}

#[cfg(not(feature="lite"))]
#[cfg(target_os = "windows")]
impl Drop for RustAutoGui {
    fn drop(&mut self) {
        self.screen.destroy();
    }
}