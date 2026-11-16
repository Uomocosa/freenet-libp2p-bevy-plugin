use crate::testing;

pub fn drop(display: &mut testing::harness::E2EDisplay) {
    let Some(mut child) = display.child.take() else {
        return;
    };
    let _ = child.kill();
    let _ = child.wait();
    let _ = std::fs::remove_file(testing::harness::E2EDisplay::socket_path(&display.name));
    let _ = std::fs::remove_file(testing::harness::E2EDisplay::lock_path(&display.name));
}

// no test_usage necessary
