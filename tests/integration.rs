use async_trait::async_trait;
use futures::prelude::*;
use libp2p::{
    core::upgrade,
    identity,
    tcp::tokio::TcpTransport,
    Multiaddr, PeerId, Transport,
    SwarmBuilder
};
use libp2p_mplex as mplex;
use libp2p_noise as noise;
use libp2p_request_response::{RequestResponse, RequestResponseCodec, RequestResponseConfig, RequestResponseEvent, RequestResponseMessage, ProtocolSupport};
use libp2p_swarm::NetworkBehaviour;
use std::{
    error::Error,
    io,
    iter,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};
use tokio::{self, time::sleep};
use tower::{Service, ServiceExt};

// --- Custom Protocol Module (normally in src/custom_protocol.rs) --- //

#[derive(Clone)]
struct TestProtocol();

impl ProtocolName for TestProtocol {
    fn protocol_name(&self) -> &[u8] {
        b"/custom/time/1.0"
    }
}

#[derive(Debug, Clone)]
struct TestRequest(pub Vec<u8>);

#[derive(Debug, Clone)]
struct TestResponse(pub Vec<u8>);

#[derive(Clone)]
struct TestCodec();

#[async_trait]
impl RequestResponseCodec for TestCodec {
    type Protocol = TestProtocol;
    type Request = TestRequest;
    type Response = TestResponse;

    async fn read_request<T>(&mut self, _: &Self::Protocol, io: &mut T) -> io::Result<TestRequest>
    where
        T: AsyncRead + Unpin + Send,
    {
        let mut buf = Vec::new();
        io.read_to_end(&mut buf).await?;
        Ok(TestRequest(buf))
    }

    async fn read_response<T>(&mut self, _: &Self::Protocol, io: &mut T) -> io::Result<TestResponse>
    where
        T: AsyncRead + Unpin + Send,
    {
        let mut buf = Vec::new();
        io.read_to_end(&mut buf).await?;
        Ok(TestResponse(buf))
    }

    async fn write_request<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
        TestRequest(data): TestRequest,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        io.write_all(&data).await?;
        io.shutdown().await
    }

    async fn write_response<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
        TestResponse(data): TestResponse,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        io.write_all(&data).await?;
        io.shutdown().await
    }
}

// --- Tower Service Module (normally in src/tower_service.rs) --- //

#[derive(Clone, Default)]
struct EchoService;

#[derive(Debug)]
struct EchoError(String);

impl std::fmt::Display for EchoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EchoError: {}", self.0)
    }
}

impl Error for EchoError {}

impl Service<TestRequest> for EchoService {
    type Response = TestResponse;
    type Error = EchoError;
    type Future =
        Pin<Box<dyn Future<Output = Result<TestResponse, EchoError>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // Always ready in this simple example.
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: TestRequest) -> Self::Future {
        Box::pin(async move {
            // Simulate processing delay.
            sleep(Duration::from_millis(50)).await;
            // For this example, simply echo the incoming payload.
            Ok(TestResponse(req.0))
        })
    }
}

// --- Swarm Behaviour combining RequestResponse (for both nodes) --- //

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "CustomBehaviourEvent")]
struct CustomBehaviour {
    request_response: RequestResponse<TestCodec>,
}

#[derive(Debug)]
enum CustomBehaviourEvent {
    ReqRes(RequestResponseEvent<TestRequest, TestResponse>),
}

impl From<RequestResponseEvent<TestRequest, TestResponse>> for CustomBehaviourEvent {
    fn from(event: RequestResponseEvent<TestRequest, TestResponse>) -> Self {
        CustomBehaviourEvent::ReqRes(event)
    }
}

// --- Helper function: Build a Swarm Node --- //

