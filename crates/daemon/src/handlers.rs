/// Message handlers for daemon IPC requests
use terminal_lock_ipc::{DaemonRequest, DaemonResponse, InvalidReason, SessionStatus, SessionState};
use crate::daemon::Daemon;

impl Daemon {
    /// Handle incoming IPC message and return response
    pub fn handle_message(&self, request: DaemonRequest) -> Result<DaemonResponse, String> {
        match request {
            DaemonRequest::Ping => self.handle_ping(),
            
            DaemonRequest::CreateSession { shell_pid, password_hash } => {
                self.handle_create_session(shell_pid, &password_hash)
            }
            
            DaemonRequest::ValidateSession { shell_pid } => {
                self.handle_validate_session(shell_pid)
            }
            
            DaemonRequest::ExtendSession { shell_pid } => {
                self.handle_extend_session(shell_pid)
            }
            
            DaemonRequest::InvalidateSession { shell_pid } => {
                self.handle_invalidate_session(shell_pid)
            }
            
            DaemonRequest::GetSessionStatus { shell_pid } => {
                self.handle_get_session_status(shell_pid)
            }
            
            DaemonRequest::ListSessions => {
                self.handle_list_sessions()
            }
            
            DaemonRequest::PauseAllSessions => {
                self.handle_pause_all_sessions()
            }
            
            DaemonRequest::ResumeAllSessions => {
                self.handle_resume_all_sessions()
            }
            
            DaemonRequest::IncrementLockGeneration => {
                self.handle_increment_lock_generation()
            }
            
            DaemonRequest::LockAllSessions => {
                self.handle_lock_all_sessions()
            }
            
            DaemonRequest::SystemLocked => {
                self.handle_system_locked()
            }
            
            DaemonRequest::SystemUnlocked => {
                self.handle_system_unlocked()
            }
            
            DaemonRequest::Shutdown => {
                self.handle_shutdown()
            }
        }
    }

    fn handle_ping(&self) -> Result<DaemonResponse, String> {
        Ok(DaemonResponse::Pong)
    }

    fn handle_create_session(
        &self,
        shell_pid: u32,
        password_hash: &str,
    ) -> Result<DaemonResponse, String> {
        let store = self.session_store.lock().map_err(|e| e.to_string())?;
        
        match store.create_session(shell_pid, password_hash) {
            Ok((session_id, expires_at)) => {
                Ok(DaemonResponse::SessionCreated {
                    session_id,
                    expires_at,
                })
            }
            Err(e) => {
                Err(format!("Failed to create session: {}", e))
            }
        }
    }

    fn handle_validate_session(&self, shell_pid: u32) -> Result<DaemonResponse, String> {
        let store = self.session_store.lock().map_err(|e| e.to_string())?;
        
        match store.validate_session(shell_pid) {
            Ok((session_id, time_remaining_secs, lock_generation)) => {
                Ok(DaemonResponse::SessionValid {
                    session_id,
                    time_remaining_secs,
                    lock_generation,
                })
            }
            Err(reason) => {
                Ok(DaemonResponse::SessionInvalid { reason })
            }
        }
    }

    fn handle_extend_session(&self, shell_pid: u32) -> Result<DaemonResponse, String> {
        let store = self.session_store.lock().map_err(|e| e.to_string())?;
        
        match store.extend_session(shell_pid) {
            Ok(session_id) => {
                Ok(DaemonResponse::SessionExtended { session_id })
            }
            Err(_) => {
                Ok(DaemonResponse::SessionInvalid {
                    reason: InvalidReason::NotFound,
                })
            }
        }
    }

    fn handle_invalidate_session(&self, shell_pid: u32) -> Result<DaemonResponse, String> {
        let store = self.session_store.lock().map_err(|e| e.to_string())?;
        
        store.invalidate_session(shell_pid).ok();
        
        Ok(DaemonResponse::SessionInvalidated { shell_pid })
    }

