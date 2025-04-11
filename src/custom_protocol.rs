use async_trait::async_trait;
use futures::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use libp2p::core::upgrade::ProtocolName;
use libp2p::request_response::Codec;
use std::io;

#[derive(Clone)]
pub struct TimeProtocol;

impl ProtocolName for TimeProtocol {
    fn protocol_name(&self) -> &[u8] {
        b"/time/reqrep/1.0.0"
    }
}

impl Default for TimeCodec {
    fn default() -> Self {
        TimeCodec
    }
}

#[derive(Clone)]
pub struct TimeCodec;

#[async_trait]
impl Codec for TimeCodec {
    type Protocol = TimeProtocol;
    type Request = Vec<u8>;
    type Response = Vec<u8>;

    async fn read_request<'life0, T>(
        &'life0 mut self,
        _: &Self::Protocol,
        io: &mut T,
    ) -> io::Result<Self::Request>
    where
        T: AsyncRead + Unpin + Send + 'life0,
    {
        let mut buffer = vec![];
        io.read_to_end(&mut buffer).await?;
        Ok(buffer)
    }

    async fn read_response<'life0, T>(
        &'life0 mut self,
        _: &Self::Protocol,
        io: &mut T,
    ) -> io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send + 'life0,
    {
        let mut buffer = vec![];
        io.read_to_end(&mut buffer).await?;
        Ok(buffer)
    }

    async fn write_request<'life0, T>(
        &'life0 mut self,
        _: &Self::Protocol,
        io: &mut T,
        req: Self::Request,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send + 'life0,
    {
        io.write_all(&req).await?;
        io.flush().await?;
        Ok(())
    }

    async fn write_response<'life0, T>(
        &'life0 mut self,
        _: &Self::Protocol,
        io: &mut T,
        res: Self::Response,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send + 'life0,
    {
        io.write_all(&res).await?;
        io.flush().await?;
        Ok(())
    }
}