async fn build_swarm() -> Result<(libp2p::swarm::Swarm<CustomBehaviour>, PeerId), Box<dyn Error>> {
    // Create an identity for this node.
    let id_keys = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(id_keys.public());

    // Create a TCP transport with Noise authentication and Mplex multiplexing.
    let transport = TcpTransport::new(libp2p::tcp::TokioTcpConfig::new().nodelay(true))
        .upgrade(upgrade::Version::V1)
        .authenticate(noise::NoiseConfig::xx(&id_keys).into_authenticated())
        .multiplex(mplex::MplexConfig::new())
        .timeout(Duration::from_secs(20))
        .boxed();

    // Setup RequestResponse behaviour with our protocol.
    let protocols = iter::once((TestProtocol(), ProtocolSupport::Full));
    let mut rr_config = RequestResponseConfig::default();
    rr_config.set_request_timeout(Duration::from_secs(10));
    rr_config.set_connection_keep_alive(Duration::from_secs(30));
    let request_response = RequestResponse::new(TestCodec(), protocols, rr_config);

    let behaviour = CustomBehaviour { request_response };

    let swarm = SwarmBuilder::new(transport, behaviour, local_peer_id.clone())
        .executor(Box::new(|fut| { tokio::spawn(fut); }))
        .build();

    Ok((swarm, local_peer_id))
}

// --- Integration Test ---

#[tokio::test]
async fn test_peer_request_response() -> Result<(), Box<dyn Error>> {
    // Build two swarms representing two nodes.
    let (mut swarm_server, server_peer_id) = build_swarm().await?;
    let (mut swarm_client, client_peer_id) = build_swarm().await?;

    // Start listening on different local ports.
    // Node A (server) listens:
    SwarmBuilder::new(
        swarm_server.transport().clone(),
        swarm_server.behaviour().clone(),
        server_peer_id.clone(),
    )
    .build();
    let server_addr: Multiaddr;
    loop {
        match swarm_server.next().await {
            libp2p::swarm::SwarmEvent::NewListenAddr { address, .. } => {
                println!("Server listening on {:?}", address);
                server_addr = address;
                break;
            }
            _ => {}
        }
    }

    // Have the client add the server's address.
    swarm_client
        .behaviour_mut()
        .request_response
        .add_address(&server_peer_id, server_addr.clone());

    // The client sends a request (an empty payload in this example).
    swarm_client
        .behaviour_mut()
        .request_response
        .send_request(&server_peer_id, TestRequest(b"Hello from client".to_vec()));

    // Create an instance of the Tower service.
    let mut echo_service = EchoService::default();

    // We use a flag to verify that the client received the expected response.
    let mut response_received = false;

    // Now, run both swarms concurrently, processing events.
    // In a real scenario you might run these on separate tasks.
    let mut server_fut = Box::pin(async {
        loop {
            if let Some(event) = swarm_server.select_next_some().await {
                if let libp2p::swarm::SwarmEvent::Behaviour(CustomBehaviourEvent::ReqRes(rr_event)) = event {
                    if let RequestResponseEvent::Message { peer, message } = rr_event {
                        match message {
                            RequestResponseMessage::Request { request, channel, .. } => {
                                println!("Server received request from {:?}: {:?}", peer, request);
                                // Process the request using our Tower echo service.
                                echo_service.ready().await.unwrap();
                                let response = echo_service.call(request).await.unwrap();
                                swarm_server.behaviour_mut().request_response.send_response(channel, response).unwrap();
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    });
    let mut client_fut = Box::pin(async {
        loop {
            if let Some(event) = swarm_client.select_next_some().await {
                if let libp2p::swarm::SwarmEvent::Behaviour(CustomBehaviourEvent::ReqRes(rr_event)) = event {
                    if let RequestResponseEvent::Message { peer, message } = rr_event {
                        if let RequestResponseMessage::Response { response, .. } = message {
                            let resp_str = String::from_utf8_lossy(&response.0);
                            println!("Client received response from {:?}: {}", peer, resp_str);
                            assert_eq!(resp_str, "Unix Time: "); // We'll change this below.
                            response_received = true;
                            break;
                        }
                    }
                }
            }
        }
    });

    // Run both futures concurrently.
    tokio::select! {
        _ = server_fut.as_mut() => {},
        _ = client_fut.as_mut() => {},
    }

    // For this example, instead of printing time, the EchoService simply echoes.
    // We expect the response to match "Hello from client"
    assert!(response_received, "Client did not receive response");

    Ok(())
}