    fn handle_get_session_status(
        &self,
        shell_pid: Option<u32>,
    ) -> Result<DaemonResponse, String> {
        let store = self.session_store.lock().map_err(|e| e.to_string())?;
        
        let sessions = match shell_pid {
            Some(pid) => {
                // Get specific session
                store.get_session(pid)
                    .ok()
                    .map(|session| {
                        vec![SessionStatus {
                            id: session.id,
                            shell_pid: session.terminal_pid,
                            created_at: session.created_at,
                            expires_at: session.expires_at,
                            last_activity: session.last_activity,
                            state: session.state.into(),
                            lock_generation: session.lock_generation,
                        }]
                    })
                    .unwrap_or_default()
            }
            None => {
                // Get all sessions
                store.list_sessions()
                    .unwrap_or_default()
                    .into_iter()
                    .map(|session| {
                        SessionStatus {
                            id: session.id,
                            shell_pid: session.terminal_pid,
                            created_at: session.created_at,
                            expires_at: session.expires_at,
                            last_activity: session.last_activity,
                            state: session.state.into(),
                            lock_generation: session.lock_generation,
                        }
                    })
                    .collect()
            }
        };
        
        Ok(DaemonResponse::SessionStatus { sessions })
    }

    fn handle_list_sessions(&self) -> Result<DaemonResponse, String> {
        let store = self.session_store.lock().map_err(|e| e.to_string())?;
        
        let sessions = store.list_sessions()
            .unwrap_or_default()
            .into_iter()
            .map(|session| {
                SessionStatus {
                    id: session.id,
                    shell_pid: session.terminal_pid,
                    created_at: session.created_at,
                    expires_at: session.expires_at,
                    last_activity: session.last_activity,
                    state: session.state.into(),
                    lock_generation: session.lock_generation,
                }
            })
            .collect();
        
        Ok(DaemonResponse::SessionStatus { sessions })
    }

    fn handle_pause_all_sessions(&self) -> Result<DaemonResponse, String> {
        let store = self.session_store.lock().map_err(|e| e.to_string())?;
        
        store.pause_all_sessions().map_err(|e| e.to_string())?;
        
        Ok(DaemonResponse::AllSessionsPaused)
    }

    fn handle_resume_all_sessions(&self) -> Result<DaemonResponse, String> {
        let store = self.session_store.lock().map_err(|e| e.to_string())?;
        
        store.resume_all_sessions().map_err(|e| e.to_string())?;
        
        Ok(DaemonResponse::AllSessionsResumed)
    }

    fn handle_increment_lock_generation(&self) -> Result<DaemonResponse, String> {
        let mut state = self.state.lock().map_err(|e| e.to_string())?;
        state.lock_generation += 1;
        let new_generation = state.lock_generation;
        
        Ok(DaemonResponse::LockGenerationIncremented {
            new_generation,
        })
    }

    fn handle_lock_all_sessions(&self) -> Result<DaemonResponse, String> {
        let store = self.session_store.lock().map_err(|e| e.to_string())?;
        
        store.lock_all_sessions().map_err(|e| e.to_string())?;
        
        Ok(DaemonResponse::AllSessionsLocked)
    }

    fn handle_system_locked(&self) -> Result<DaemonResponse, String> {
        let store = self.session_store.lock().map_err(|e| e.to_string())?;
        
        store.pause_all_sessions().map_err(|e| e.to_string())?;
        
        Ok(DaemonResponse::SystemLockAcknowledged)
    }

    fn handle_system_unlocked(&self) -> Result<DaemonResponse, String> {
        let store = self.session_store.lock().map_err(|e| e.to_string())?;
        
        store.resume_all_sessions().map_err(|e| e.to_string())?;
        
        Ok(DaemonResponse::SystemUnlockAcknowledged)
    }

    fn handle_shutdown(&self) -> Result<DaemonResponse, String> {
        // Graceful shutdown
        std::process::exit(0);
    }
}
