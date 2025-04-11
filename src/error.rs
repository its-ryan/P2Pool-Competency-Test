use std::error::Error as StdError;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Noise(libp2p::noise::Error),
    Transport(libp2p::TransportError<std::io::Error>),
    Multiaddr(libp2p::multiaddr::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "IO error: {}", e),
            Error::Noise(e) => write!(f, "Noise error: {}", e),
            Error::Transport(e) => write!(f, "Transport error: {}", e),
            Error::Multiaddr(e) => write!(f, "Multiaddr error: {}", e),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            Error::Noise(e) => Some(e),
            Error::Transport(e) => Some(e),
            Error::Multiaddr(e) => Some(e),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<libp2p::noise::Error> for Error {
    fn from(e: libp2p::noise::Error) -> Self {
        Error::Noise(e)
    }
}

impl From<libp2p::TransportError<std::io::Error>> for Error {
    fn from(e: libp2p::TransportError<std::io::Error>) -> Self {
        Error::Transport(e)
    }
}

impl From<libp2p::multiaddr::Error> for Error {
    fn from(e: libp2p::multiaddr::Error) -> Self {
        Error::Multiaddr(e)
    }
}