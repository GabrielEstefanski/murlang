use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::fs;
use std::io;
use tokio::runtime::Runtime;
use log::{info, warn, error};

use crate::ast::{Statement, Value, Expression, Type };
use crate::value_parser::ParseError;

use crate::interpreter::environment::Environment;
use crate::interpreter::async_manager::AsyncManager;
use crate::interpreter::error::{RuntimeError, RuntimeResult};
use crate::interpreter::evaluator::{evaluate_condition, evaluate_expression};

pub struct MurlocRuntime {
    pub env: Environment,
    pub async_manager: AsyncManager,
    pub recursion_depth: Arc<Mutex<usize>>,
    pub max_recursion_depth: usize,
    pub runtime: Arc<Runtime>,
}

impl MurlocRuntime {
    pub fn new() -> Self {
        let runtime = Runtime::new().expect("Failed to create Tokio runtime");
        
        Self {
            env: Environment::new(),
            async_manager: AsyncManager::new(),
            recursion_depth: Arc::new(Mutex::new(0)),
            max_recursion_depth: 500,
            runtime: Arc::new(runtime),
        }
    }

    pub fn run(&self, statements: Vec<Statement>) -> Result<(), ParseError> {
        self.runtime.block_on(async {
            self.exec_block_impl(&statements).await
        })
    }

