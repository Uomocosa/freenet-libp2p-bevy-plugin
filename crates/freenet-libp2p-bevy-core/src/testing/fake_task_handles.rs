use bevy::prelude::Resource;

#[derive(Resource, Default)]
pub struct FakeTaskHandles {
    pub(crate) swarm: Option<tokio::task::JoinHandle<()>>,
    pub(crate) lobby: Option<tokio::task::JoinHandle<()>>,
}

impl FakeTaskHandles {
    pub fn abort_all(&mut self) {
        if let Some(handle) = self.swarm.take() {
            handle.abort();
        }
        if let Some(handle) = self.lobby.take() {
            handle.abort();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::FakeTaskHandles;

    #[test]
    fn test_usage() {
        let mut handles = FakeTaskHandles::default();
        assert!(handles.swarm.is_none());
        handles.abort_all();
        assert!(handles.lobby.is_none());
    }
}
