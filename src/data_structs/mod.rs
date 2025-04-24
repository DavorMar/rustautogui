#[cfg(feature = "opencl")]
pub mod opencl;
#[cfg(feature = "opencl")]
pub use opencl::*;

use crate::{MatchMode, PreparedData, RustAutoGui};

pub struct BackupData {
    pub starting_data: PreparedData,
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
