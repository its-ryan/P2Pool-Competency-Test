use crate::error::Error;
use libp2p::core::transport::{self, Boxed, Transport as TransportUpgrade};
use libp2p::identity;
use libp2p::noise;
use libp2p::tcp;
use libp2p::yamux;
use std::time::Duration;

pub async fn build_transport(
    keypair: &identity::Keypair,
) -> Result<Boxed<(libp2p::PeerId, libp2p::core::muxing::StreamMuxerBox)>, Error> {
    let transport = tcp::tokio::Transport::default()
        .upgrade(transport::upgrade::Version::V1)
        .authenticate(noise::Config::new(keypair)?)
        .multiplex(yamux::Config::default())
        .timeout(Duration::from_secs(20))
        .boxed();
    Ok(transport)
}