use serde::{Deserialize, Serialize};

/// Represents the types of requests a node can send to another node
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileRequest {
    /// "What files do you have available in your shared directory?"
    ListFiles,
    
    /// "Send me a specific chunk of a specific file."
    Chunk {
        file_name: String,
        chunk_index: usize,
    },
}

/// Represents the types of responses a node will send back
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileResponse {
    /// "Here is a list of the file names I am currently sharing."
    FileList(Vec<String>),
    
    /// "Here are the raw bytes for the chunk you requested."
    ChunkData {
        file_name: String,
        chunk_index: usize,
        data: Vec<u8>,
    },
    
    /// "I don't have the file or chunk you are looking for."
    NotFound,
}