    pub async fn exec_block_impl(&self, statements: &[Statement]) -> RuntimeResult<()> 
    where
        Self: Send + Sync,
    {
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

    async fn execute_statement(&self, statement: &Statement) -> RuntimeResult<()> 
    where
        Self: Send + Sync,
    {
        match statement {
            Statement::AsyncFunction { name, args, body, parent_scope } => {
                self.env.set_function(name.to_string(), args.clone(), body.clone());
                Ok::<(), ParseError>(())
            },
            Statement::Spawn { body, thread_name } => {
                let vars_shared = self.env.variables.clone();
                let funcs_shared = self.env.functions.clone();
                let structs_shared = self.env.structs.clone();
                
                let runtime_clone = self.runtime.clone();
                let body_clone = body.clone();
                let recursion_depth_clone = self.recursion_depth.clone();
                
                let handle = self.runtime.spawn_blocking(move || {
                    let runtime_for_block_on = runtime_clone.clone();
                    let thread_runtime = MurlocRuntime {
                        env: Environment {
                            variables: vars_shared,
                            functions: funcs_shared,
                            structs: structs_shared,
                        },
                        async_manager: AsyncManager::new(),
                        recursion_depth: recursion_depth_clone,
                        max_recursion_depth: 500,
                        runtime: runtime_clone,
                    };
                    
                    runtime_for_block_on.block_on(thread_runtime.exec_block_impl(&body_clone))
                });
                
                self.async_manager.register_thread(thread_name.clone(), handle);
                Ok(())
            },
            Statement::SpawnAsync { future, thread_name } => {
                let vars_copy = {
                    let vars = self.env.variables.lock()
                        .map_err(|e| RuntimeError::LockError(format!("Failed to lock variables: {}", e)))?;
                    vars.clone()
                };
                
                let funcs_copy = {
                    let funcs = self.env.functions.lock()
                        .map_err(|e| RuntimeError::LockError(format!("Failed to lock functions: {}", e)))?;
                    funcs.clone()
                };
                
                let structs_copy = {
                    let structs = self.env.structs.lock()
                        .map_err(|e| RuntimeError::LockError(format!("Failed to lock structs: {}", e)))?;
                    structs.clone()
                };
                
                let runtime_clone = self.runtime.clone();
                let future_clone = (**future).clone();
                let recursion_depth_clone = self.recursion_depth.clone();
                
                let handle = self.runtime.spawn_blocking(move || {
                    let runtime_for_block_on = runtime_clone.clone();
                    let thread_runtime = MurlocRuntime {
                        env: Environment {
                            variables: Arc::new(Mutex::new(vars_copy)),
                            functions: Arc::new(Mutex::new(funcs_copy)),
                            structs: Arc::new(Mutex::new(structs_copy)),
                        },
                        async_manager: AsyncManager::new(),
                        recursion_depth: recursion_depth_clone,
                        max_recursion_depth: 500,
                        runtime: runtime_clone,
                    };
                    
                    runtime_for_block_on.block_on(thread_runtime.exec_block_impl(&[future_clone]))
                });
                
                self.async_manager.register_thread(thread_name.clone(), handle);
                Ok(())
            },
            Statement::ThreadPool { size, tasks } => {
                let size_value = match self.env.evaluate(size)? {
                    Value::Number(n) => n as usize,
                    _ => return Err(RuntimeError::TypeError("Thread pool size must be a number".to_string()).into()),
                };
                
                let pool = rayon::ThreadPoolBuilder::new()
                    .num_threads(size_value)
                    .build()
                    .map_err(|e| RuntimeError::InvalidOperation(format!("Failed to create thread pool: {}", e)))?;
                
                pool.install(|| {
                    for task in tasks {
                        let task_clone = task.clone();
                        
                        pool.spawn(move || {
                            let rtm = Runtime::new()
                                .map_err(|e| RuntimeError::InvalidOperation(format!("Failed to create runtime: {}", e)))
                                .expect("Failed to create runtime");
                            let runtime_clone = MurlocRuntime::new();
                            
                            if let Err(e) = rtm.block_on(runtime_clone.execute_statement(&task_clone)) {
                                error!("Failed to execute task in pool: {:?}", e);
                            }
                        });
                    }
                });
                
                Ok(())
            },
            Statement::Wait { thread_names } => {
                self.wait_for_threads(thread_names)?;
                Ok(())
            },
            Statement::Await { future } => {
                info!("Waiting for future to complete...");
                let future_clone = (**future).clone();
                
                let result_variables = Arc::new(Mutex::new(HashMap::new()));
                
                let env_clone = Environment {
                    variables: self.env.variables.clone(),
                    functions: self.env.functions.clone(),
                    structs: self.env.structs.clone(),
                };
                
                let runtime_clone = self.runtime.clone();
                let max_recursion = self.max_recursion_depth;
                let recursion_depth = self.recursion_depth.clone();
  
                {
                    let current_depth = recursion_depth.lock()
                        .map_err(|e| RuntimeError::LockError(format!("Failed to lock recursion depth: {}", e)))?;
                    if *current_depth > max_recursion / 2 {
                        return Err(RuntimeError::InvalidOperation(
                            format!("Excessive recursion detected while awaiting future. Current depth: {}", 
                                   *current_depth)
                        ).into());
                    }
                }
                
                let result_vars_clone = result_variables.clone();
                
                let thread_runtime = MurlocRuntime {
                    env: env_clone,
                    async_manager: AsyncManager::new(),
                    recursion_depth: recursion_depth,
                    max_recursion_depth: max_recursion,
                    runtime: runtime_clone.clone(),
                };
                
                let result = std::thread::spawn(move || {
                    runtime_clone.block_on(async move {
                        let result = thread_runtime.exec_block_impl(&[future_clone]).await;
                        
                        if result.is_ok() {
                            let vars = thread_runtime.env.variables.lock()
                                .map_err(|e| RuntimeError::LockError(format!("Failed to lock variables: {}", e)))?;
                            if let Some(return_val) = vars.get("retorno") {
                                let mut result_vars = result_vars_clone.lock()
                                    .map_err(|e| RuntimeError::LockError(format!("Failed to lock result variables: {}", e)))?;
                                result_vars.insert("retorno".to_string(), return_val.clone());
                            }
                        }
                        
                        result
                    })
                }).join()
                .map_err(|e| RuntimeError::InvalidOperation(format!("Failed to join thread: {:?}", e)))?;
                
                if let Ok(()) = result {
                    let result_vars = result_variables.lock()
                        .map_err(|e| RuntimeError::LockError(format!("Failed to lock result variables: {}", e)))?;
                    if let Some(return_val) = result_vars.get("retorno") {
                        self.env.set_var("retorno".to_string(), return_val.clone());
                    }
                }
                
                info!("Future completed");
                result
            },
            _ => self.execute_non_async_statement(statement).await,
        }
    }

    async fn execute_non_async_statement(&self, statement: &Statement) -> RuntimeResult<()> 
    where
        Self: Send + Sync,
    {
        match statement {
            Statement::Import(path) => {
                let contents = fs::read_to_string(path)
                    .map_err(|e| RuntimeError::InvalidOperation(format!("Error importing '{}': {}", path, e)))?;
                
                let spanned_tokens = crate::lexer::tokenize(&contents)
                    .map_err(|e| RuntimeError::LexerError(format!("At file {}: {}", path, e.message)))?;
                
                let tokens: Vec<crate::lexer::Token> = spanned_tokens.iter().map(|t| t.token.clone()).collect();
                
                let positions: Vec<(usize, usize)> = spanned_tokens.iter().map(|t| (t.line, t.column)).collect();
                
                let imported_stmts = crate::parser::parse(tokens).map_err(|e| {
                    match &e {
                        ParseError::UnexpectedToken(msg) => {
                            if let Some((line, column)) = positions.get(0) {
                                ParseError::InvalidValue(format!("Error at line {}, column {}: {}", line, column, msg))
                            } else {
                                ParseError::InvalidValue(format!("Syntax error: {}", msg))
                            }
                        },
                        _ => ParseError::InvalidValue(format!("Parse error: {}", e))
                    }
                })?;
                self.exec_block_impl(&imported_stmts).await?;
                Ok::<(), ParseError>(())
            },
            Statement::Function { name, args, body, parent_scope } => {
                self.env.set_function(name.to_string(), args.clone(), body.clone());
                info!("Function '{}' defined", name);
                Ok(())
            },
            Statement::VarDeclaration(name, value) => {
                self.env.set_var(name.to_string(), value.clone());
                info!("{} = {:?}", name, value);
                Ok(())
            },
            Statement::VarDeclarationExpr(name, expr) => {
                let value = self.env.evaluate_with_runtime(expr, self)?;
                
                if let Value::Struct(struct_name, fields) = &value {
                    let structs = self.env.structs.lock()
                        .map_err(|e| RuntimeError::LockError(format!("Failed to lock structs: {}", e)))?;
                    if let Some(struct_fields) = structs.get(struct_name) {
                        for (field_name, _) in fields {
                            if !struct_fields.iter().any(|(name, _)| name == field_name) {
                                return Err(RuntimeError::InvalidOperation(
                                    format!("Field '{}' does not exist in struct '{}'", field_name, struct_name)
                                ).into());
                            }
                        }
                    } else {
                        return Err(RuntimeError::InvalidOperation(
                            format!("Type '{}' not found in the cosmic void", struct_name)
                        ).into());
                    }
                }
                
                info!("{} = {:?}", name, &value);
                self.env.set_var(name.to_string(), value);
                Ok(())
            },
            Statement::Assignment(name, expr) => {
                let value = self.env.evaluate_with_runtime(expr, self)?;
                self.env.set_var(name.to_string(), value.clone());
                println!("[INFO] {} = {:?}", name, &value);
                Ok(())
            },
            Statement::CallFunction { name, args } => {
                println!("[INFO] Callingdasdasdsa function '{}'...", name);
                
                let (params, body) = self.env.get_function(name)?;

                let mut local_vars: HashMap<String, Value> = HashMap::new();

                if !args.is_empty() {
                    if args.len() != params.len() {
                        println!("[WARN] Number of arguments ({}) different from number of parameters ({}) for function '{}'",
                                args.len(), params.len(), name);
                    }
                    
                    let env_vars = self.env.variables.lock().unwrap();
                    
                    for (i, arg) in args.iter().enumerate() {
                        if i < params.len() {
                            let arg_value = evaluate_expression(arg, &env_vars, Some(self))?;
                            local_vars.insert(params[i].clone(), arg_value.clone());
                            println!("[INFO] Parameter {}: {} = {:?}", i + 1, params[i], arg_value);
                        }
                    }
                }
                
                Box::pin(self.call_function_impl(name, local_vars, &body)).await
            },
            Statement::IfStatement { condition, body } => {
                if evaluate_condition(condition, &self.env.variables.lock().unwrap(), Some(self)) {
                    self.exec_block_impl(body).await?;
                }
                Ok(())
            },
            Statement::WhileLoop { condition, body } => {
                while evaluate_condition(condition, &self.env.variables.lock().unwrap(), Some(self)) {
                    self.exec_block_impl(body).await?;
                }
                Ok(())
            },
            Statement::Print(expr) => {
                let value = self.env.evaluate_with_runtime(expr, self)?;
                println!("[OUTPUT] {}", &value);
                Ok(())
            },
            Statement::Return(expr) => {
                let value = self.env.evaluate_with_runtime(expr, self)?;
                self.env.set_var("retorno".to_string(), value.clone());
                println!("[INFO] Returning value: {:?}", value);
  
                return Err(RuntimeError::Return(value).into());
            },
            Statement::Read(name) => {
                let mut input = String::new();
                io::stdin().read_line(&mut input)
                    .map_err(|e| RuntimeError::InvalidOperation(format!("Failed to read input: {}", e)))?;
                let mut env = self.env.variables.lock()
                    .map_err(|e| RuntimeError::LockError(format!("Failed to lock variables: {}", e)))?;
                env.insert(name.to_string(), Value::Text(input.trim().to_string()));
                Ok(())
            },
            Statement::ArrayDeclaration { name, elements } => {
                self.env.with_locked_vars(|env| {
                    env.insert(name.to_string(), Value::Array(elements.clone()));
                });
                Ok(())
            },
            Statement::StructDeclaration { name, fields } => {
                let mut structs = self.env.structs.lock()
                    .map_err(|e| RuntimeError::LockError(format!("Failed to lock structs: {}", e)))?;
                structs.insert(name.to_string(), fields.clone());
                info!("Struct '{}' defined with {} fields", name, fields.len());
                Ok(())
            },
            Statement::Loop { variable, start, end, body } => {
                for i in *start..=*end {
                    self.env.with_locked_vars(|env| {
                        env.insert(variable.to_string(), Value::Number(i));
                    });
                    self.exec_block_impl(body).await?;
                }
                Ok(())
            },
            Statement::ForLoop { init_var, init_value, condition, increment_var, increment_expr, body } => {
                let init_result = self.env.evaluate(init_value)?;
                self.env.set_var(init_var.to_string(), init_result);

                loop {
                    let cond_result = self.env.evaluate(condition)?;

                    let continue_loop = match cond_result {
                        Value::Number(n) => n != 0,
                        _ => return Err(RuntimeError::TypeError("Loop condition must return a number".to_string()).into()),
                    };

                    if !continue_loop {
                        break;
                    }

                    self.exec_block_impl(body).await?;

                    let incr_result = self.env.evaluate(increment_expr)?;
                    self.env.set_var(increment_var.to_string(), incr_result);
                }
                Ok(())
            },
            Statement::ForInLoop { iterator_var, array_name, body } => {
                let array = self.env.get_var(array_name)?;
                
                match array {
                    Value::Array(elements) => {
                        for element in elements {
                            self.env.set_var(iterator_var.clone(), element.clone());
                            self.exec_block_impl(body).await?;
                        }
                        Ok(())
                    },
                    _ => Err(RuntimeError::TypeError(format!("Cannot iterate over non-array value: {}", array_name)).into()),
                }
            },
            Statement::LoopBlock { body } => {
                self.exec_block_impl(body).await?;
                Ok(())
            },
            Statement::SwitchStatement { value, cases, default } => {
                let val = self.env.evaluate(value)?;
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
                let mut threads = self.async_manager.threads.lock()
                    .map_err(|e| RuntimeError::LockError(format!("Failed to lock threads: {}", e)))?;
                if let Some(handle) = threads.remove(name) {
                    handle.await.map_err(|e| RuntimeError::AsyncError(format!("Error waiting for thread: {}", e)))??;
                }
                Ok(())
            },
            _ => Ok(())
        }
    }

    pub async fn exec_block(&self, statements: &[Statement]) -> RuntimeResult<()> 
    where
        Self: Send + Sync,
    {
        self.exec_block_impl(statements).await
    }

    pub fn call_function_expr(&self, name: &str, args: Vec<Value>) -> RuntimeResult<Value> {
        println!("[DEBUG] Iniciando call_function_expr para '{}'", name);
        match self.env.execute_sync_function(name, args.clone()) {
            Ok(result) => return Ok(result),
            Err(e) => {
                println!("[DEBUG] Erro ao executar função assíncrona: {:?}", e);
                let (param_names, body) = self.env.get_function(name)?;

                let mut call_vars = self.env.variables.lock()
                    .map_err(|e| RuntimeError::LockError(format!("Failed to lock variables: {}", e)))?
                    .clone();

                if args.len() != param_names.len() {
                    warn!("Number of arguments ({}) different from number of parameters ({}) for function '{}'", 
                            args.len(), param_names.len(), name);

                    let args_to_use = if args.len() > param_names.len() {
                        args[0..param_names.len()].to_vec()
                    } else {
                        args.clone()
                    };
                    
                    for (i, param) in param_names.iter().enumerate() {
                        if i < args_to_use.len() {
                            call_vars.insert(param.clone(), args_to_use[i].clone());
                        } else {
                            call_vars.insert(param.clone(), Value::Number(0));
                        }
                    }
                } else {
                    for (param, arg) in param_names.iter().zip(args.iter()) {
                        call_vars.insert(param.clone(), arg.clone());
                    }
                }
                
                let function_env = Environment {
                    variables: Arc::new(Mutex::new(call_vars)),
                    functions: self.env.functions.clone(),
                    structs: self.env.structs.clone(),
                };
                
                let vars_arc = function_env.variables.clone();
                
                let function_runtime = MurlocRuntime {
                    env: function_env,
                    async_manager: AsyncManager::new(),
                    recursion_depth: self.recursion_depth.clone(),
                    max_recursion_depth: self.max_recursion_depth,
                    runtime: self.runtime.clone(),
                };

                let is_async = if let Some(first_stmt) = body.first() {
                    matches!(first_stmt, Statement::AsyncFunction { .. })
                } else {
                    false
                };

                let result = if is_async {
                    std::thread::spawn(move || {
                        function_runtime.runtime.block_on(function_runtime.exec_block_impl(&body))
                    }).join().unwrap()
                } else {
                    self.runtime.block_on(function_runtime.exec_block_impl(&body))
                };
                
                match result {
                    Ok(()) => {
                        let vars = vars_arc.lock().unwrap();
                        if let Some(return_val) = vars.get("retorno") {
                            Ok(return_val.clone())
                        } else {
                            Ok(Value::Number(0))
                        }
                    },
                    Err(e) => Err(e)
                }
            }
        }
    }

    async fn call_function_impl(&self, name: &str, local_vars: HashMap<String, Value>, body: &[Statement]) -> RuntimeResult<()> 
    where
        Self: Send + Sync,
    {
        let current_vars = self.env.variables.lock().unwrap().clone();
        
        for (param, value) in local_vars.iter() {
            self.env.set_var(param.clone(), value.clone());
        }
        
        let result = self.exec_block_impl(body).await;
        
        let retorno = if let Ok(ret) = self.env.get_var("return") {
            Some(ret)
        } else {
            None
        };
        
        let mut vars = self.env.variables.lock().unwrap();
        *vars = current_vars;
        
        if let Some(ret) = retorno {
            vars.insert("return".to_string(), ret.clone());
            println!("[INFO] Function '{}' returned {:?}", name, ret);
        }
        
        match result {
            Err(ParseError::RuntimeError(RuntimeError::Return(_))) => Ok(()),
            Err(e) => Err(e),
            Ok(()) => Ok(())
        }
    }
    
    pub fn create_thread_runtime(&self, vars_copy: HashMap<String, Value>, funcs_copy: HashMap<String, (Vec<String>, Vec<Statement>)>, structs_copy: HashMap<String, Vec<(String, Type)>>) -> MurlocRuntime {
        MurlocRuntime {
            env: Environment {
                variables: Arc::new(Mutex::new(vars_copy)),
                functions: Arc::new(Mutex::new(funcs_copy)),
                structs: Arc::new(Mutex::new(structs_copy)),
            },
            async_manager: AsyncManager::new(),
            recursion_depth: self.recursion_depth.clone(),
            max_recursion_depth: 500,
            runtime: self.runtime.clone(),
        }
    }

    pub fn wait_for_threads(&self, names: &[String]) -> RuntimeResult<()> {
        let mut handles = Vec::new();
        let names_cloned = names.to_vec();
        
        {
            let mut threads_map = self.async_manager.threads.lock()
                .map_err(|e| RuntimeError::LockError(format!("Failed to lock threads: {}", e)))?;
            for name in &names_cloned {
                if let Some(handle) = threads_map.remove(name) {
                    handles.push(handle);
                } else {
                    warn!("Thread '{}' not found for waiting", name);
                }
            }
        }
        
        if handles.is_empty() {
            warn!("No threads to wait for");
            return Ok(());
        }
        
        info!("Waiting for {} threads", handles.len());
        let runtime_clone = self.runtime.clone();
        
        let result = std::thread::spawn(move || {
            runtime_clone.block_on(async move {
                for handle in handles {
                    match handle.await {
                        Ok(result) => {
                            match result {
                                Ok(_) => info!("Thread completed successfully"),
                                Err(e) => error!("Thread completed with error: {:?}", e)
                            }
                        },
                        Err(e) => {
                            error!("Error waiting for thread: {}", e);
                            return Err(RuntimeError::AsyncError(e.to_string()).into());
                        }
                    }
                }
                Ok(())
            })
        }).join()
        .map_err(|e| RuntimeError::InvalidOperation(format!("Failed to join thread: {:?}", e)))?;
        
        info!("All threads completed");
        result
    }

    pub fn call_function_from_expression(&self, name: &str, args: Vec<Expression>) -> RuntimeResult<Value> {
        let (param_names, body) = self.env.get_function(name)?;
        
        let evaluated_args = args.iter()
            .map(|arg| self.env.evaluate(arg))
            .collect::<Result<Vec<Value>, ParseError>>()?;

        let mut function_env = self.env.variables.lock()
            .map_err(|e| RuntimeError::LockError(format!("Failed to lock variables: {}", e)))?
            .clone();

        for (param, arg) in param_names.iter().zip(evaluated_args.iter()) {
            function_env.insert(param.clone(), arg.clone());
        }

        let mut result = Value::Number(0);
        for stmt in body {
            match stmt {
                Statement::Return(expr) => {
                    result = self.env.evaluate(&expr)?;
                    break;
                },
                _ => continue,
            }
        }
        
        Ok(result)
    }
} 