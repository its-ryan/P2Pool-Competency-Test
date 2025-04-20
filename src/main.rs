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
    // If you provide a multiaddr as first arg, act as client; otherwise server.
    let mut args = env::args().skip(1);
    let target = args.next();

    // Build swarm + Tower service
    let key = identity::Keypair::generate_ed25519();
    let mut swarm = build_swarm(key).await?;
    let mut service = TowerService;

    if let Some(addr_str) = target {
        // Client mode: dial and ping
        let remote: Multiaddr = addr_str.parse()?;
        println!("Client: dialing {}...", remote);
        swarm.dial(remote.clone())?;

        // Wait for a connection and then send ping once
        loop {
            if let SwarmEvent::ConnectionEstablished { peer_id, .. } = swarm.select_next_some().await {
                println!("Client: connected to {}", peer_id);
                let _req_id = swarm.behaviour_mut()
                    .send_request(&peer_id, MyRequest(b"ping".to_vec()));

                // Await the pong
                while let SwarmEvent::Behaviour(ReqRespEvent::Message { message, .. }) = swarm.select_next_some().await {
                    if let ReqRespMessage::Response { request_id, response } = message {
                        let MyResponse(data) = response;
                        println!("Client: got pong '{}' for req {:?}", String::from_utf8_lossy(&data), request_id);
                        return Ok(());
                    }
                }
            }
        }
    } else {
        // Server mode: listen and reply
        let listen: Multiaddr = "/ip4/0.0.0.0/tcp/0".parse()?;
        swarm.listen_on(listen)?;

        // Wait for the OS to assign a port, then print it
        if let SwarmEvent::NewListenAddr { address, .. } = swarm.select_next_some().await {
            let pid = swarm.local_peer_id();
            println!("Server listening on {}", address);
            println!("Full multiaddr: {}/p2p/{}", address, pid);
        }

        println!("Server: waiting for incoming ping...");

        // Wait for request and echo
        loop {
            if let SwarmEvent::Behaviour(ReqRespEvent::Message { message, .. }) = swarm.select_next_some().await {
                if let ReqRespMessage::Request { request, channel, .. } = message {
                    let MyRequest(data) = request;
                    println!("Server: received ping '{}'", String::from_utf8_lossy(&data));

                    // Use TowerService to echo back
                    futures::future::poll_fn(|cx| service.poll_ready(cx)).await?;
                    let resp_data = service.call(data).await?;
                    if let Err(e) = swarm.behaviour_mut().send_response(channel, MyResponse(resp_data)) {
                        eprintln!("Server: failed to send pong: {:?}", e);
                    }
                    println!("Server: sent pong");
                    return Ok(());
                }
            }
        }
    }
}
