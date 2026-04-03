# BitTorrent Lite

BitTorrent Lite is an educational P2P file-sharing application built with Rust. It demonstrates the core principles of peer-to-peer networking, including decentralized peer discovery, chunk-based file transfer, and asynchronous I/O.

## 🚀 Features

- **P2P Networking**: Built using [libp2p](https://libp2p.io/), a modular network stack for peer-to-peer applications.
- **Automatic Peer Discovery**: Uses **mDNS** to automatically find and connect to other nodes on the local network.
- **Chunk-based Transfer**: Files are transferred in **256KB chunks**, allowing for efficient and reliable downloads.
- **Asynchronous Architecture**: Powered by [tokio](https://tokio.rs/) for non-blocking networking and file system operations.
- **Simple CLI**: Easy-to-use command-line interface built with [clap](https://clap.rs/).
- **Custom Protocol**: Implements a Request-Response protocol using CBOR for structured message exchange.

## 🛠️ Tech Stack

- **Language**: Rust (Edition 2024)
- **Async Runtime**: `tokio`
- **Networking**: `libp2p` (TCP, Noise, Yamux, mDNS, Request-Response)
- **Serialization**: `serde`, `serde_json`, `cbor4ii`
- **CLI**: `clap`

## 📁 Project Structure

- `src/main.rs`: The entry point and network event loop handling peer discovery and requests.
- `src/file_system.rs`: Logic for managing the shared directory, scanning files, and reading/writing chunks.
- `src/messages.rs`: Definition of the network protocol messages (`FileRequest` and `FileResponse`).
- `shared_files/`: The default directory for hosting and downloading files.

## 🚦 Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable version recommended)

### Installation

1. Clone the repository:
   ```bash
   git clone <repository-url>
   cd bittorrent_lite
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

### Usage

The application provides three main commands:

#### 1. Start a Node
Starts the P2P node, listens for incoming connections, and shares files from the `shared_files` directory.
```bash
cargo run -- start
```

#### 2. List Local Files
Lists all files currently available in your local `shared_files` directory.
```bash
cargo run -- list
```

#### 3. Request a File
Requests a specific file from a discovered peer on the network. The file will be downloaded in chunks and saved with a `downloaded_` prefix.
```bash
cargo run -- request <file_name>
```

## ⚙️ How It Works

1. **Initialization**: On startup, the node ensures the `shared_files` directory exists.
2. **Discovery**: The node uses mDNS to broadcast its presence and listen for other peers on the local network.
3. **Connection**: When a peer is discovered, the node automatically dials their address.
4. **Requests**:
   - `ListFiles`: Queries a peer for the names of files they are sharing.
   - `Chunk`: Requests a specific 256KB segment of a file.
5. **Transfer**: If the peer has the requested chunk, they send the raw bytes back. The requester appends these bytes to a local file until the entire file is assembled.

## 📝 License

This project is created for educational purposes.
