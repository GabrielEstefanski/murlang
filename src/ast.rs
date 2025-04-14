use std::fmt;
use num_bigint::BigInt;
use std::collections::HashMap;
use crate::ParseError;

#[derive(Debug, Clone)]
pub enum Statement {
    VarDeclaration(String, Value),
    VarDeclarationExpr(String, Expression),
    Assignment(String, Expression),
    Expr(Expression),
    IfStatement {
        condition: Expression,
        body: Vec<Statement>,
    },
    ForLoop {
        init_var: String,
        init_value: Expression,
        condition: Expression,
        increment_var: String,
        increment_expr: Expression,
        body: Vec<Statement>,
    },
    Loop {
        variable: String,
        start: i32,
        end: i32,
        body: Vec<Statement>,
    },
    LoopBlock {
        body: Vec<Statement>,
    },
    WhileLoop {
        condition: Expression,
        body: Vec<Statement>,
    },
    SwitchStatement {
        value: Expression,
        cases: Vec<(Value, Vec<Statement>)>,
        default: Option<Vec<Statement>>,
    },
    Break,
    Continue,
    Return(Expression),
    Print(Expression),
    Read(String),
    Function {
        name: String,
        args: Vec<String>,
        body: Vec<Statement>,
    },
    CallFunction {
        name: String,
        args: Vec<Expression>,
    },
    Import(String),
    ArrayDeclaration {
        name: String,
        elements: Vec<Value>,
    },
    StructDeclaration {
        name: String,
        fields: Vec<(String, Type)>,
    },
    Spawn {
        body: Vec<Statement>,
        thread_name: Option<String>,
    },
    SpawnAsync {
        future: Box<Statement>,
        thread_name: Option<String>,
    },
    Await {
        future: Box<Statement>,
    },
    ThreadPool {
        size: Expression,
        tasks: Vec<Statement>,
    },
    Wait {
        thread_names: Vec<String>,
    },
    AsyncFunction {
        name: String,
        args: Vec<String>,
        body: Vec<Statement>,
    },
    Sync {
        name: String,
    },
    FishArray {
        name: String,
        elements: Vec<Value>,
        operation: FishOperation,
    },
    BubbleString {
        name: String,
        value: String,
        format: BubbleFormat,
    },
    TideFlow {
        source: Expression,
        destination: Expression,
        operation: TideOperation,
    },
    ShellStruct {
        name: String,
        fields: Vec<(String, Type)>,
        protection: ProtectionLevel,
    },
    WaveString {
        name: String,
        parts: Vec<Expression>,
        operation: WaveOperation,
    },
    WhenStatement {
        condition: Expression,
        body: Vec<Statement>,
        alternatives: Vec<(Expression, Vec<Statement>)>,
    },
    CatchBlock {
        try_block: Vec<Statement>,
        catch_blocks: Vec<(String, Vec<Statement>)>,
    },
    SwimLoop {
        variable: String,
        collection: Expression,
        body: Vec<Statement>,
    },
    SchoolBlock {
        name: String,
        members: Vec<Statement>,
    },
    SplashIO {
        operation: SplashOperation,
        target: Expression,
    },
    CurrentFlow {
        source: Expression,
        operation: CurrentOperation,
    },
    PearlValue {
        name: String,
        value: Expression,
    },
    CoralStructure {
        name: String,
        elements: Vec<(String, Type)>,
    },
}

#[derive(Debug, Clone)]
pub enum Type {
    Number,
    Text,
    Array(Box<Type>),
    Struct(String),
}

#[derive(Debug, Clone)]
pub enum Value {
    Number(i32),
    NumberI64(i64),
    NumberBig(BigInt),
    Text(String),
    Array(Vec<Value>),
    Struct(String, Vec<(String, Value)>),
    Future(Box<Statement>),
    Thread(String),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::NumberI64(n) => write!(f, "{}", n),
            Value::NumberBig(n) => write!(f, "{}", n),
            Value::Text(s) => write!(f, "{}", s),
            Value::Array(arr) => {
                write!(f, "[")?;
                for (i, value) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", value)?;
                }
                write!(f, "]")
            },
            Value::Struct(name, fields) => {
                write!(f, "{} {{", name)?;
                for (i, (field, value)) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", field, value)?;
                }
                write!(f, "}}")
            },
            Value::Future(_) => write!(f, "<future>"),
            Value::Thread(name) => write!(f, "<thread:{}>", name),
        }
    }
}

