use clap::{Parser, Subcommand};
use futures::StreamExt;
use libp2p::{
    mdns,
    request_response::{self, ProtocolSupport},
    swarm::{NetworkBehaviour, SwarmEvent},
    StreamProtocol, SwarmBuilder,
};
use std::error::Error;

mod file_system;
mod messages;

use messages::{FileRequest, FileResponse};

#[derive(Parser)]
#[command(name = "bittorrent-lite")]
#[command(version = "1.0")]
#[command(about = "BitTorrent Lite - Educational P2P File Sharing", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Start,
    List,
    Request { file_name: String },
}

// UPGRADE: Our node now speaks two languages: mDNS and Request-Response!
#[derive(NetworkBehaviour)]
struct MyBehaviour {
    mdns: mdns::tokio::Behaviour,
    req_resp: request_response::cbor::Behaviour<FileRequest, FileResponse>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    file_system::init_shared_dir().await?;
    let cli = Cli::parse();

    match &cli.command {
        Commands::Start => {
            println!("🚀 Starting BitTorrent Lite node...");

            let mut swarm = SwarmBuilder::with_new_identity()
                .with_tokio()
                .with_tcp(
                    Default::default(),
                    libp2p::noise::Config::new,
                    libp2p::yamux::Config::default,
                )?
                .with_behaviour(|key| {
                    // 1. Setup mDNS
                    let mdns = mdns::tokio::Behaviour::new(
                        mdns::Config::default(),
                        key.public().to_peer_id(),
                    )?;
                    
                    // 2. Setup Request-Response using CBOR
                    // We define a custom protocol name for our app
                    let req_resp = request_response::cbor::Behaviour::new(
                        [(StreamProtocol::new("/bittorrent-lite/1.0.0"), ProtocolSupport::Full)],
                        request_response::Config::default(),
                    );

                    Ok(MyBehaviour { mdns, req_resp })
                })?
                .with_swarm_config(|c| c.with_idle_connection_timeout(std::time::Duration::from_secs(60)))
                .build();

            swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;
            println!("📡 Scanning for peers and waiting for requests...");

            // The Network Event Loop
            loop {
                tokio::select! {
                    event = swarm.select_next_some() => match event {
                        SwarmEvent::NewListenAddr { address, .. } => {
                            println!("✅ Listening on: {:?}", address);
                        }
                        SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                            for (peer_id, multiaddr) in list {
                                println!("👀 Discovered peer: {}", peer_id);
                                // Automatically connect to the peer we just found!
                                let _ = swarm.dial(multiaddr); 
                            }
                        }
                        // Handle incoming network requests from other peers!
                        SwarmEvent::Behaviour(MyBehaviourEvent::ReqResp(request_response::Event::Message { peer, message, .. })) => {
                            match message {
                                request_response::Message::Request { request, channel, .. } => {
                                    println!("📥 Received request from {}: {:?}", peer, request);
                                    
                                    // Let's handle the specific request type
                                    match request {
                                        FileRequest::ListFiles => {
                                            // Read our local directory
                                            let files = file_system::list_files().await.unwrap_or_default();
                                            // Send the list back to the peer!
                                            let _ = swarm.behaviour_mut().req_resp.send_response(channel, FileResponse::FileList(files));
                                        }
                                        FileRequest::Chunk { file_name, chunk_index } => {
                                            // Try to read the specific chunk from our hard drive
                                            match file_system::read_chunk(&file_name, chunk_index).await {
                                                Ok(data) => {
                                                    // Success! Send the raw bytes back.
                                                    println!("📤 Sending chunk {} of '{}' to {}", chunk_index, file_name, peer);
                                                    let _ = swarm.behaviour_mut().req_resp.send_response(
                                                        channel, 
                                                        FileResponse::ChunkData { file_name, chunk_index, data }
                                                    );
                                                }
                                                Err(e) => {
                                                    // File not found or read error
                                                    println!("⚠️ Failed to read chunk: {}", e);
                                                    let _ = swarm.behaviour_mut().req_resp.send_response(channel, FileResponse::NotFound);
                                                }
                                            }
                                        }
                                    }
                                }
                                request_response::Message::Response { request_id, response } => {
                                    println!("📤 Received response for request {}: {:?}", request_id, response);
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        Commands::List => {
            let files = file_system::list_files().await?;
            println!("📦 Local files: {:?}", files);
            println!("(Network querying comes next!)");
        }
        
        // --- THIS IS THE UPGRADED BLOCK ---
        Commands::Request { file_name } => {
            println!("📥 Requesting file: '{}'", file_name);
            println!("🚀 Starting temporary downloader node...");

            let mut swarm = SwarmBuilder::with_new_identity()
                .with_tokio()
                .with_tcp(Default::default(), libp2p::noise::Config::new, libp2p::yamux::Config::default)?
                .with_behaviour(|key| {
                    let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), key.public().to_peer_id())?;
                    let req_resp = request_response::cbor::Behaviour::new(
                        [(StreamProtocol::new("/bittorrent-lite/1.0.0"), ProtocolSupport::Full)],
                        request_response::Config::default(),
                    );
                    Ok(MyBehaviour { mdns, req_resp })
                })?
                .with_swarm_config(|c| c.with_idle_connection_timeout(std::time::Duration::from_secs(60)))
                .build();

            swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

            let mut expected_chunk = 0; 
            let mut target_peer: Option<libp2p::PeerId> = None;
            let save_path = format!("{}/downloaded_{}", file_system::SHARED_DIR, file_name);
            
            // Delete any half-downloaded file from previous tests so we start fresh
            let _ = tokio::fs::remove_file(&save_path).await;

            loop {
                tokio::select! {
                    event = swarm.select_next_some() => match event {
                        SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                            for (peer_id, multiaddr) in list {
                                if target_peer.is_none() {
                                    println!("👀 Found peer: {}. Starting download...", peer_id);
                                    let _ = swarm.dial(multiaddr);
                                    target_peer = Some(peer_id);
                                    
                                    swarm.behaviour_mut().req_resp.send_request(
                                        &peer_id,
                                        FileRequest::Chunk {
                                            file_name: file_name.clone(),
                                            chunk_index: expected_chunk,
                                        }
                                    );
                                }
                            }
                        }
                        SwarmEvent::Behaviour(MyBehaviourEvent::ReqResp(request_response::Event::Message { message, .. })) => {
                            if let request_response::Message::Response { response, .. } = message {
                                match response {
                                    FileResponse::ChunkData { file_name, chunk_index, data } => {
                                        println!("✅ Received chunk {} ({} bytes)", chunk_index, data.len());
                                        
                                        // Append the new chunk bytes to the file
                                        use tokio::io::AsyncWriteExt;
                                        let mut file = tokio::fs::OpenOptions::new()
                                            .create(true)
                                            .append(true)
                                            .open(&save_path)
                                            .await.unwrap();
                                        file.write_all(&data).await.unwrap();
                                        
                                        // Ask for the NEXT chunk!
                                        expected_chunk += 1;
                                        if let Some(peer) = target_peer {
                                            swarm.behaviour_mut().req_resp.send_request(
                                                &peer,
                                                FileRequest::Chunk {
                                                    file_name: file_name.clone(),
                                                    chunk_index: expected_chunk,
                                                }
                                            );
                                        }
                                    }
                                    FileResponse::NotFound => {
                                        // If they say NotFound for Chunk 0, the file doesn't exist.
                                        // If they say NotFound for Chunk 3, it means Chunk 2 was the end of the file!
                                        if expected_chunk == 0 {
                                            println!("❌ Peer did not have the file.");
                                        } else {
                                            println!("💾 Success! All chunks downloaded. File assembled at: {}", save_path);
                                        }
                                        return Ok(()); // Exit the program
                                    }
                                    _ => {}
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(())
}