#[cfg(not(feature = "lite"))]
use crate::core::template_match;
#[cfg(not(feature = "lite"))]
use crate::data::*;
#[cfg(not(feature = "lite"))]
use crate::imgtools;
#[cfg(not(feature = "lite"))]
use crate::{AutoGuiError, ImageProcessingError, MatchMode, DEFAULT_ALIAS, DEFAULT_BCKP_ALIAS};
#[cfg(not(feature = "lite"))]
use image::{
    imageops::{resize, FilterType::Nearest},
    ImageBuffer, Luma, Pixel, Primitive,
};
#[cfg(not(feature = "lite"))]
use rustfft::{num_complex::Complex, num_traits::ToPrimitive};
#[cfg(not(feature = "lite"))]
impl crate::RustAutoGui {
    #[cfg(not(feature = "lite"))]
    /// main prepare template picture which takes ImageBuffer Luma u8. all the other variants
    /// of prepare/store funtions call this function
    #[allow(unused_mut)]
    fn prepare_template_picture_bw(
        &mut self,
        mut template: ImageBuffer<Luma<u8>, Vec<u8>>,
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
                        template = resize(
                            &template,
                            template.width() / self.screen.screen_data.scaling_factor_x as u32,
                            template.height() / self.screen.screen_data.scaling_factor_y as u32,
                            Nearest,
                        );
                    }
                }
                None => {
                    template = resize(
                        &template,
                        template.width() / self.screen.screen_data.scaling_factor_x as u32,
                        template.height() / self.screen.screen_data.scaling_factor_y as u32,
                        Nearest,
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
                    PreparedData::FFT(template_match::fft_ncc::prepare_template_picture(
                        &template, region.2, region.3,
                    ));
                let match_mode = Some(MatchMode::FFT);
                (prepared_data, match_mode)
            }

            MatchMode::Segmented => {
                let prepared_data: PreparedData =
                    template_match::segmented_ncc::prepare_template_picture(
                        &template,
                        &self.debug,
                        user_threshold,
                    );
                if let PreparedData::Segmented(ref segmented) = prepared_data {
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
                let prepared_data: PreparedData =
                    template_match::segmented_ncc::prepare_template_picture(
                        &template,
                        &self.debug,
                        user_threshold,
                    );
                let prepared_data: SegmentedData = if let PreparedData::Segmented(segmented) =
                    prepared_data
                {
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
                    let ocl_buffer_data = GpuMemoryPointers::new(
                        region.2,
                        region.3,
                        template_width,
                        template_height,
                        &self.opencl_data.ocl_queue,
                        &prepared_data.template_segments_slow,
                        &prepared_data.template_segments_fast,
                    )?;

                    let kernels = KernelStorage::new(
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

                (PreparedData::Segmented(prepared_data), match_mode)
            }
        };

        // Alias Some -> storing the image , we just save it to Hashmap
        // Alias None -> not storing, then we change struct attributes to fit the single loaded image search
        match alias {
            Some(name) => {
                self.template_data
                    .prepared_data_stored
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
        Ok(())
    }

    #[cfg(not(feature = "lite"))]
    // prepares also unscaled variant of image if retina display is on
    // since it is recursively calling again preparation of template with another alias
    // checks are made on alias_name to not run infinitely preparations of backups of backups
    #[cfg(target_os = "macos")]
    fn prepare_macos_backup(
        &mut self,
        match_mode: &MatchMode,
        template: ImageBuffer<Luma<u8>, Vec<u8>>,
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
        let template: ImageBuffer<Luma<u8>, Vec<u8>> = imgtools::load_image_bw(template_path)?;
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
        let template: ImageBuffer<Luma<u8>, Vec<u8>> = imgtools::load_image_bw(template_path)?;
        self.prepare_template_picture_bw(template, region, match_mode, None, Some(threshold))
    }
    #[cfg(not(feature = "lite"))]
    /// prepare from imagebuffer, works only on types RGB/RGBA/Luma
    pub fn prepare_template_from_imagebuffer<P, T>(
        &mut self,
        image: ImageBuffer<P, Vec<T>>,
        region: Option<(u32, u32, u32, u32)>,
        match_mode: MatchMode,
    ) -> Result<(), AutoGuiError>
    where
        P: Pixel<Subpixel = T> + 'static,
        T: Primitive + ToPrimitive + 'static,
    {
        let color_scheme = imgtools::check_imagebuffer_color_scheme(&image)?;
        let luma_img = imgtools::convert_t_imgbuffer_to_luma(&image, color_scheme)?;
        self.prepare_template_picture_bw(luma_img, region, match_mode, None, None)?;
        Ok(())
    }

    #[cfg(not(feature = "lite"))]
    pub fn prepare_template_from_imagebuffer_custom<P, T>(
        &mut self,
        image: ImageBuffer<P, Vec<T>>,
        region: Option<(u32, u32, u32, u32)>,
        match_mode: MatchMode,
        threshold: f32,
    ) -> Result<(), AutoGuiError>
    where
        P: Pixel<Subpixel = T> + 'static,
        T: Primitive + ToPrimitive + 'static,
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
        let template: ImageBuffer<Luma<u8>, Vec<u8>> = imgtools::load_image_bw(template_path)?;
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
        let template: ImageBuffer<Luma<u8>, Vec<u8>> = imgtools::load_image_bw(template_path)?;
        self.prepare_template_picture_bw(template, region, match_mode, Some(alias), Some(threshold))
    }
    #[cfg(not(feature = "lite"))]
    /// Load template from imagebuffer and store prepared template data for multiple image search
    pub fn store_template_from_imagebuffer<P, T>(
        &mut self,
        image: ImageBuffer<P, Vec<T>>,
        region: Option<(u32, u32, u32, u32)>,
        match_mode: MatchMode,
        alias: &str,
    ) -> Result<(), AutoGuiError>
    where
        P: Pixel<Subpixel = T> + 'static,
        T: Primitive + ToPrimitive + 'static,
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
        image: ImageBuffer<P, Vec<T>>,
        region: Option<(u32, u32, u32, u32)>,
        match_mode: MatchMode,
        alias: &str,
        threshold: f32,
    ) -> Result<(), AutoGuiError>
    where
        P: Pixel<Subpixel = T> + 'static,
        T: Primitive + ToPrimitive + 'static,
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
    #[cfg(not(feature = "lite"))]
    /// DEPRECATED
    #[deprecated(since = "2.2.0", note = "Renamed to prepare_template_from_file.")]
    pub fn load_and_prepare_template(
        &mut self,
        template_path: &str,
        region: Option<(u32, u32, u32, u32)>,
        match_mode: MatchMode,
    ) -> Result<(), AutoGuiError> {
        let template: ImageBuffer<Luma<u8>, Vec<u8>> = imgtools::load_image_bw(template_path)?;
        self.prepare_template_picture_bw(template, region, match_mode, None, None)
    }
}