impl Expression {
    pub fn eval(&self, env: &HashMap<String, Value>) -> Result<Value, ParseError> {
        match self {
            Expression::Literal(value) => Ok(value.clone()),
            Expression::Variable(name) => {
                if let Some(value) = env.get(name) {
                    Ok(value.clone())
                } else {
                    Err(ParseError::InvalidValue(format!("Variável não encontrada: {}", name)))
                }
            },
            Expression::BinaryOp { left, right, op } => {
                let left_val = left.eval(env)?;
                let right_val = right.eval(env)?;
                match op {
                    BinaryOperator::Add => match (&left_val, &right_val) {
                        (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
                        (Value::Text(a), Value::Text(b)) => Ok(Value::Text(format!("{}{}", a, b))),
                        (Value::Text(a), Value::Number(b)) => Ok(Value::Text(format!("{}{}", a, b))),
                        (Value::Number(a), Value::Text(b)) => Ok(Value::Text(format!("{}{}", a, b))),
                        _ => Err(ParseError::InvalidValue("Operação de adição inválida".to_string())),
                    },
                    BinaryOperator::Subtract => match (&left_val, &right_val) {
                        (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a - b)),
                        _ => Err(ParseError::InvalidValue("Operação de subtração inválida".to_string())),
                    },
                    BinaryOperator::Multiply => match (&left_val, &right_val) {
                        (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a * b)),
                        _ => Err(ParseError::InvalidValue("Operação de multiplicação inválida".to_string())),
                    },
                    BinaryOperator::Divide => match (&left_val, &right_val) {
                        (Value::Number(a), Value::Number(b)) => {
                            if *b == 0 {
                                Err(ParseError::InvalidValue("Divisão por zero".to_string()))
                            } else {
                                Ok(Value::Number(a / b))
                            }
                        },
                        _ => Err(ParseError::InvalidValue("Operação de divisão inválida".to_string())),
                    },
                    BinaryOperator::Modulo => match (&left_val, &right_val) {
                        (Value::Number(a), Value::Number(b)) => {
                            if *b == 0 {
                                Err(ParseError::InvalidValue("Módulo por zero".to_string()))
                            } else {
                                Ok(Value::Number(a % b))
                            }
                        },
                        _ => Err(ParseError::InvalidValue("Operação de módulo inválida".to_string())),
                    },
                }
            },
            Expression::Comparison { left, right, op } => {
                let left_val = left.eval(env)?;
                let right_val = right.eval(env)?;
                let result = match op {
                    ComparisonOperator::Equals => left_val == right_val,
                    ComparisonOperator::NotEquals => left_val != right_val,
                    ComparisonOperator::LessThan => match (&left_val, &right_val) {
                        (Value::Number(a), Value::Number(b)) => a < b,
                        _ => return Err(ParseError::InvalidValue("Comparação < só funciona com números".to_string())),
                    },
                    ComparisonOperator::GreaterThan => match (&left_val, &right_val) {
                        (Value::Number(a), Value::Number(b)) => a > b,
                        _ => return Err(ParseError::InvalidValue("Comparação > só funciona com números".to_string())),
                    },
                    ComparisonOperator::LessThanOrEqual => match (&left_val, &right_val) {
                        (Value::Number(a), Value::Number(b)) => a <= b,
                        _ => return Err(ParseError::InvalidValue("Comparação <= só funciona com números".to_string())),
                    },
                    ComparisonOperator::GreaterThanOrEqual => match (&left_val, &right_val) {
                        (Value::Number(a), Value::Number(b)) => a >= b,
                        _ => return Err(ParseError::InvalidValue("Comparação >= só funciona com números".to_string())),
                    },
                };
                Ok(Value::Number(if result { 1 } else { 0 }))
            },
            Expression::LogicalOp { left, right, op } => {
                let left_val = left.eval(env)?;
                let left_bool = match &left_val {
                    Value::Number(n) => *n != 0,
                    _ => return Err(ParseError::InvalidValue("Operações lógicas precisam de operandos numéricos".to_string())),
                };
                
                match op {
                    LogicalOperator::Not => Ok(Value::Number(if left_bool { 0 } else { 1 })),
                    LogicalOperator::And => {
                        if !left_bool {
                            return Ok(Value::Number(0));
                        }
                        
                        let right = right.as_ref().ok_or_else(|| 
                            ParseError::InvalidValue("Operador AND requer um operando direito".to_string())
                        )?;
                        
                        let right_val = right.eval(env)?;
                        let right_bool = match &right_val {
                            Value::Number(n) => *n != 0,
                            _ => return Err(ParseError::InvalidValue("Operações lógicas precisam de operandos numéricos".to_string())),
                        };
                        
                        Ok(Value::Number(if left_bool && right_bool { 1 } else { 0 }))
                    },
                    LogicalOperator::Or => {
                        if left_bool {
                            return Ok(Value::Number(1));
                        }
                        
                        let right = right.as_ref().ok_or_else(|| 
                            ParseError::InvalidValue("Operador OR requer um operando direito".to_string())
                        )?;
                        
                        let right_val = right.eval(env)?;
                        let right_bool = match &right_val {
                            Value::Number(n) => *n != 0,
                            _ => return Err(ParseError::InvalidValue("Operações lógicas precisam de operandos numéricos".to_string())),
                        };
                        
                        Ok(Value::Number(if left_bool || right_bool { 1 } else { 0 }))
                    },
                }
            },
            Expression::ArrayAccess { name, index } => {
                if let Some(Value::Array(arr)) = env.get(name) {
                    let idx_val = index.eval(env)?;
                    match idx_val {
                        Value::Number(idx) => {
                            if idx < 0 || idx as usize >= arr.len() {
                                Err(ParseError::InvalidValue(format!("Índice {} fora dos limites do array {}", idx, name)))
                            } else {
                                Ok(arr[idx as usize].clone())
                            }
                        },
                        _ => Err(ParseError::InvalidValue("Índice de array deve ser um número".to_string())),
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
                        Err(ParseError::InvalidValue(format!("Campo '{}' não encontrado na estrutura '{}'", field, name)))
                    }
                } else {
                    Err(ParseError::InvalidValue(format!("Estrutura não encontrada: {}", name)))
                }
            },
            Expression::FunctionCall { name, args } => {
                let function_name = name.clone();
                Err(ParseError::InvalidValue(format!(
                    "Chamada de função '{}' não pode ser avaliada diretamente neste contexto",
                    function_name
                )))
            },
            Expression::Equals(_, _) => {
                Err(ParseError::InvalidValue("Equals não é uma expressão avaliável".to_string()))
            },
        }
    }
}

