#[cfg(feature = "opencl")]
pub mod opencl;
#[cfg(not(feature = "lite"))]
use crate::MatchMode;
use crate::RustAutoGui;
#[cfg(feature = "opencl")]
pub use opencl::*;

use rustfft::{num_complex::Complex, num_traits::ToPrimitive};

#[cfg(not(feature = "lite"))]
use image::{ImageBuffer, Luma};
#[cfg(not(feature = "lite"))]
use std::collections::HashMap;

#[cfg(not(feature = "lite"))]
pub struct TemplateMatchingData {
    pub template: Option<ImageBuffer<Luma<u8>, Vec<u8>>>,
    pub prepared_data: PreparedData, // used direct load and search
    pub prepared_data_stored: HashMap<String, (PreparedData, (u32, u32, u32, u32), MatchMode)>, //prepared data, region, matchmode
    pub match_mode: Option<MatchMode>,
    pub region: (u32, u32, u32, u32),
    pub alias_used: String,
}

#[cfg(not(feature = "lite"))]
pub struct BackupData {
    pub starting_data: PreparedData,
    pub starting_region: (u32, u32, u32, u32),
    pub starting_match_mode: Option<MatchMode>,
    pub starting_template_height: u32,
    pub starting_template_width: u32,
    pub starting_alias_used: String,
}
#[cfg(not(feature = "lite"))]
impl BackupData {
    pub fn update_rustautogui(self, target: &mut RustAutoGui) {
        target.template_data.prepared_data = self.starting_data.clone();
        target.template_data.region = self.starting_region;
        target.template_data.match_mode = self.starting_match_mode;
        target.screen.screen_data.screen_region_width = self.starting_region.2;
        target.screen.screen_data.screen_region_height = self.starting_region.3;
        target.template_width = self.starting_template_width;
        target.template_height = self.starting_template_height;
        target.template_data.alias_used = self.starting_alias_used;
    }
}

pub enum PreparedData {
    Segmented(SegmentedData),
    FFT(FFTData),
    None,
}
impl Clone for PreparedData {
    fn clone(&self) -> Self {
        match self {
            PreparedData::Segmented(data) => PreparedData::Segmented(data.clone()),
            PreparedData::FFT(data) => PreparedData::FFT(data.clone()),
            PreparedData::None => PreparedData::None,
        }
    }
}

pub struct SegmentedData {
    pub template_segments_fast: Vec<(u32, u32, u32, u32, f32)>,
    pub template_segments_slow: Vec<(u32, u32, u32, u32, f32)>,
    pub template_width: u32,
    pub template_height: u32,
    pub segment_sum_squared_deviations_fast: f32,
    pub segment_sum_squared_deviations_slow: f32,
    pub expected_corr_fast: f32,
    pub expected_corr_slow: f32,
    pub segments_mean_fast: f32,
    pub segments_mean_slow: f32,
}

impl Clone for SegmentedData {
    fn clone(&self) -> Self {
        Self {
            template_segments_fast: self.template_segments_fast.clone(),
            template_segments_slow: self.template_segments_slow.clone(),
            template_width: self.template_width,
            template_height: self.template_height,
            segment_sum_squared_deviations_fast: self.segment_sum_squared_deviations_fast,
            segment_sum_squared_deviations_slow: self.segment_sum_squared_deviations_slow,
            expected_corr_fast: self.expected_corr_fast,
            expected_corr_slow: self.expected_corr_slow,
            segments_mean_fast: self.segments_mean_fast,
            segments_mean_slow: self.segments_mean_slow,
        }
    }
}

pub struct FFTData {
    pub template_conj_freq: Vec<Complex<f32>>,
    pub template_sum_squared_deviations: f32,
    pub template_width: u32,
    pub template_height: u32,
    pub padded_size: u32,
}
impl Clone for FFTData {
    fn clone(&self) -> Self {
        Self {
            template_conj_freq: self.template_conj_freq.clone(),
            template_sum_squared_deviations: self.template_sum_squared_deviations,
            template_width: self.template_width,
            template_height: self.template_height,
            padded_size: self.padded_size,
        }
    }
}
