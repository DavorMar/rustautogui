#[cfg(feature = "opencl")]
pub mod opencl;
#[cfg(feature = "opencl")]
pub use opencl::*;
use rustfft::{num_complex::Complex, num_traits::ToPrimitive};
use crate::{MatchMode, RustAutoGui};

pub struct BackupData {
    pub starting_data: PreparedData2,
    pub starting_region: (u32, u32, u32, u32),
    pub starting_match_mode: Option<MatchMode>,
    pub starting_template_height: u32,
    pub starting_template_width: u32,
    pub starting_alias_used: String,
}
impl BackupData {
    pub fn update_rustautogui(self, target: &mut RustAutoGui) {
        target.prepared_data = self.starting_data.clone();
        target.region = self.starting_region;
        target.match_mode = self.starting_match_mode;
        target.screen.screen_region_width = self.starting_region.2;
        target.screen.screen_region_height = self.starting_region.3;
        target.template_width = self.starting_template_width;
        target.template_height = self.starting_template_height;
        target.alias_used = self.starting_alias_used;
    }
}



pub enum PreparedData2 {
    Segmented(SegmentedData),
    FFT(FFTData),
    None,
}
impl Clone for PreparedData2 {
    fn clone(&self) -> Self {
        match self {
            PreparedData2::Segmented(data) => PreparedData2::Segmented(data.clone()),
            PreparedData2::FFT(data) => PreparedData2::FFT(data.clone()),
            PreparedData2::None => PreparedData2::None,
        }
    }
}

pub struct SegmentedData{
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
            segments_mean_slow: self.segments_mean_slow
        }
    }
}

pub struct FFTData{
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