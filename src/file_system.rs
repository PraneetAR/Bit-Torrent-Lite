use std::io::Error;
use std::path::Path;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncSeekExt, SeekFrom};

// Define our constants
pub const SHARED_DIR: &str = "./shared_files";
pub const CHUNK_SIZE: usize = 256 * 1024; // 256 KB chunks, as per your spec

/// Ensures the shared directory exists when the node starts
pub async fn init_shared_dir() -> Result<(), Error> {
    if !Path::new(SHARED_DIR).exists() {
        fs::create_dir_all(SHARED_DIR).await?;
        println!("📁 Created shared directory at: {}", SHARED_DIR);
    }
    Ok(())
}

/// Scans the directory and returns a list of available file names
pub async fn list_files() -> Result<Vec<String>, Error> {
    let mut files = Vec::new();
    // Read the directory asynchronously so we don't block the network
    let mut entries = fs::read_dir(SHARED_DIR).await?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.is_file() {
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                files.push(file_name.to_string());
            }
        }
    }
    Ok(files)
}

/// Reads a specific 256KB chunk of a file
pub async fn read_chunk(file_name: &str, chunk_index: usize) -> Result<Vec<u8>, Error> {
    let path = format!("{}/{}", SHARED_DIR, file_name);
    let mut file = fs::File::open(path).await?;
    
    // Calculate where to start reading based on the chunk index
    let offset = (chunk_index * CHUNK_SIZE) as u64;
    file.seek(SeekFrom::Start(offset)).await?;
    
    // Create a buffer exactly the size of our chunk
    let mut buffer = vec![0; CHUNK_SIZE];
    let bytes_read = file.read(&mut buffer).await?;
    
    // If it's the last chunk, it might be smaller than 256KB, so we shrink the buffer to fit
    buffer.truncate(bytes_read);
    
    Ok(buffer)
}