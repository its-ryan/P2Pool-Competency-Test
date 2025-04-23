# P2Pool: A Peer-to-Peer Networking Node

This project implements a peer-to-peer (P2P) networking node using the [libp2p](https://libp2p.io/) library in Rust. It supports a custom request-response protocol for communication between nodes, enabling both client and server functionality.

## Features

- **Custom Protocol**: Implements a custom protocol (`/my-custom-protocol/1.0.0`) for exchanging messages between peers.
- **Request-Response Messaging**: Supports sending and receiving requests and responses using a custom codec.
- **Secure Communication**: Uses the Noise protocol for encrypted communication.
- **Multiplexing**: Supports multiple streams over a single connection using Yamux.
- **Client and Server Modes**:
  - **Client**: Dials a remote peer, sends a "ping" request, and waits for a "pong" response.
  - **Server**: Listens for incoming connections, processes "ping" requests, and sends "pong" responses.

## Project Structure

- **`p2p` Module**:
  - Defines the custom protocol, request/response types, and codec.
  - Implements the `build_swarm` function to configure and initialize the libp2p swarm.
- **`service` Module**:
  - Implements the `TowerService`, which processes incoming requests and generates responses.
- **`main.rs`**:
  - Entry point of the application.
  - Handles command-line arguments to determine whether the node runs in client or server mode.
  - Implements the main event loop for processing libp2p swarm events.

## Usage

### Prerequisites

- Rust (latest stable version)
- Tokio runtime (used for asynchronous operations)

### Running the Node

1. **Clone the Repository**:
   ```bash
   git clone https://github.com/your-username/p2pool.git
   cd p2pool/p2p_node
   ```

2. **Build the Project**:
   ```bash
   cargo build --release
   ```

3. **Run as a Server**:
   Start the node in server mode (default if no arguments are provided):
   ```bash
   cargo run --release
   ```
   The server will listen on a random TCP port and print its multiaddress.

4. **Run as a Client**:
   Start the node in client mode by providing the server's multiaddress:
   ```bash
   cargo run --release -- /ip4/127.0.0.1/tcp/12345/p2p/QmPeerId
   ```
   Replace `/ip4/127.0.0.1/tcp/12345/p2p/QmPeerId` with the actual multiaddress of the server.

### Example Workflow

1. Start the server:
   ```bash
   cargo run --release
   ```
   Output:
   ```
   Server listening on /ip4/0.0.0.0/tcp/12345
   Full multiaddr: /ip4/127.0.0.1/tcp/12345/p2p/QmPeerId
   ```

2. Start the client with the server's multiaddress:
   ```bash
   cargo run --release -- /ip4/127.0.0.1/tcp/12345/p2p/QmPeerId
   ```
   Output:
   ```
   Client: dialing /ip4/127.0.0.1/tcp/12345/p2p/QmPeerId...
   Client: connected to QmPeerId
   Client: got pong 'pong' for req RequestId
   ```

3. The server will log:
   ```
   Server: received ping 'ping'
   Server: sent pong
   ```

## Code Overview

### `p2p.rs`

- **Custom Protocol**: Defines the protocol identifier and request/response types.
- **Codec**: Implements serialization and deserialization for requests and responses.
- **Swarm Builder**: Configures the libp2p swarm with TCP transport, Noise encryption, and Yamux multiplexing.

### main.rs

- **Client Mode**:
  - Dials a remote peer and sends a "ping" request.
  - Waits for a "pong" response and prints it.
- **Server Mode**:
  - Listens for incoming connections.
  - Processes "ping" requests and sends "pong" responses.

### `service.rs`

- Implements a simple service using the [Tower](https://github.com/tower-rs/tower) library to process requests and generate responses.

## Dependencies

- [libp2p](https://crates.io/crates/libp2p): Peer-to-peer networking library.
- [tokio](https://crates.io/crates/tokio): Asynchronous runtime.
- [tower](https://crates.io/crates/tower): Abstraction for request-response services.
- [thiserror](https://crates.io/crates/thiserror): Error handling library.

## License

This project is licensed under the MIT License. See the LICENSE file for details.

## Acknowledgments

- [libp2p Documentation](https://docs.libp2p.io/)
- [Rust Async Programming](https://rust-lang.github.io/async-book/)

Feel free to contribute or raise issues if you encounter any problems!
