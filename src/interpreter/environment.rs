use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::ast::{Statement, Value, Expression, Type};
use crate::interpreter::error::{RuntimeError, RuntimeResult};
use crate::interpreter::evaluator::evaluate_expression;
use crate::value_parser::ParseError;
pub struct Environment {
    pub variables: Arc<Mutex<HashMap<String, Value>>>,
    pub functions: Arc<Mutex<HashMap<String, (Vec<String>, Vec<Statement>)>>>,
    pub structs: Arc<Mutex<HashMap<String, Vec<(String, Type)>>>>,
    pub exports: Arc<Mutex<HashMap<String, bool>>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            variables: Arc::new(Mutex::new(HashMap::new())),
            functions: Arc::new(Mutex::new(HashMap::new())),
            structs: Arc::new(Mutex::new(HashMap::new())),
            exports: Arc::new(Mutex::new(HashMap::new())),
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

    pub fn get_function_sync(&self, name: &str) -> Option<(Vec<String>, Vec<Statement>)> {
        self.functions
            .lock()
            .unwrap()
            .get(name)
            .cloned()
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
        match expr {
            Expression::FunctionCall { name, args } => {
                let evaluated_args = args.iter()
                    .map(|arg| self.evaluate(arg))
                    .collect::<Result<Vec<Value>, ParseError>>()?;
                
                match self.execute_sync_function(name, evaluated_args) {
                    Ok(result) => Ok(result),
                    Err(_) => {
                        evaluate_expression(expr, &self.variables.lock().unwrap(), Some(runtime))
                    }
                }
            },
            _ => evaluate_expression(expr, &self.variables.lock().unwrap(), Some(runtime))
        }
    }

    pub fn execute_sync_function(&self, name: &str, args: Vec<Value>) -> RuntimeResult<Value> {
        let (param_names, body) = self.get_function_sync(name)
            .ok_or_else(|| RuntimeError::InvalidOperation(format!("Function '{}' not found in the ritual book", name)))?;

        if args.len() != param_names.len() {
            return Err(RuntimeError::InvalidOperation(format!(
                "Function '{}' expects {} arguments, but got {}",
                name, param_names.len(), args.len()
            )).into());
        }

        let mut function_env = self.variables.lock()
            .map_err(|e| RuntimeError::LockError(format!("Failed to lock variables: {}", e)))?
            .clone();
        
        for (param, arg) in param_names.iter().zip(args.iter()) {
            function_env.insert(param.clone(), arg.clone());
        }
        
        let mut result = Value::Number(0);
        for stmt in body {
            match stmt {
                Statement::Return(expr) => {
                    result = evaluate_expression(&expr, &function_env, None)?;
                    break;
                },
                Statement::VarDeclaration(name, value) => {
                    function_env.insert(name, value);
                },
                Statement::VarDeclarationExpr(name, expr) => {
                    let value = evaluate_expression(&expr, &function_env, None)?;
                    function_env.insert(name, value);
                },
                Statement::Assignment(name, expr) => {
                    let value = evaluate_expression(&expr, &function_env, None)?;
                    function_env.insert(name, value);
                },
                Statement::Expr(expr) => {
                    evaluate_expression(&expr, &function_env, None)?;
                },
                _ => continue,
            }
        }
        
        Ok(result)
    }

    pub fn execute_async_function(&self, name: &str, args: Vec<Value>) -> RuntimeResult<Value> {
        let (param_names, body) = self.get_function(name)?;

        if args.len() != param_names.len() {
            return Err(RuntimeError::InvalidOperation(format!(
                "Function '{}' expects {} arguments, but got {}",
                name, param_names.len(), args.len()
            )).into());
        }

        Ok(Value::Future(Box::new(Statement::Function {
            name: name.to_string(),
            args: param_names,
            body,
            parent_scope: None,
        })))
    }

    pub fn is_async_function(&self, name: &str) -> bool {
        self.functions.lock().unwrap()
            .get(name)
            .map(|(_, body)| {
                body.iter().any(|stmt| matches!(stmt, Statement::AsyncFunction { .. }))
            })
            .unwrap_or(false)
    }

    pub fn add_export(&self, name: String, is_default: bool) -> Result<(), RuntimeError> {
        let mut exports = self.exports.lock()
            .map_err(|e| RuntimeError::LockError(format!("Failed to lock exports: {}", e)))?;
        exports.insert(name, is_default);
        Ok(())
    }

    pub fn is_exported(&self, name: &str) -> Result<bool, RuntimeError> {
        let exports = self.exports.lock()
            .map_err(|e| RuntimeError::LockError(format!("Failed to lock exports: {}", e)))?;
        Ok(exports.contains_key(name))
    }

    pub fn is_default_export(&self, name: &str) -> Result<bool, RuntimeError> {
        let exports = self.exports.lock()
            .map_err(|e| RuntimeError::LockError(format!("Failed to lock exports: {}", e)))?;
        Ok(exports.get(name).map_or(false, |&is_default| is_default))
    }
}

impl Clone for Environment {
    fn clone(&self) -> Self {
        Self {
            variables: Arc::clone(&self.variables),
            functions: Arc::clone(&self.functions),
            structs: Arc::clone(&self.structs),
            exports: Arc::clone(&self.exports),
        }
    }
} 