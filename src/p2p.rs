use libp2p::{
    identity, 
    noise, 
    request_response::{self, Behaviour, Config, ProtocolSupport}, 
    tcp, yamux, SwarmBuilder
};

use futures::{AsyncReadExt, AsyncWriteExt, AsyncRead, AsyncWrite};
use std::{io, iter, time::Duration};
use thiserror::Error;

// Defines a custom protocol identifier.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MyProtocol();

impl AsRef<str> for MyProtocol {
    fn as_ref(&self) -> &str {
        "/my-custom-protocol/1.0.0"
    }
}

// Defines the request type as a vector of bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MyRequest(pub Vec<u8>);

// Defines the response type as a vector of bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MyResponse(pub Vec<u8>);

// Implements a custom codec for serializing and deserializing MyRequest and MyResponse.
#[derive(Clone)]
pub struct MyCodec;

const MAX_SIZE: usize = 1_024 * 1_024; // Maximum allowed message size (1MB).

// Defines potential errors that can occur during encoding or decoding.
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

    // Asynchronously reads a length-prefixed request from the given reader.
    async fn read_request<T>(&mut self, _: &MyProtocol, io: &mut T) -> io::Result<Self::Request>
    where
        T: AsyncRead + Unpin + Send,
    {
        // Reads the 4-byte length prefix.
        let mut len_bytes = [0u8; 4];
        io.read_exact(&mut len_bytes).await?;
        let len = u32::from_be_bytes(len_bytes) as usize;

        // Checks if the message length exceeds the maximum allowed size.
        if len > MAX_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                CodecError::TooLarge(len),
            ));
        }

        // Reads the actual data based on the length prefix.
        let mut buffer = vec![0u8; len];
        io.read_exact(&mut buffer).await?;

        // Returns the deserialized request.
        Ok(MyRequest(buffer))
    }

    // Asynchronously reads a length-prefixed response from the given reader.
    async fn read_response<T>(&mut self, _: &MyProtocol, io: &mut T) -> io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        // Reads the 4-byte length prefix.
        let mut len_bytes = [0u8; 4];
        io.read_exact(&mut len_bytes).await?;
        let len = u32::from_be_bytes(len_bytes) as usize;

        // Checks if the message length exceeds the maximum allowed size.
        if len > MAX_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                CodecError::TooLarge(len),
            ));
        }

        // Reads the actual data based on the length prefix.
        let mut buffer = vec![0u8; len];
        io.read_exact(&mut buffer).await?;

        // Returns the deserialized response.
        Ok(MyResponse(buffer))
    }

    // Asynchronously writes a length-prefixed request to the given writer.
    async fn write_request<T>(
        &mut self,
        _: &MyProtocol,
        io: &mut T,
        MyRequest(data): Self::Request, // Destructures the MyRequest to access the inner Vec<u8>.
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        let len = data.len() as u32;

        // Checks if the message length exceeds the maximum allowed size.
        if len as usize > MAX_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                CodecError::TooLarge(len as usize),
            ));
        }

        // Writes the 4-byte length prefix.
        io.write_all(&len.to_be_bytes()).await?;
        // Writes the actual data.
        io.write_all(&data).await?;
        // Ensures all buffered data is written to the underlying transport.
        io.flush().await?;
        Ok(())
    }

    // Asynchronously writes a length-prefixed response to the given writer.
    async fn write_response<T>(
        &mut self,
        _: &MyProtocol,
        io: &mut T,
        MyResponse(data): Self::Response, // Destructures the MyResponse to access the inner Vec<u8>.
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        let len = data.len() as u32;

        // Checks if the message length exceeds the maximum allowed size.
        if len as usize > MAX_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                CodecError::TooLarge(len as usize),
            ));
        }

        // Writes the 4-byte length prefix.
        io.write_all(&len.to_be_bytes()).await?;
        // Writes the actual data.
        io.write_all(&data).await?;
        // Ensures all buffered data is written to the underlying transport.
        io.flush().await?;
        Ok(())
    }
}

// Defines potential errors that can occur during Swarm building.
#[derive(Debug, Error)]
pub enum P2pError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Swarm build error: {0}")]
    SwarmBuild(String),
}

// Asynchronously builds a libp2p Swarm configured with the custom protocol and codec.
pub async fn build_swarm(
    keypair: identity::Keypair,
) -> Result<libp2p::Swarm<Behaviour<MyCodec>>, P2pError> {
    let protocol = MyProtocol();
    // Configures the request-response behaviour with a timeout.
    let rr_config = Config::default().with_request_timeout(Duration::from_secs(10));
    // Specifies the supported protocols for the request-response behaviour.
    let protocols = iter::once((protocol.clone(), ProtocolSupport::Full));
    // Creates the request-response behaviour with the custom codec and protocols.
    let request_response = Behaviour::with_codec(MyCodec, protocols, rr_config);

    // Builds the libp2p Swarm.
    let swarm = SwarmBuilder::with_existing_identity(keypair)
        .with_tokio() // Uses the Tokio runtime for async operations.
        .with_tcp(
            tcp::Config::default(), // Default TCP transport configuration.
            noise::Config::new,     // Configures the Noise protocol for secure connections.
            yamux::Config::default,   // Configures the Yamux multiplexing protocol.
        ).map_err(|e| P2pError::SwarmBuild(e.to_string()))?
        .with_behaviour(|_| Ok(request_response)) // Adds the request-response behaviour to the Swarm.
        .map_err(|e| P2pError::SwarmBuild(e.to_string()))?
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(60))) // Sets a timeout for idle connections.
        .build();

    println!("Local peer ID: {}", swarm.local_peer_id());
    Ok(swarm)
}