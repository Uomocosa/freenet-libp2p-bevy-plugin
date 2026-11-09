use std::time::Duration;

pub const QUIT_PUBLISH_GRACE: Duration = Duration::from_secs(2);

pub const SCRIPT_FAILED_EXIT_CODE: u8 = 3;

pub const ROOM_PLACEHOLDER: &str = "{room}";

pub const STEP_DONE_MARKER: &str = "script step done step=";

pub const SCRIPT_DONE_MARKER: &str = "script done";

pub const SCRIPT_FAILED_MARKER: &str = "script failed";

pub const SCRIPT_CHECKPOINT_COMPLETED_PREFIX: &str = "script checkpoint completed name=";

// no test_usage necessary
