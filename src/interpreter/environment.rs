use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::ast::{Statement, Value, Expression, Type};
use crate::interpreter::error::{RuntimeError, RuntimeResult};
use crate::interpreter::evaluator::evaluate_expression;

pub struct Environment {
    pub variables: Arc<Mutex<HashMap<String, Value>>>,
    pub functions: Arc<Mutex<HashMap<String, (Vec<String>, Vec<Statement>)>>>,
    pub structs: Arc<Mutex<HashMap<String, Vec<(String, Type)>>>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            variables: Arc::new(Mutex::new(HashMap::new())),
            functions: Arc::new(Mutex::new(HashMap::new())),
            structs: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get_var(&self, name: &str) -> RuntimeResult<Value> {
        self.variables
            .lock()
            .unwrap()
            .get(name)
            .cloned()
            .ok_or_else(|| RuntimeError::VariableNotFound(format!("Variable '{}' lost in the cosmic void", name)).into())
    }

    pub fn set_var(&self, name: String, value: Value) {
        self.variables.lock().unwrap().insert(name, value);
    }

    pub fn get_function(&self, name: &str) -> RuntimeResult<(Vec<String>, Vec<Statement>)> {
        self.functions
            .lock()
            .unwrap()
            .get(name)
            .cloned()
            .ok_or_else(|| RuntimeError::InvalidOperation(format!("Function '{}' not found in the ritual book", name)).into())
    }

    pub fn set_function(&self, name: String, args: Vec<String>, body: Vec<Statement>) {
        self.functions.lock().unwrap().insert(name, (args, body));
    }
    
    pub fn with_locked_vars<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut HashMap<String, Value>) -> R,
    {
        let mut guard = self.variables.lock().unwrap();
        f(&mut guard)
    }
    
    pub fn evaluate(&self, expr: &Expression) -> RuntimeResult<Value> {
        evaluate_expression(expr, &self.variables.lock().unwrap(), None)
    }
    
    pub fn evaluate_with_runtime(&self, expr: &Expression, runtime: &super::runtime::MurlocRuntime) -> RuntimeResult<Value> {
        evaluate_expression(expr, &self.variables.lock().unwrap(), Some(runtime))
    }
}

impl Clone for Environment {
    fn clone(&self) -> Self {
        Self {
            variables: Arc::clone(&self.variables),
            functions: Arc::clone(&self.functions),
            structs: Arc::clone(&self.structs),
        }
    }
} 