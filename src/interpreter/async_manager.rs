use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::task::JoinHandle;
use crate::interpreter::error::{RuntimeError, RuntimeResult};

pub struct AsyncManager {
    pub threads: Arc<Mutex<HashMap<String, JoinHandle<RuntimeResult<()>>>>>,
}

impl AsyncManager {
    pub fn new() -> Self {
        Self {
            threads: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn register_thread(&self, name: Option<String>, handle: JoinHandle<RuntimeResult<()>>) -> RuntimeResult<()> {
        if let Some(name) = name {
            let mut threads = self.threads.lock().unwrap();
            if threads.contains_key(&name) {
                return Err(RuntimeError::AsyncError(
                    format!("Thread '{}' already exists in the cosmic void", name)
                ).into());
            }
            threads.insert(name, handle);
        }
        Ok(())
    }

    pub fn unregister_thread(&self, name: &str) -> RuntimeResult<Option<JoinHandle<RuntimeResult<()>>>> {
        let mut threads = self.threads.lock().unwrap();
        if !threads.contains_key(name) {
            return Err(RuntimeError::AsyncError(
                format!("Thread '{}' not found in the cosmic void", name)
            ).into());
        }
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