use crate::error::Error;
use crate::swarm::build_swarm;
use crate::tower_service::TimeService;
use libp2p::identity;
use libp2p::request_response::{Event as RequestResponseEvent, Message as RequestResponseMessage};
use libp2p::swarm::SwarmEvent;

mod custom_protocol;
mod tower_service;
mod transport;
mod swarm;
mod error;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let keypair = identity::Keypair::generate_ed25519();
    let peer_id = keypair.public().to_peer_id();
    println!("Local peer id: {:?}", peer_id);

    let mut swarm = build_swarm(keypair).await?;
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    let mut tower_service = TimeService;
    println!("Swarm is running. Waiting for incoming messages...");

    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("Listening on {:?}", address);
            }
            SwarmEvent::Behaviour(RequestResponseEvent::Message { peer, message }) => match message {
                RequestResponseMessage::Request { request, channel, .. } => {
                    println!("Received request from {:?}: {:?}", peer, request);
                    if let Err(e) = tower_service.ready().await {
                        eprintln!("Tower service not ready: {}", e);
                        continue;
                    }
                    let response = tower_service.call(request).await?;
                    swarm.behaviour_mut().send_response(channel, response)
                        .unwrap_or_else(|err| eprintln!("Failed to send response: {:?}", err));
                }
                RequestResponseMessage::Response { response, .. } => {
                    println!("Received response: {:?}", String::from_utf8_lossy(&response.0));
                }
            },
            SwarmEvent::Behaviour(RequestResponseEvent::OutboundFailure { peer, error, .. }) => {
                eprintln!("Outbound request error to {:?}: {:?}", peer, error);
            }
            SwarmEvent::Behaviour(RequestResponseEvent::InboundFailure { peer, error, .. }) => {
                eprintln!("Inbound request error from {:?}: {:?}", peer, error);
            }
            other => {
                println!("Other event: {:?}", other);
            }
        }
    }
}