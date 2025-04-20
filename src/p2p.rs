use libp2p::{
    identity, noise, request_response::{self, Behaviour, Config, ProtocolSupport}, tcp, yamux, SwarmBuilder
};

use futures::{AsyncReadExt, AsyncWriteExt, AsyncRead, AsyncWrite};
use std::{io, iter, time::Duration};
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MyProtocol();

impl AsRef<str> for MyProtocol {
    fn as_ref(&self) -> &str {
        "/my-custom-protocol/1.0.0"
    }
}

// 2. Define Request and Response Types using Vec<u8>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MyRequest(pub Vec<u8>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MyResponse(pub Vec<u8>);

// 3. Define the Codec for serialization/deserialization (Updated for Vec<u8>)
#[derive(Clone)]
pub struct MyCodec;

const MAX_SIZE: usize = 1_024 * 1_024; // Max message size 1MB

#[derive(Debug, Error)]
pub enum CodecError {
    #[error("IO Error: {0}")]
    Io(#[from] io::Error),
    #[error("Message too large: {0} bytes")]
    TooLarge(usize),
}

#[async_trait::async_trait]
impl request_response::Codec for MyCodec {
    type Protocol = MyProtocol;
    type Request = MyRequest;
    type Response = MyResponse;

    // Read a request (length-prefixed bytes)
    async fn read_request<T>(&mut self, _: &MyProtocol, io: &mut T) -> io::Result<Self::Request>
    where
        T: AsyncRead + Unpin + Send,
    {
        // Read the length prefix (u32, 4 bytes)
        let mut len_bytes = [0u8; 4];
        io.read_exact(&mut len_bytes).await?;
        let len = u32::from_be_bytes(len_bytes) as usize;

        if len > MAX_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                CodecError::TooLarge(len),
            ));
        }

        // Read the actual data
        let mut buffer = vec![0u8; len];
        io.read_exact(&mut buffer).await?;

        Ok(MyRequest(buffer)) // Wrap directly in MyRequest
    }

    // Read a response (length-prefixed bytes)
    async fn read_response<T>(&mut self, _: &MyProtocol, io: &mut T) -> io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        // Read the length prefix (u32, 4 bytes)
        let mut len_bytes = [0u8; 4];
        io.read_exact(&mut len_bytes).await?;
        let len = u32::from_be_bytes(len_bytes) as usize;

        if len > MAX_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                CodecError::TooLarge(len),
            ));
        }

        // Read the actual data
        let mut buffer = vec![0u8; len];
        io.read_exact(&mut buffer).await?;

        Ok(MyResponse(buffer)) // Wrap directly in MyResponse
    }

    // Write a request (length-prefixed bytes)
    async fn write_request<T>(
        &mut self,
        _: &MyProtocol,
        io: &mut T,
        MyRequest(data): Self::Request, // Destructure here
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        let len = data.len() as u32;

        if len as usize > MAX_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                CodecError::TooLarge(len as usize),
            ));
        }

        // Write length prefix
        io.write_all(&len.to_be_bytes()).await?;
        // Write data
        io.write_all(&data).await?;
        io.flush().await?; // Ensure data is sent
        Ok(())
    }

    // Write a response (length-prefixed bytes)
    async fn write_response<T>(
        &mut self,
        _: &MyProtocol,
        io: &mut T,
        MyResponse(data): Self::Response, // Destructure here
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        let len = data.len() as u32;

        if len as usize > MAX_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                CodecError::TooLarge(len as usize),
            ));
        }

        // Write length prefix
        io.write_all(&len.to_be_bytes()).await?;
        // Write data
        io.write_all(&data).await?;
        io.flush().await?; // Ensure data is sent
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum P2pError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Swarm build error: {0}")]
    SwarmBuild(String),
}

pub async fn build_swarm(
    keypair: identity::Keypair,
) -> Result<libp2p::Swarm<Behaviour<MyCodec>>, P2pError> {
    let protocol = MyProtocol();
    let rr_config = Config::default().with_request_timeout(Duration::from_secs(10));
    let protocols = iter::once((protocol.clone(), ProtocolSupport::Full));
    let request_response = Behaviour::with_codec(MyCodec, protocols, rr_config);
    let swarm = SwarmBuilder::with_existing_identity(keypair)
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        ).map_err(|e| P2pError::SwarmBuild(e.to_string()))?
        .with_behaviour(|_| Ok(request_response))
        .map_err(|e| P2pError::SwarmBuild(e.to_string()))?
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(60)))
        .build();

    println!("Local peer ID: {}", swarm.local_peer_id());
    Ok(swarm)
}