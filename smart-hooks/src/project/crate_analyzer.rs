/// Crate analysis functionality
use anyhow::Result;
use std::path::Path;

use crate::project::types::CrateType;

/// Analyzes individual crates within a Rust project
pub struct CrateAnalyzer;

impl CrateAnalyzer {
    /// Determine if a crate is library, binary, or both
    pub fn determine_crate_type(src_path: &Path) -> Result<CrateType> {
        let has_lib = src_path.join("lib.rs").exists();
        let has_main = src_path.join("main.rs").exists();
        let has_bin = src_path.join("bin").exists();

        match (has_lib, has_main || has_bin) {
            (true, true) => Ok(CrateType::Mixed),
            (true, false) => Ok(CrateType::Library),
            (false, true) => Ok(CrateType::Binary),
            (false, false) => {
                // If neither lib.rs nor main.rs exists, assume it's a library crate that's not yet implemented
                // This allows project discovery to work with empty src directories
                Ok(CrateType::Library)
            }
        }
    }
}
