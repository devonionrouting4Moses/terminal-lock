use crate::session_store::SessionStore;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct DaemonState {
    pub lock_generation: u64,
}

pub struct Daemon {
    pub session_store: Arc<Mutex<SessionStore>>,
    pub state: Arc<Mutex<DaemonState>>,
}

impl Daemon {
    pub fn new(db_path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let store = SessionStore::new(db_path)?;
        
        Ok(Daemon {
            session_store: Arc::new(Mutex::new(store)),
            state: Arc::new(Mutex::new(DaemonState {
                lock_generation: 0,
            })),
        })
    }

    pub fn cleanup_expired(&self) -> Result<(), Box<dyn std::error::Error>> {
        let store = self.session_store.lock().unwrap();
        store.cleanup_expired_sessions()?;
        Ok(())
    }
}
