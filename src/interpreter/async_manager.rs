use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::task::JoinHandle;
use crate::interpreter::error::{RuntimeError, RuntimeResult};
use log;

pub struct AsyncManager {
    pub threads: Arc<Mutex<HashMap<String, JoinHandle<RuntimeResult<()>>>>>,
    thread_timeouts: Arc<Mutex<HashMap<String, Instant>>>,
    thread_timeout: Duration,
}

impl AsyncManager {
    pub fn new() -> Self {
        Self {
            threads: Arc::new(Mutex::new(HashMap::new())),
            thread_timeouts: Arc::new(Mutex::new(HashMap::new())),
            thread_timeout: Duration::from_secs(30),
        }
    }

    pub fn register_thread(&self, name: Option<String>, handle: JoinHandle<RuntimeResult<()>>) -> RuntimeResult<()> {
        if let Some(name) = name {
            let mut threads = self.threads.lock().unwrap();
            let mut timeouts = self.thread_timeouts.lock().unwrap();
            
            if threads.contains_key(&name) {
                log::warn!("Thread '{}' already exists, replacing old thread", name);
                threads.remove(&name);
                timeouts.remove(&name);
            }
            
            threads.insert(name.clone(), handle);
            timeouts.insert(name, Instant::now());
        } else {
            let mut threads = self.threads.lock().unwrap();
            let mut timeouts = self.thread_timeouts.lock().unwrap();
            let name = format!("anonymous_{}", threads.len());
            threads.insert(name.clone(), handle);
            timeouts.insert(name, Instant::now());
        }
        Ok(())
    }

    pub fn cleanup_stale_threads(&self) -> RuntimeResult<()> {
        let mut threads = self.threads.lock().unwrap();
        let mut timeouts = self.thread_timeouts.lock().unwrap();
        let now = Instant::now();
        let mut to_remove = Vec::new();

        for (name, start_time) in timeouts.iter() {
            if now.duration_since(*start_time) > self.thread_timeout {
                to_remove.push(name.clone());
            }
        }

        for name in to_remove {
            if let Some(handle) = threads.remove(&name) {
                handle.abort();
            }
            timeouts.remove(&name);
        }

        Ok(())
    }

    pub fn unregister_thread(&self, name: &str) -> RuntimeResult<Option<JoinHandle<RuntimeResult<()>>>> {
        let mut threads = self.threads.lock().unwrap();
        let mut timeouts = self.thread_timeouts.lock().unwrap();
        
        if !threads.contains_key(name) {
            return Err(RuntimeError::AsyncError(
                format!("Thread '{}' not found in the cosmic void", name)
            ).into());
        }
        
        timeouts.remove(name);
        Ok(threads.remove(name))
    }

    pub fn has_thread(&self, name: &str) -> bool {
        self.threads.lock().unwrap().contains_key(name)
    }

    pub fn list_threads(&self) -> Vec<String> {
        self.threads.lock().unwrap().keys().cloned().collect()
    }

    pub fn is_thread_running(&self, name: &str) -> RuntimeResult<bool> {
        let threads = self.threads.lock().unwrap();
        if let Some(handle) = threads.get(name) {
            Ok(!handle.is_finished())
        } else {
            Err(RuntimeError::AsyncError(
                format!("Thread '{}' not found in the cosmic void", name)
            ).into())
        }
    }

    pub fn get_thread_status(&self, name: &str) -> RuntimeResult<String> {
        let threads = self.threads.lock().unwrap();
        if let Some(handle) = threads.get(name) {
            if handle.is_finished() {
                Ok("completed".to_string())
            } else {
                Ok("running".to_string())
            }
        } else {
            Err(RuntimeError::AsyncError(
                format!("Thread '{}' not found in the cosmic void", name)
            ).into())
        }
    }
} 