impl BinaryOperator {
    pub fn apply(&self, left: &Value, right: &Value) -> Result<Value, ParseError> {
        match self {
            BinaryOperator::Add => match (left, right) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
                (Value::Text(a), Value::Text(b)) => Ok(Value::Text(a.clone() + b)),
                _ => Err(ParseError::InvalidValue("Invalid types for Add".to_string())),
            },
            _ => unimplemented!(),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::Text(a), Value::Text(b)) => a == b,
            (Value::Array(a), Value::Array(b)) => a == b,
            (Value::Struct(_, a), Value::Struct(_, b)) => a == b,
            (Value::Thread(a), Value::Thread(b)) => a == b,
            _ => false,
        }
    }
}

impl Statement {
    pub fn eval(&self, env: &HashMap<String, Value>) -> Result<Value, ParseError> {
        match self {
            Statement::CallFunction { name, args } => {
                Err(ParseError::InvalidValue(format!(
                    "Chamada de função '{}' não pode ser avaliada diretamente neste contexto",
                    name
                )))
            },
            _ => unimplemented!(),
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => a.partial_cmp(b),
            (Value::Text(a), Value::Text(b)) => a.partial_cmp(b),
            (Value::Thread(a), Value::Thread(b)) => a.partial_cmp(b),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Expression {
    Equals(String, i32),
    BinaryOp {
        left: Box<Expression>,
        right: Box<Expression>,
        op: BinaryOperator,
    },
    Comparison {
        left: Box<Expression>,
        right: Box<Expression>,
        op: ComparisonOperator,
    },
    LogicalOp {
        left: Box<Expression>,
        right: Option<Box<Expression>>,
        op: LogicalOperator,
    },
    Literal(Value),
    Variable(String),
    ArrayAccess {
        name: String,
        index: Box<Expression>,
    },
    StructAccess {
        name: String,
        field: String,
    },
    FunctionCall {
        name: String,
        args: Vec<Expression>,
    },
}

#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
}

#[derive(Debug, Clone)]
pub enum ComparisonOperator {
    Equals,
    NotEquals,
    LessThan,
    GreaterThan,
    LessThanOrEqual,
    GreaterThanOrEqual,
}

#[derive(Debug, Clone)]
pub enum LogicalOperator {
    And,
    Or,
    Not,
}

#[derive(Debug, Clone)]
pub enum FishOperation {
    Add,
    Remove,
    Find,
    Sort,
}

#[derive(Debug, Clone)]
pub enum BubbleFormat {
    Normal,
    Uppercase,
    Lowercase,
    Reversed,
}

#[derive(Debug, Clone)]
pub enum TideOperation {
    Copy,
    Move,
    Transform,
}

#[derive(Debug, Clone)]
pub enum ProtectionLevel {
    Public,
    Protected,
    Private,
}

#[derive(Debug, Clone)]
pub enum WaveOperation {
    Concat,
    Split,
    Join,
}

#[derive(Debug, Clone)]
pub enum SplashOperation {
    Read,
    Write,
    Append,
}

#[derive(Debug, Clone)]
pub enum CurrentOperation {
    Map,
    Filter,
    Reduce,
}