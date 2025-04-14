use crate::ast::{Statement, Value, Expression, Type, BinaryOperator, ComparisonOperator, LogicalOperator, FishOperation, BubbleFormat, TideOperation, WaveOperation, SplashOperation, CurrentOperation};
use crate::value_parser::ParseError;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;
use tokio::task::JoinHandle;
use rayon::prelude::*;
use futures::future::try_join_all;
use futures::future::BoxFuture;
use futures::FutureExt;
use std::future::Future;
use std::pin::Pin;

#[derive(Debug, Clone)]
pub enum RuntimeError {
    InvalidOperation(String),
    TypeError(String),
    UndefinedVariable(String),
    UndefinedFunction(String),
    IndexOutOfBounds(String),
    FileError(String),
    DivisionByZero,
    VariableNotFound(String),
    AsyncError(String),
    Return(Value),
}

impl From<RuntimeError> for ParseError {
    fn from(err: RuntimeError) -> Self {
        match err {
            RuntimeError::TypeError(msg) => ParseError::InvalidValue(msg),
            RuntimeError::DivisionByZero => ParseError::InvalidValue("Divisão por zero".to_string()),
            RuntimeError::VariableNotFound(name) => ParseError::InvalidValue(format!("Variável não encontrada: {}", name)),
            RuntimeError::InvalidOperation(msg) => ParseError::InvalidValue(msg),
            RuntimeError::AsyncError(msg) => ParseError::InvalidValue(msg),
            RuntimeError::UndefinedVariable(name) => ParseError::InvalidValue(format!("Variável não definida: {}", name)),
            RuntimeError::UndefinedFunction(name) => ParseError::InvalidValue(format!("Função não definida: {}", name)),
            RuntimeError::IndexOutOfBounds(msg) => ParseError::InvalidValue(msg),
            RuntimeError::FileError(msg) => ParseError::InvalidValue(msg),
            RuntimeError::Return(value) => ParseError::RuntimeError(RuntimeError::Return(value)),
        }
    }
}

type RuntimeResult<T> = Result<T, ParseError>;

#[derive(Debug, Clone)]
pub enum ReturnValue {
    None,
    Value(Value),
}

struct Environment {
    variables: Arc<Mutex<HashMap<String, Value>>>,
    functions: Arc<Mutex<HashMap<String, (Vec<String>, Vec<Statement>)>>>,
    structs: Arc<Mutex<HashMap<String, Vec<(String, Type)>>>>,
}

