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
        else_branch: Option<Box<Statement>>,
    },
    ForLoop {
        init_var: String,
        init_value: Expression,
        condition: Expression,
        increment_var: String,
        increment_expr: Expression,
        body: Vec<Statement>,
    },
    ForInLoop {
        iterator_var: String,
        array_name: String,
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
        parent_scope: Option<Vec<String>>,
    },
    CallFunction {
        name: String,
        args: Vec<Expression>,
    },
    Import {
        path: String,
        imports: Vec<ImportSpecifier>,
    },
    Export {
        name: String,
        is_default: bool,
    },
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
        parent_scope: Option<Vec<String>>,
    },
    Sync {
        name: String,
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
}

#[derive(Debug, Clone)]
pub enum Type {
    Number,
    Text,
    Array(Box<Type>),
    Struct(String),
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Type::Number => write!(f, "Number"),
            Type::Text => write!(f, "Text"),
            Type::Array(t) => write!(f, "Array<{}>", t),
            Type::Struct(name) => write!(f, "{}", name),
        }
    }
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
            Value::Struct(_, fields) => {
                let mut first = true;
                for (field, value) in fields {
                    if !first {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", field, value)?;
                    first = false;
                }
                Ok(())
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
                    Err(ParseError::InvalidValue(format!("Variable '{}' lost in the cosmic void", name)))
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
                        _ => Err(ParseError::InvalidValue("Invalid addition operation in the cosmic void".to_string())),
                    },
                    BinaryOperator::Subtract => match (&left_val, &right_val) {
                        (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a - b)),
                        _ => Err(ParseError::InvalidValue("Invalid subtraction operation in the cosmic void".to_string())),
                    },
                    BinaryOperator::Multiply => match (&left_val, &right_val) {
                        (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a * b)),
                        _ => Err(ParseError::InvalidValue("Invalid multiplication operation in the cosmic void".to_string())),
                    },
                    BinaryOperator::Divide => match (&left_val, &right_val) {
                        (Value::Number(a), Value::Number(b)) => {
                            if *b == 0 {
                                Err(ParseError::InvalidValue("Attempted to divide by the void".to_string()))
                            } else {
                                Ok(Value::Number(a / b))
                            }
                        },
                        _ => Err(ParseError::InvalidValue("Invalid division operation in the cosmic void".to_string())),
                    },
                    BinaryOperator::Modulo => match (&left_val, &right_val) {
                        (Value::Number(a), Value::Number(b)) => {
                            if *b == 0 {
                                Err(ParseError::InvalidValue("Attempted to modulo by the void".to_string()))
                            } else {
                                Ok(Value::Number(a % b))
                            }
                        },
                        _ => Err(ParseError::InvalidValue("Invalid modulo operation in the cosmic void".to_string())),
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
                        _ => return Err(ParseError::InvalidValue("Comparison < only works with numbers in the cosmic void".to_string())),
                    },
                    ComparisonOperator::GreaterThan => match (&left_val, &right_val) {
                        (Value::Number(a), Value::Number(b)) => a > b,
                        _ => return Err(ParseError::InvalidValue("Comparison > only works with numbers in the cosmic void".to_string())),
                    },
                    ComparisonOperator::LessThanOrEqual => match (&left_val, &right_val) {
                        (Value::Number(a), Value::Number(b)) => a <= b,
                        _ => return Err(ParseError::InvalidValue("Comparison <= only works with numbers in the cosmic void".to_string())),
                    },
                    ComparisonOperator::GreaterThanOrEqual => match (&left_val, &right_val) {
                        (Value::Number(a), Value::Number(b)) => a >= b,
                        _ => return Err(ParseError::InvalidValue("Comparison >= only works with numbers in the cosmic void".to_string())),
                    },
                };
                Ok(Value::Number(if result { 1 } else { 0 }))
            },
            Expression::LogicalOp { left, right, op } => {
                let left_val = left.eval(env)?;
                let left_bool = match &left_val {
                    Value::Number(n) => *n != 0,
                    _ => return Err(ParseError::InvalidValue("Logical operations require numeric operands in the cosmic void".to_string())),
                };
                
                match op {
                    LogicalOperator::Not => Ok(Value::Number(if left_bool { 0 } else { 1 })),
                    LogicalOperator::And => {
                        if !left_bool {
                            return Ok(Value::Number(0));
                        }
                        
                        let right = right.as_ref().ok_or_else(|| 
                            ParseError::InvalidValue("AND operator requires a right operand in the ritual".to_string())
                        )?;
                        
                        let right_val = right.eval(env)?;
                        let right_bool = match &right_val {
                            Value::Number(n) => *n != 0,
                            _ => return Err(ParseError::InvalidValue("Logical operations require numeric operands in the cosmic void".to_string())),
                        };
                        
                        Ok(Value::Number(if left_bool && right_bool { 1 } else { 0 }))
                    },
                    LogicalOperator::Or => {
                        if left_bool {
                            return Ok(Value::Number(1));
                        }
                        
                        let right = right.as_ref().ok_or_else(|| 
                            ParseError::InvalidValue("OR operator requires a right operand in the ritual".to_string())
                        )?;
                        
                        let right_val = right.eval(env)?;
                        let right_bool = match &right_val {
                            Value::Number(n) => *n != 0,
                            _ => return Err(ParseError::InvalidValue("Logical operations require numeric operands in the cosmic void".to_string())),
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
                                Err(ParseError::InvalidValue(format!("Array index {} out of bounds in the matrix for array '{}'", idx, name)))
                            } else {
                                Ok(arr[idx as usize].clone())
                            }
                        },
                        _ => Err(ParseError::InvalidValue("Array index must be a number in the cosmic void".to_string())),
                    }
                } else {
                    Err(ParseError::InvalidValue(format!("Array '{}' not found in the cosmic void", name)))
                }
            },
            Expression::StructAccess { name, field } => {
                if let Some(Value::Struct(_, fields)) = env.get(name) {
                    if let Some((_, value)) = fields.iter().find(|(f, _)| f == field) {
                        Ok(value.clone())
                    } else {
                        Err(ParseError::InvalidValue(format!("Field '{}' not found in struct '{}' in the matrix", field, name)))
                    }
                } else {
                    Err(ParseError::InvalidValue(format!("Struct '{}' not found in the cosmic void", name)))
                }
            },
            Expression::FunctionCall { name, args: _ } => {
                let function_name = name.clone();
                Err(ParseError::InvalidValue(format!(
                    "Function call '{}' cannot be evaluated directly in this context of the ritual",
                    function_name
                )))
            },
            Expression::Equals(_, _) => {
                Err(ParseError::InvalidValue("Equals is not an evaluable expression in the ritual".to_string()))
            },
            Expression::StructInstance { struct_name, fields } => {
                if let Some(Value::Struct(name, existing_fields)) = env.get(struct_name) {
                    let mut new_fields = existing_fields.clone();
                    for (field, value) in fields {
                        let field_value = value.eval(env)?;
                        new_fields.push((field.clone(), field_value));
                    }
                    Ok(Value::Struct(name.clone(), new_fields))
                } else {
                    Err(ParseError::InvalidValue(format!("Struct '{}' not found in the cosmic void", struct_name)))
                }
            },
            Expression::InOperator { left, right } => {
                let left_val = left.eval(env)?;
                let right_val = right.eval(env)?;
                
                match (&left_val, &right_val) {
                    (Value::Text(item), Value::Array(arr)) => {
                        Ok(Value::Number(if arr.contains(&Value::Text(item.clone())) { 1 } else { 0 }))
                    },
                    (Value::Number(item), Value::Array(arr)) => {
                        Ok(Value::Number(if arr.contains(&Value::Number(*item)) { 1 } else { 0 }))
                    },
                    _ => Err(ParseError::InvalidValue("Operador 'in' só pode ser usado com arrays no reino dos murlocs".to_string())),
                }
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
                _ => Err(ParseError::InvalidValue("Invalid types for Add in the cosmic void".to_string())),
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
    pub fn eval(&self, _env: &HashMap<String, Value>) -> Result<Value, ParseError> {
        match self {
            Statement::CallFunction { name, args: _ } => {
                Err(ParseError::InvalidValue(format!(
                    "Function call '{}' cannot be evaluated directly in this context of the ritual",
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
    StructInstance {
        struct_name: String,
        fields: Vec<(String, Expression)>,
    },
    FunctionCall {
        name: String,
        args: Vec<Expression>,
    },
    InOperator {
        left: Box<Expression>,
        right: Box<Expression>,
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
pub enum ImportSpecifier {
    Default(String),           // import x from 'y'
    Named(String, String),     // import { x as y } from 'z'
    Namespace(String),         // import * as x from 'y'
    Specific(String),          // import { x } from 'y'
}