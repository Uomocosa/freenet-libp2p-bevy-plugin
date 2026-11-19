use serde::Serialize;
use serde::de::DeserializeOwned;

pub trait Message: Serialize + DeserializeOwned + Clone + Send + Sync + 'static {}
impl<Payload> Message for Payload where
    Payload: Serialize + DeserializeOwned + Clone + Send + Sync + 'static
{
}

#[cfg(test)]
mod tests {
    use crate::testing;

    use super::Message;

    #[test]
    fn test_usage() {
        fn assert_message<Payload: Message>() {}
        assert_message::<testing::DummyMessage>();
    }
}
