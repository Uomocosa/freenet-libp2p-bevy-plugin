use std::path::Path;

use crate::testing;

pub fn from_env(run_dir: &Path) -> Result<Option<testing::harness::E2EDisplay>, String> {
    if let Some(name) = testing::harness::E2EDisplay::reused_display_name() {
        return Ok(Some(testing::harness::E2EDisplay { name, child: None }));
    }
    testing::harness::E2EDisplay::start_virtual_display(run_dir)
}

// no test_usage necessary
