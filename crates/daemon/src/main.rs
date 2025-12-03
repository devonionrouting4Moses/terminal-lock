mod daemon;
mod session_store;
mod handlers;
mod lock_monitor;

use daemon::Daemon;
use std::io::{Read, Write};
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::Duration;
use terminal_lock_ipc::{socket_path, DaemonRequest, DaemonResponse};
use tracing::{info, warn, error, debug};

#[cfg(unix)]
use std::os::unix::net::{UnixListener, UnixStream};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    // Get database path
    let home = dirs::home_dir().ok_or("Cannot find home directory")?;
    let lock_dir = home.join(".terminal_lock");
    std::fs::create_dir_all(&lock_dir)?;
    
    let db_path = lock_dir.join("sessions.db");
    info!("📁 Database: {}", db_path.display());
    
    // Create daemon
    let daemon = Arc::new(Daemon::new(db_path)?);
    info!("✅ Daemon initialized");

    // Start periodic cleanup task
    let daemon_cleanup = Arc::clone(&daemon);
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(60));
            if let Err(e) = daemon_cleanup.cleanup_expired() {
                tracing::warn!("Cleanup error: {}", e);
            }
        }
    });

    // Start lock monitor (detects system lock/unlock events)
    let daemon_lock = Arc::clone(&daemon);
    thread::spawn(move || {
        if let Err(e) = lock_monitor::start(daemon_lock) {
            tracing::warn!("Lock monitor error: {}", e);
        }
    });
    
    // Start listening
    #[cfg(unix)]
    {
        start_unix_server(daemon)?;
    }
    
    #[cfg(not(unix))]
    {
        eprintln!("Daemon not yet implemented for this platform");
    }
    
    Ok(())
}

#[cfg(unix)]
fn start_unix_server(daemon: Arc<Daemon>) -> Result<(), Box<dyn std::error::Error>> {
    let socket_path = socket_path();
    
    // Ensure socket directory exists
    if let Some(parent) = socket_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    // Remove old socket if it exists
    if socket_path.exists() {
        std::fs::remove_file(&socket_path)?;
    }
    
    let listener = UnixListener::bind(&socket_path)?;
    eprintln!("✅ Daemon listening on {:?}", socket_path);
    tracing::info!("🔌 IPC listening on: {}", socket_path.display());
    
    // Set socket permissions
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&socket_path, std::fs::Permissions::from_mode(0o600))?;
    }

    // Create bounded thread pool using channels
    // 4 worker threads is enough for IPC requests
    const NUM_WORKERS: usize = 4;
    let (tx, rx) = mpsc::channel::<UnixStream>();
    let rx = Arc::new(std::sync::Mutex::new(rx));
    
    // Spawn worker threads
    for i in 0..NUM_WORKERS {
        let daemon_clone = Arc::clone(&daemon);
        let rx_clone = Arc::clone(&rx);
        
        thread::spawn(move || {
            tracing::debug!("Worker {} started", i);
            
            loop {
                // Wait for incoming connection
                let stream = match rx_clone.lock().unwrap().recv() {
                    Ok(stream) => stream,
                    Err(_) => {
                        tracing::info!("Worker {} shutting down", i);
                        break;
                    }
                };
                
                // Handle client
                if let Err(e) = handle_client(stream, Arc::clone(&daemon_clone)) {
                    tracing::warn!("Worker {} error: {}", i, e);
                }
            }
        });
    }
    
    // Accept connections in a loop and send to workers
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // Set timeouts on the socket
                stream.set_read_timeout(Some(Duration::from_secs(30)))?;
                stream.set_write_timeout(Some(Duration::from_secs(30)))?;
                
                // Send to worker pool
                if tx.send(stream).is_err() {
                    tracing::error!("Failed to send stream to worker pool");
                    break;
                }
            }
            Err(e) => {
                tracing::error!("Error accepting connection: {}", e);
                // Don't exit daemon on accept errors
                thread::sleep(Duration::from_millis(100));
            }
        }
    }
    
    Ok(())
}

#[cfg(unix)]
fn handle_client(mut stream: UnixStream, daemon: Arc<Daemon>) -> Result<(), Box<dyn std::error::Error>> {
    stream.set_read_timeout(Some(std::time::Duration::from_secs(30)))?;
    stream.set_write_timeout(Some(std::time::Duration::from_secs(30)))?;
    stream.set_nonblocking(true)?;
    
    // Read message length
    let mut len_bytes = [0u8; 4];
    stream.read_exact(&mut len_bytes)?;
    let len = u32::from_le_bytes(len_bytes) as usize;

    // Sanity check (prevent DoS)
    if len > 1_000_000 {
        return Err("Message too large".into());
    }
    
    // Read message
    let mut msg_bytes = vec![0u8; len];
    stream.read_exact(&mut msg_bytes)?;
    
    // Deserialize message
    let msg: DaemonRequest = serde_json::from_slice(&msg_bytes)?;
    
    // Handle message
    let response = daemon.handle_message(msg).unwrap_or_else(|e| {
        DaemonResponse::Error {
            message: e.to_string(),
        }
    });
    
    // Serialize response
    let response_json = serde_json::to_string(&response)?;
    let response_bytes = response_json.as_bytes();
    let response_len = response_bytes.len() as u32;
    
    // Send response
    stream.write_all(&response_len.to_le_bytes())?;
    stream.write_all(&response_bytes)?;
    stream.flush()?;
    
    Ok(())
}
