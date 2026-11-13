use crate::testing;

pub fn poll_exits(running: &mut [testing::RunningPeer]) {
    for slot in running.iter_mut() {
        let Some(child) = slot.child.as_mut() else {
            continue;
        };
        if slot.exit.is_some() {
            continue;
        }
        if let Ok(Some(status)) = child.try_wait() {
            slot.exit = Some(
                status
                    .code()
                    .map_or(testing::PeerExit::Signal, testing::PeerExit::Code),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::poll_exits;
    use crate::testing;

    #[test]
    fn test_usage() {
        let mut exited = std::process::Command::new("sh")
            .args(["-c", "exit 3"])
            .spawn()
            .expect("spawn sh");
        exited.wait().expect("wait for sh");
        let mut running = vec![
            testing::RunningPeer {
                log: PathBuf::new(),
                child: Some(exited),
                exit: None,
            },
            testing::RunningPeer {
                log: PathBuf::new(),
                child: None,
                exit: None,
            },
        ];
        poll_exits(&mut running);
        assert_eq!(running[0].exit, Some(testing::PeerExit::Code(3)));
        assert_eq!(running[1].exit, None);
    }
}
