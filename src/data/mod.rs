#[cfg(feature = "opencl")]
pub mod opencl;

use crate::MatchMode;
use crate::RustAutoGui;
#[cfg(feature = "opencl")]
pub use opencl::*;

use rustfft::{num_complex::Complex, num_traits::ToPrimitive};

use crate::core::CaptureableScreen;

use image::{ImageBuffer, Luma};

use std::collections::HashMap;

pub struct Template {
    pub prepared_data: PreparedData, // used direct load and search
    pub region: (u32, u32, u32, u32),
    pub match_mode: Option<MatchMode>,
    pub width: u32,
    pub height: u32,
}

impl Template {
    pub fn new(
        prepared_data: PreparedData,
        region: (u32, u32, u32, u32),
        match_mode: MatchMode,
        dimension: (u32, u32),
    ) -> Self {
        Template {
            prepared_data,
            region,
            match_mode: Some(match_mode),
            width: dimension.0,
            height: dimension.1,
        }
    }
}

#[derive(Clone)]
pub enum PreparedData {
    Segmented(SegmentedData),
    FFT(FFTData),
    None,
}

#[derive(Clone)]
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
