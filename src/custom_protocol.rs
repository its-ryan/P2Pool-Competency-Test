use async_trait::async_trait;
use libp2p::request_response::{ProtocolName, RequestResponseCodec};
use std::io;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

#[derive(Clone)]
pub struct TimeProtocol();

impl ProtocolName for TimeProtocol {
    fn protocol_name(&self) -> &[u8] {
        b"/custom/time/1.0"
    }
}

#[derive(Debug, Clone)]
pub struct TimeRequest(pub Vec<u8>); // could be empty or contain requester info

#[derive(Debug, Clone)]
pub struct TimeResponse(pub Vec<u8>); // UTF-8 string of current time

#[derive(Clone)]
pub struct TimeCodec();

#[async_trait]
impl RequestResponseCodec for TimeCodec {
    type Protocol = TimeProtocol;
    type Request = TimeRequest;
    type Response = TimeResponse;

    async fn read_request<T>(&mut self, _: &Self::Protocol, io: &mut T) -> io::Result<TimeRequest>
    where
        T: AsyncRead + Unpin + Send,
    {
        let mut buf = Vec::new();
        io.read_to_end(&mut buf).await?;
        Ok(TimeRequest(buf))
    }

    async fn read_response<T>(&mut self, _: &Self::Protocol, io: &mut T) -> io::Result<TimeResponse>
    where
        T: AsyncRead + Unpin + Send,
    {
        let mut buf = Vec::new();
        io.read_to_end(&mut buf).await?;
        Ok(TimeResponse(buf))
    }

    async fn write_request<T>(&mut self, _: &Self::Protocol, io: &mut T, TimeRequest(data): TimeRequest) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        io.write_all(&data).await?;
        io.shutdown().await
    }

    async fn write_response<T>(&mut self, _: &Self::Protocol, io: &mut T, TimeResponse(data): TimeResponse) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        io.write_all(&data).await?;
        io.shutdown().await
    }
}
