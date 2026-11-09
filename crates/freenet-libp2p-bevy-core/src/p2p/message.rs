use serde::Serialize;
use serde::de::DeserializeOwned;

pub trait Message: Serialize + DeserializeOwned + Clone + Send + Sync + 'static {}
impl<Payload> Message for Payload where
    Payload: Serialize + DeserializeOwned + Clone + Send + Sync + 'static
{
}

#[cfg(test)]
mod tests {
    use derive_more::Deref;
    use serde::{Deserialize, Serialize};

    use super::Message;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Deref)]
    struct Dummy(u32);

    #[test]
    fn test_usage() {
        fn assert_message<Payload: Message>() {}
        assert_message::<Dummy>();
    }
}
