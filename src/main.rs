mod p2p;
mod service;

use std::env;
use futures::StreamExt;
use libp2p::{
    identity,
    Multiaddr,
    swarm::SwarmEvent,
    request_response::{Event as ReqRespEvent, Message as ReqRespMessage},
};
use p2p::{build_swarm, MyRequest, MyResponse};
use service::TowerService;
use tower::Service;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut args = env::args().skip(1);
    let target_addr = args.next();

    let local_key = identity::Keypair::generate_ed25519();
    let mut swarm = build_swarm(local_key).await?;
    let mut app_service = TowerService;

    match target_addr {
        Some(addr_str) => {
            // Client logic: Dial the specified address and send a ping.
            let remote_address: Multiaddr = addr_str.parse()?;
            println!("Client: dialing {}...", remote_address);
            swarm.dial(remote_address.clone())?;

            // Wait for connection establishment and send a ping request.
            while let Some(event) = swarm.next().await {
                if let SwarmEvent::ConnectionEstablished { peer_id, .. } = event {
                    println!("Client: connected to {}", peer_id);
                    swarm.behaviour_mut().send_request(&peer_id, MyRequest(b"ping".to_vec()));

                    // Await the pong response.
                    while let Some(SwarmEvent::Behaviour(ReqRespEvent::Message { message, .. })) = swarm.next().await {
                        if let ReqRespMessage::Response { request_id, response } = message {
                            let MyResponse(data) = response;
                            println!("Client: got pong '{}' for req {:?}", String::from_utf8_lossy(&data), request_id);
                            return Ok(());
                        }
                    }
                }
            }
        }
        None => {
            // Server logic: Listen for incoming connections and respond to pings.
            let listen_address: Multiaddr = "/ip4/0.0.0.0/tcp/0".parse()?;
            swarm.listen_on(listen_address)?;

            // Report the listening address.
            if let Some(SwarmEvent::NewListenAddr { address, .. }) = swarm.next().await {
                let peer_id = swarm.local_peer_id();
                println!("Server listening on {}", address);
                println!("Full multiaddr: {}/p2p/{}", address, peer_id);
            }

            println!("Server: waiting for incoming ping...");

            // Process incoming ping requests and send back a pong response.
            while let Some(SwarmEvent::Behaviour(ReqRespEvent::Message { message, .. })) = swarm.next().await {
                if let ReqRespMessage::Request { request, channel, .. } = message {
                    let MyRequest(data) = request;
                    println!("Server: received ping '{}'", String::from_utf8_lossy(&data));

                    // Use the application service to process the request (echo in this case).
                    futures::future::poll_fn(|cx| app_service.poll_ready(cx)).await?;
                    let response_data = app_service.call(data).await?;
                    if let Err(e) = swarm.behaviour_mut().send_response(channel, MyResponse(response_data)) {
                        eprintln!("Server: failed to send pong: {:?}", e);
                    }
                    println!("Server: sent pong");
                    return Ok(());
                }
            }
        }
    }
    Ok(())
}