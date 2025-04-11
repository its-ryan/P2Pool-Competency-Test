use crate::custom_protocol::{TimeCodec, TimeProtocol};
use crate::error::Error;
use libp2p::{identity, noise, tcp, yamux};
use libp2p::request_response::{Behaviour, Config, ProtocolSupport};
use libp2p::SwarmBuilder;
use std::iter;
use std::time::Duration;

pub async fn build_swarm(
    keypair: identity::Keypair,
) -> Result<libp2p::Swarm<Behaviour<TimeCodec>>, Error> {

    let protocol = TimeProtocol;
    let rr_config = Config::default()
        .with_request_timeout(Duration::from_secs(10));
    let protocols = iter::once((protocol, ProtocolSupport::Full));
    let request_response = Behaviour::new(protocols, rr_config);

    let swarm = SwarmBuilder::with_existing_identity(keypair)
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|_| Ok(request_response)).map_err(|e| Error::from(std::io::Error::new(std::io::ErrorKind::Other, e)))?
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(60)))
        .build();

    Ok(swarm)
}