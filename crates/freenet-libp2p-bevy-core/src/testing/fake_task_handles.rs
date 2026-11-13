use bevy::prelude::Resource;

#[derive(Resource, Default)]
pub struct FakeTaskHandles {
    pub(crate) swarm: Option<tokio::task::JoinHandle<()>>,
    pub(crate) extra: Vec<tokio::task::JoinHandle<()>>,
}

#[rustfmt::skip]
impl FakeTaskHandles {
    pub fn adopt(&mut self, handle: tokio::task::JoinHandle<()>) { self.extra.push(handle); }
}

impl FakeTaskHandles {
    pub fn abort_all(&mut self) {
        if let Some(handle) = self.swarm.take() {
            handle.abort();
        }
        for handle in self.extra.drain(..) {
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
        assert!(handles.extra.is_empty());
    }
}
