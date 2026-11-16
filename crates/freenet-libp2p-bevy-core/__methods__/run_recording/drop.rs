use crate::testing;

pub fn drop(recording: &mut testing::harness::RunRecording) {
    let Some(process) = recording.process.take() else {
        return;
    };
    let elapsed = recording.started.elapsed();
    if std::thread::panicking() {
        recording.outcome = testing::harness::RecordingOutcome::Failed;
    }
    testing::harness::RunRecording::stop_capture(process);
    let clip = recording.clip_path();
    match testing::harness::RunRecording::fit_under_budget(&clip) {
        Ok(clip) => testing::harness::RunRecording::deliver(
            &recording.test_name,
            recording.outcome,
            elapsed,
            &clip,
        ),
        Err(error) => eprintln!("e2e recording: {error}"),
    }
}

// no test_usage necessary
