use async_trait::async_trait;
use derive_more::Deref;
use libp2p::StreamProtocol;
use libp2p::request_response::Codec;

use crate::p2p;

#[derive(Clone, Debug, Deref)]
pub struct MessageCodec<T>(std::marker::PhantomData<T>);

impl<T> Default for MessageCodec<T> {
    fn default() -> Self {
        Self(std::marker::PhantomData)
    }
}

#[async_trait]
impl<T: p2p::Message> Codec for MessageCodec<T> {
    type Protocol = StreamProtocol;
    type Request = T;
    type Response = T;

    async fn read_request<R>(
        &mut self,
        _: &StreamProtocol,
        io: &mut R,
    ) -> std::io::Result<Self::Request>
    where
        R: futures::AsyncRead + Unpin + Send,
    {
        let mut buf = Vec::new();
        futures::AsyncReadExt::read_to_end(io, &mut buf).await?;
        bincode::deserialize(&buf)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    async fn read_response<R>(
        &mut self,
        _: &StreamProtocol,
        io: &mut R,
    ) -> std::io::Result<Self::Response>
    where
        R: futures::AsyncRead + Unpin + Send,
    {
        let mut buf = Vec::new();
        futures::AsyncReadExt::read_to_end(io, &mut buf).await?;
        bincode::deserialize(&buf)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    async fn write_request<W>(
        &mut self,
        _: &StreamProtocol,
        io: &mut W,
        req: Self::Request,
    ) -> std::io::Result<()>
    where
        W: futures::AsyncWrite + Unpin + Send,
    {
        let bytes = bincode::serialize(&req)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        futures::AsyncWriteExt::write_all(io, &bytes).await?;
        futures::AsyncWriteExt::close(io).await
    }

    async fn write_response<W>(
        &mut self,
        _: &StreamProtocol,
        io: &mut W,
        res: Self::Response,
    ) -> std::io::Result<()>
    where
        W: futures::AsyncWrite + Unpin + Send,
    {
        let bytes = bincode::serialize(&res)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        futures::AsyncWriteExt::write_all(io, &bytes).await?;
        futures::AsyncWriteExt::close(io).await
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    use super::MessageCodec;
    use derive_more::Deref;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Deref)]
    struct Dummy(u32);

    #[test]
    fn test_usage() {
        let _c = MessageCodec::<Dummy>::default();
    }
}