impl Environment {
    fn new() -> Self {
        Self {
            variables: Arc::new(Mutex::new(HashMap::new())),
            functions: Arc::new(Mutex::new(HashMap::new())),
            structs: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn get_var(&self, name: &str) -> RuntimeResult<Value> {
        self.variables
            .lock()
            .unwrap()
            .get(name)
            .cloned()
            .ok_or_else(|| RuntimeError::VariableNotFound(name.to_string()).into())
    }

    fn set_var(&self, name: String, value: Value) {
        self.variables.lock().unwrap().insert(name, value);
    }

    fn get_function(&self, name: &str) -> RuntimeResult<(Vec<String>, Vec<Statement>)> {
        self.functions
            .lock()
            .unwrap()
            .get(name)
            .cloned()
            .ok_or_else(|| RuntimeError::InvalidOperation(format!("Função não encontrada: {}", name)).into())
    }

    fn set_function(&self, name: String, args: Vec<String>, body: Vec<Statement>) {
        self.functions.lock().unwrap().insert(name, (args, body));
    }
}

struct AsyncManager {
    runtime: Runtime,
    threads: Arc<Mutex<HashMap<String, JoinHandle<RuntimeResult<()>>>>>,
}

impl AsyncManager {
    fn new() -> Self {
        Self {
            runtime: Runtime::new().unwrap(),
            threads: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn spawn_thread(&self, name: Option<String>, future: JoinHandle<RuntimeResult<()>>) {
        if let Some(name) = name {
            self.threads.lock().unwrap().insert(name, future);
        }
    }

    fn spawn_thread_blocking(&self, name: Option<String>, future: JoinHandle<RuntimeResult<()>>) {
        if let Some(name) = name {
            self.threads.lock().unwrap().insert(name, future);
        }
    }

    async fn wait_for_threads(&self, names: &[String]) -> RuntimeResult<()> {
        let mut handles = Vec::new();
        let mut threads = self.threads.lock().unwrap();
        
        for name in names {
            if let Some(handle) = threads.remove(name) {
                handles.push(handle);
            }
        }

        let results = try_join_all(handles)
            .await
            .map_err(|e| RuntimeError::AsyncError(e.to_string()))?;
            
        for result in results {
            result?;
        }
        Ok(())
    }
}

pub struct MurlocRuntime {
    env: Environment,
    async_manager: AsyncManager,
    recursion_depth: Arc<Mutex<usize>>,
    max_recursion_depth: usize,
}

impl MurlocRuntime {
    pub fn new() -> Self {
        Self {
            env: Environment::new(),
            async_manager: AsyncManager::new(),
            recursion_depth: Arc::new(Mutex::new(0)),
            max_recursion_depth: 100,
        }
    }

    pub fn run(&self, statements: Vec<Statement>) -> Result<(), ParseError> {
        self.async_manager.runtime.block_on(async {
            self.exec_block_impl(&statements).await
        })
    }

    pub async fn exec_block_impl(&self, statements: &[Statement]) -> RuntimeResult<()> {
        for statement in statements {
            let result = Box::pin(self.execute_statement(statement)).await;
            
            if let Err(err) = result {
                match &err {
                    ParseError::RuntimeError(RuntimeError::Return(_)) => {
                        return Err(err);
                    },
                    _ => return Err(err),
                }
            }
        }
        Ok(())
    }

    async fn execute_statement(&self, statement: &Statement) -> RuntimeResult<()> {
        let result = match statement {
            Statement::AsyncFunction { name, args, body } => {
                self.env.set_function(name.to_string(), args.clone(), body.clone());
                Ok::<(), ParseError>(())
            },
            Statement::Spawn { body, thread_name } => {
                let body_clone = body.clone();
                
                let vars_copy = self.env.variables.lock().unwrap().clone();
                let funcs_copy = self.env.functions.lock().unwrap().clone();
                let structs_copy = self.env.structs.lock().unwrap().clone();
                
                let handle = self.async_manager.runtime.spawn_blocking(move || {
                    let rt = Runtime::new().unwrap();
                    
                    rt.block_on(async {
                        let thread_runtime = MurlocRuntime {
                            env: Environment {
                                variables: Arc::new(Mutex::new(vars_copy)),
                                functions: Arc::new(Mutex::new(funcs_copy)),
                                structs: Arc::new(Mutex::new(structs_copy)),
                            },
                            async_manager: AsyncManager::new(),
                            recursion_depth: Arc::new(Mutex::new(0)),
                            max_recursion_depth: 500,
                        };
                        
                        thread_runtime.exec_block_impl(&body_clone).await
                    })
                });
                
                self.async_manager.spawn_thread_blocking(thread_name.clone(), handle);
                Ok::<(), ParseError>(())
            },
            Statement::SpawnAsync { future, thread_name } => {
                let future_clone = Box::new(*future.clone());

                let vars_copy = self.env.variables.lock().unwrap().clone();
                let funcs_copy = self.env.functions.lock().unwrap().clone();
                let structs_copy = self.env.structs.lock().unwrap().clone();
                
                let handle = self.async_manager.runtime.spawn_blocking(move || {
                    let rt = Runtime::new().unwrap();
                    
                    rt.block_on(async {
                        let thread_runtime = MurlocRuntime {
                            env: Environment {
                                variables: Arc::new(Mutex::new(vars_copy)),
                                functions: Arc::new(Mutex::new(funcs_copy)),
                                structs: Arc::new(Mutex::new(structs_copy)),
                            },
                            async_manager: AsyncManager::new(),
                            recursion_depth: Arc::new(Mutex::new(0)),
                            max_recursion_depth: 500,
                        };
                        
                        thread_runtime.exec_block_impl(&[*future_clone]).await
                    })
                });
                
                self.async_manager.spawn_thread_blocking(thread_name.clone(), handle);
                Ok::<(), ParseError>(())
            },
            Statement::ThreadPool { size, tasks } => {
                let size = match evaluate_expression(size, &self.env.variables.lock().unwrap())? {
                    Value::Number(n) => n as usize,
                    _ => return Err(RuntimeError::TypeError("Tamanho do pool deve ser um número".to_string()).into()),
                };

                let pool = rayon::ThreadPoolBuilder::new()
                    .num_threads(size)
                    .build()
                    .unwrap();

                let vars_copy = self.env.variables.lock().unwrap().clone();
                let funcs_copy = self.env.functions.lock().unwrap().clone();
                let structs_copy = self.env.structs.lock().unwrap().clone();

                pool.install(|| {
                    tasks.par_iter().try_for_each(|task| {
                        let task_clone = task.clone();
                        let vars = vars_copy.clone();
                        let funcs = funcs_copy.clone();
                        let structs = structs_copy.clone();

                        let rt = Runtime::new().unwrap();
                        
                        rt.block_on(async {
                            let thread_runtime = MurlocRuntime {
                                env: Environment {
                                    variables: Arc::new(Mutex::new(vars)),
                                    functions: Arc::new(Mutex::new(funcs)),
                                    structs: Arc::new(Mutex::new(structs)),
                                },
                                async_manager: AsyncManager::new(),
                                recursion_depth: Arc::new(Mutex::new(0)),
                                max_recursion_depth: 500,
                            };

                            thread_runtime.exec_block_impl(&[task_clone]).await
                        })
                    })
                })?;
                Ok::<(), ParseError>(())
            },
            Statement::Wait { thread_names } => {
                self.async_manager.wait_for_threads(thread_names).await?;
                Ok::<(), ParseError>(())
            },
            Statement::Await { future } => {
                let future_clone = future.clone();

                self.exec_block_impl(&[*future_clone]).await?;
                Ok::<(), ParseError>(())
            },
            _ => self.execute_non_async_statement(statement).await,
        };
        
        match result {
            Err(err) => Err(err),
            Ok(val) => Ok(val),
        }
    }

    async fn execute_non_async_statement(&self, statement: &Statement) -> RuntimeResult<()> {
        match statement {
            Statement::Import(path) => {
                let contents = fs::read_to_string(path)
                    .map_err(|e| RuntimeError::InvalidOperation(format!("Erro ao importar '{}': {}", path, e)))?;
                let spanned_tokens = crate::lexer::tokenize(&contents);
                let tokens: Vec<crate::lexer::Token> = spanned_tokens.iter().map(|t| t.token.clone()).collect();
                
                let positions: Vec<(usize, usize)> = spanned_tokens.iter().map(|t| (t.line, t.column)).collect();
                
                let imported_stmts = crate::parser::parse(tokens).map_err(|e| {
                    match &e {
                        ParseError::UnexpectedToken(msg) => {
                            if let Some((line, column)) = positions.get(0) {
                                ParseError::InvalidValue(format!("Erro na linha {}, coluna {}: {}", line, column, msg))
                            } else {
                                ParseError::InvalidValue(format!("Erro de sintaxe: {}", msg))
                            }
                        },
                        _ => ParseError::InvalidValue(format!("Erro de análise: {}", e))
                    }
                })?;
                self.exec_block_impl(&imported_stmts).await?;
                Ok::<(), ParseError>(())
            },
            Statement::Function { name, args, body } => {
                self.env.set_function(name.to_string(), args.clone(), body.clone());
                println!("🔧 Função '{}' definida", name);
                Ok(())
            },
            Statement::VarDeclaration(name, value) => {
                self.env.set_var(name.to_string(), value.clone());
                println!("💾 {} = {:?}", name, value);
                Ok(())
            },
            Statement::VarDeclarationExpr(name, expr) => {
                let value = evaluate_expression(expr, &self.env.variables.lock().unwrap())?;
                println!("💾 {} = {:?}", name, &value);
                self.env.set_var(name.to_string(), value);
                Ok(())
            },
            Statement::Assignment(name, expr) => {
                let value = evaluate_expression(expr, &self.env.variables.lock().unwrap())?;
                self.env.set_var(name.to_string(), value.clone());
                println!("✏️ {} = {:?}", name, &value);
                Ok(())
            },
            Statement::CallFunction { name, args } => {
                println!("🔍 Chamando função '{}'...", name);
                
                let (params, body) = self.env.get_function(name)?;

                let mut local_vars: HashMap<String, Value> = HashMap::new();

                if !args.is_empty() {
                    if args.len() != params.len() {
                        println!("⚠️ Aviso: Número de argumentos ({}) diferente do número de parâmetros ({}) para função '{}'",
                                args.len(), params.len(), name);
                    }
                    
                    let env_vars = self.env.variables.lock().unwrap();
                    
                    for (i, arg) in args.iter().enumerate() {
                        if i < params.len() {
                            let arg_value = evaluate_expression(arg, &env_vars)?;
                            local_vars.insert(params[i].clone(), arg_value.clone());
                            println!("  📥 Parâmetro {}: {} = {:?}", i + 1, params[i], arg_value);
                        }
                    }
                }
                
                Box::pin(self.call_function_impl(name, local_vars, &body)).await
            },
            Statement::IfStatement { condition, body } => {
                if evaluate_condition(condition, &self.env.variables.lock().unwrap()) {
                    self.exec_block_impl(body).await?;
                }
                Ok(())
            },
            Statement::WhileLoop { condition, body } => {
                while evaluate_condition(condition, &self.env.variables.lock().unwrap()) {
                    self.exec_block_impl(body).await?;
                }
                Ok(())
            },
            Statement::Print(expr) => {
                let value = evaluate_expression(expr, &self.env.variables.lock().unwrap())?;
                println!("📢 {}", &value);
                Ok(())
            },
            Statement::Return(expr) => {
                let value = evaluate_expression(expr, &self.env.variables.lock().unwrap())?;
                self.env.set_var("retorno".to_string(), value.clone());
                println!("🔄 Retornando valor: {:?}", value);
  
                return Err(RuntimeError::Return(value).into());
            },
            Statement::Read(name) => {
                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();
                let mut env = self.env.variables.lock().unwrap();
                env.insert(name.to_string(), Value::Text(input.trim().to_string()));
                Ok(())
            },
            Statement::ArrayDeclaration { name, elements } => {
                let mut env = self.env.variables.lock().unwrap();
                env.insert(name.to_string(), Value::Array(elements.clone()));
                Ok(())
            },
            Statement::StructDeclaration { name, fields } => {
                let mut structs = self.env.structs.lock().unwrap();
                structs.insert(name.to_string(), fields.clone());
                Ok(())
            },
            Statement::Loop { variable, start, end, body } => {
                for i in *start..=*end {
                    let mut env = self.env.variables.lock().unwrap();
                    env.insert(variable.to_string(), Value::Number(i));
                    drop(env);
                    self.exec_block_impl(body).await?;
                }
                Ok(())
            },
            Statement::ForLoop { init_var, init_value, condition, increment_var, increment_expr, body } => {
                let init_result = evaluate_expression(init_value, &self.env.variables.lock().unwrap())?;
                {
                    let mut env = self.env.variables.lock().unwrap();
                    env.insert(init_var.to_string(), init_result);
                }

                loop {
                    let cond_result = evaluate_expression(&condition, &self.env.variables.lock().unwrap())?;

                    let continue_loop = match cond_result {
                        Value::Number(n) => n != 0,
                        _ => return Err(RuntimeError::TypeError("Condição do loop deve retornar um número".to_string()).into()),
                    };

                    if !continue_loop {
                        break;
                    }

                    self.exec_block_impl(body).await?;

                    let incr_result = evaluate_expression(increment_expr, &self.env.variables.lock().unwrap())?;
                    {
                        let mut env = self.env.variables.lock().unwrap();
                        env.insert(increment_var.to_string(), incr_result);
                    }
                }
                Ok(())
            },
            Statement::LoopBlock { body } => {
                self.exec_block_impl(body).await?;
                Ok(())
            },
            Statement::SwitchStatement { value, cases, default } => {
                let val = evaluate_expression(value, &self.env.variables.lock().unwrap())?;
                let mut matched = false;
                
                for (case_value, case_body) in cases {
                    if &val == case_value {
                        self.exec_block_impl(case_body).await?;
                        matched = true;
                        break;
                    }
                }
                
                if !matched {
                    if let Some(default_body) = default {
                        self.exec_block_impl(default_body).await?;
                    }
                }
                Ok(())
            },
            Statement::Break => {
                Ok(())
            },
            Statement::Continue => {
                Ok(())
            },
            Statement::Sync { name } => {
                let mut threads = self.async_manager.threads.lock().unwrap();
                if let Some(handle) = threads.remove(name) {
                    handle.await.map_err(|e| RuntimeError::AsyncError(format!("Erro ao aguardar thread: {}", e)))??;
                }
                Ok(())
            },
            _ => {
                match statement {
                    Statement::FishArray { name, elements, operation } => {
                        let mut env = self.env.variables.lock().unwrap();
                        match operation {
                            FishOperation::Add => {
                                if let Some(Value::Array(arr)) = env.get(name) {
                                    let mut new_arr = arr.clone();
                                    new_arr.extend(elements.clone());
                                    env.insert(name.to_string(), Value::Array(new_arr));
                                } else {
                                    env.insert(name.to_string(), Value::Array(elements.clone()));
                                }
                            },
                            FishOperation::Remove => {
                                if let Some(Value::Array(arr)) = env.get(name) {
                                    let new_arr: Vec<Value> = arr.iter()
                                        .filter(|&x| !elements.contains(x))
                                        .cloned()
                                        .collect();
                                    env.insert(name.to_string(), Value::Array(new_arr));
                                }
                            },
                            FishOperation::Find => {
                                if let Some(Value::Array(arr)) = env.get(name) {
                                    let found: Vec<Value> = arr.iter()
                                        .filter(|&x| elements.contains(x))
                                        .cloned()
                                        .collect();
                                    env.insert(name.to_string(), Value::Array(found));
                                }
                            },
                            FishOperation::Sort => {
                                if let Some(Value::Array(arr)) = env.get(name) {
                                    let mut sorted = arr.clone();
                                    sorted.sort_by(|a, b| {
                                        let a_str = match a {
                                            Value::Number(n) => n.to_string(),
                                            Value::NumberI64(n) => n.to_string(),
                                            Value::NumberBig(n) => n.to_string(),
                                            Value::Text(s) => s.clone(),
                                            Value::Array(values) => todo!(),
                                            Value::Struct(_, items) => todo!(),
                                            Value::Future(statement) => todo!(),
                                            Value::Thread(_) => todo!(),
                                        };
                                        let b_str = match b {
                                            Value::Number(n) => n.to_string(),
                                            Value::NumberI64(n) => n.to_string(),
                                            Value::NumberBig(n) => n.to_string(),
                                            Value::Text(s) => s.clone(),
                                            Value::Array(values) => todo!(),
                                            Value::Struct(_, items) => todo!(),
                                            Value::Future(statement) => todo!(),
                                            Value::Thread(_) => todo!(),
                                        };
                                        a_str.cmp(&b_str)
                                    });
                                    env.insert(name.to_string(), Value::Array(sorted));
                                }
                            },
                        }
                        Ok(())
                    },
                    _ => Ok(())
                }
            }
        }
    }

    pub async fn exec_block(&self, statements: &[Statement]) -> RuntimeResult<()> {
        self.exec_block_impl(statements).await
    }

    pub fn call_function_expr(&self, name: &str, args: Vec<Value>) -> RuntimeResult<Value> {
        {
            let mut depth = self.recursion_depth.lock().unwrap();
            *depth += 1;
            
            if *depth > self.max_recursion_depth {
                *depth -= 1;
                return Err(RuntimeError::InvalidOperation(
                    format!("Estouro de pilha: Atingido limite máximo de recursão ({}) na função '{}'", 
                            self.max_recursion_depth, name)
                ).into());
            }
        }
        
        let (param_names, body) = self.env.get_function(name)?;

        let mut call_vars = self.env.variables.lock().unwrap().clone();
        if args.len() != param_names.len() {
            let mut depth = self.recursion_depth.lock().unwrap();
            *depth -= 1;
            
            return Err(RuntimeError::InvalidOperation(format!(
                "Função '{}' espera {} argumentos, mas recebeu {}",
                name, param_names.len(), args.len()
            )).into());
        }
        
        for (param, arg) in param_names.iter().zip(args.iter()) {
            call_vars.insert(param.clone(), arg.clone());
        }
        
        let function_env = Environment {
            variables: Arc::new(Mutex::new(call_vars)),
            functions: self.env.functions.clone(),
            structs: self.env.structs.clone(),
        };
        
        let function_runtime = MurlocRuntime {
            env: function_env,
            async_manager: AsyncManager::new(),
            recursion_depth: self.recursion_depth.clone(),
            max_recursion_depth: self.max_recursion_depth,
        };
        
        let result = match function_runtime.async_manager.runtime.block_on(function_runtime.exec_block_impl(&body)) {
            Err(e) => match e {
                ParseError::RuntimeError(RuntimeError::Return(val)) => {
                    Ok(val)
                },
                _ => Err(e),
            },
            Ok(()) => {
                let vars = function_runtime.env.variables.lock().unwrap();
                if let Some(return_val) = vars.get("retorno") {
                    Ok(return_val.clone())
                } else {
                    Ok(Value::Number(0))
                }
            }
        };
        
        {
            let mut depth = self.recursion_depth.lock().unwrap();
            *depth -= 1;
        }
        
        result
    }

    async fn call_function_impl(&self, name: &str, local_vars: HashMap<String, Value>, body: &[Statement]) -> RuntimeResult<()> {
        let current_vars = self.env.variables.lock().unwrap().clone();
        
        for (param, value) in local_vars.iter() {
            self.env.set_var(param.clone(), value.clone());
        }
        
        let result = self.exec_block_impl(body).await;
        
        let retorno = if let Ok(ret) = self.env.get_var("retorno") {
            Some(ret)
        } else {
            None
        };
        
        let mut vars = self.env.variables.lock().unwrap();
        *vars = current_vars;
        
        if let Some(ret) = retorno {
            vars.insert("retorno".to_string(), ret.clone());
            println!("🔙 Função '{}' retornou {:?}", name, ret);
        }
        
        match result {
            Err(ParseError::RuntimeError(RuntimeError::Return(_))) => Ok(()),
            Err(e) => Err(e),
            Ok(()) => Ok(())
        }
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

fn evaluate_condition(condition: &Expression, env: &HashMap<String, Value>) -> bool {
    if let Ok(Value::Number(n)) = evaluate_expression(condition, env) {
        n != 0
    } else {
        panic!("Condição inválida");
    }
}

fn evaluate_expression(expr: &Expression, env: &HashMap<String, Value>) -> Result<Value, ParseError> {
    match expr {
        Expression::Equals(name, value) => {
            Ok(Value::Number(*value))
        },
        Expression::BinaryOp { left, right, op } => {
            let left_val = evaluate_expression(left, env)?;
            let right_val = evaluate_expression(right, env)?;
            
            match op {
                BinaryOperator::Add => {
                    match (&left_val, &right_val) {
                        (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
                        (Value::Text(a), Value::Text(b)) => Ok(Value::Text(format!("{}{}", a, b))),
                        (Value::Text(a), Value::Number(b)) => Ok(Value::Text(format!("{}{}", a, b))),
                        (Value::Number(a), Value::Text(b)) => Ok(Value::Text(format!("{}{}", a, b))),
                        _ => Err(ParseError::InvalidValue("Operação de adição inválida".to_string())),
                    }
                },
                _ => match (left_val, right_val) {
                    (Value::Number(a), Value::Number(b)) => {
                        match op {
                            BinaryOperator::Add => Ok(Value::Number(a + b)),
                            BinaryOperator::Subtract => Ok(Value::Number(a - b)),
                            BinaryOperator::Multiply => Ok(Value::Number(a * b)),
                            BinaryOperator::Divide => {
                                if b == 0 {
                                    Err(ParseError::InvalidValue("Divisão por zero".to_string()))
                                } else {
                                    Ok(Value::Number(a / b))
                                }
                            },
                            BinaryOperator::Modulo => Ok(Value::Number(a % b)),
                        }
                    },
                    _ => Err(ParseError::InvalidValue("Operação aritmética inválida".to_string())),
                }
            }
        },
        Expression::Comparison { left, right, op } => {
            let left_val = evaluate_expression(left, env)?;
            let right_val = evaluate_expression(right, env)?;
            
            let result = match op {
                ComparisonOperator::Equals => left_val == right_val,
                ComparisonOperator::NotEquals => left_val != right_val,
                ComparisonOperator::LessThan => left_val < right_val,
                ComparisonOperator::GreaterThan => left_val > right_val,
                ComparisonOperator::LessThanOrEqual => left_val <= right_val,
                ComparisonOperator::GreaterThanOrEqual => left_val >= right_val,
            };
            
            Ok(Value::Number(if result { 1 } else { 0 }))
        },
        Expression::LogicalOp { left, right, op } => {
            let left_val = evaluate_expression(left, env)?;
            
            match op {
                LogicalOperator::Not => {
                    if right.is_some() {
                        return Err(ParseError::InvalidValue("Operador 'Not' não deve ter operando direito".to_string()));
                    }
                    match left_val {
                        Value::Number(a) => Ok(Value::Number(if a == 0 { 1 } else { 0 })),
                        _ => Err(ParseError::InvalidValue("Operando lógico 'Not' deve ser um número".to_string())),
                    }
                },
                _ => {
                    let right_val = evaluate_expression(right.as_ref().unwrap(), env)?;
                    match (left_val, right_val) {
                        (Value::Number(a), Value::Number(b)) => {
                            let result = match op {
                                LogicalOperator::And => a != 0 && b != 0,
                                LogicalOperator::Or => a != 0 || b != 0,
                                LogicalOperator::Not => unreachable!(),
                            };
                            Ok(Value::Number(if result { 1 } else { 0 }))
                        },
                        _ => Err(ParseError::InvalidValue("Operandos lógicos devem ser números".to_string())),
                    }
                }
            }
        },
        Expression::Literal(value) => Ok(value.clone()),
        Expression::Variable(name) => {
            if let Some(value) = env.get(name) {
                Ok(value.clone())
            } else {
                Err(ParseError::InvalidValue(format!("Variável não encontrada: {}", name)))
            }
        },
        Expression::ArrayAccess { name, index } => {
            if let Some(Value::Array(arr)) = env.get(name) {
                if let Ok(Value::Number(idx)) = evaluate_expression(index, env) {
                    let idx = idx as usize;
                    if idx < arr.len() {
                        Ok(arr[idx].clone())
                    } else {
                        Err(ParseError::InvalidValue(format!("Índice fora do alcance do array: {}", idx)))
                    }
                } else {
                    Err(ParseError::InvalidValue("Índice de array inválido".to_string()))
                }
            } else {
                Err(ParseError::InvalidValue(format!("Array não encontrado: {}", name)))
            }
        },
        Expression::StructAccess { name, field } => {
            if let Some(Value::Struct(_, fields)) = env.get(name) {
                if let Some((_, value)) = fields.iter().find(|(f, _)| f == field) {
                    Ok(value.clone())
                } else {
                    Err(ParseError::InvalidValue(format!("Campo não encontrado: {}", field)))
                }
            } else {
                Err(ParseError::InvalidValue(format!("Struct não encontrada: {}", name)))
            }
        },
        Expression::FunctionCall { name, args } => {
            Err(ParseError::InvalidValue(format!(
                "Chamada de função '{}' não pode ser avaliada diretamente em uma expressão",
                name
            )))
        },
    }
}
