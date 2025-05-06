use crate::ast::Value;
use crate::value_parser::ParseError;
use std::fmt;

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
    LexerError(String),
    LockError(String),
}

impl From<RuntimeError> for ParseError {
    fn from(err: RuntimeError) -> Self {
        match err {
            RuntimeError::Return(value) => ParseError::RuntimeError(RuntimeError::Return(value.clone())),
            RuntimeError::LexerError(msg) => ParseError::InvalidValue(format!("BLRGHH! Unreadable glyphs in the kelp scroll: {}", msg)),
            RuntimeError::VariableNotFound(name) => ParseError::InvalidValue(format!("Lost rune '{}' — perhaps eaten by deep sea worms?", name)),
            RuntimeError::UndefinedVariable(name) => ParseError::InvalidValue(format!("'{}' floats undefined in the tide. Summon it, fool!", name)),
            RuntimeError::UndefinedFunction(name) => ParseError::InvalidValue(format!("Spell '{}' not found in the sacred bubble texts!", name)),
            RuntimeError::TypeError(msg) => ParseError::InvalidValue(format!("Glub! Type spirits are angry: {}", msg)),
            RuntimeError::DivisionByZero => ParseError::InvalidValue("You dare divide by the abyss?! Void screams back!".to_string()),
            RuntimeError::InvalidOperation(msg) => ParseError::InvalidValue(format!("Forbidden dance of operations: {}", msg)),
            RuntimeError::AsyncError(msg) => ParseError::InvalidValue(format!("Temporal rift detected in async currents: {}", msg)),
            RuntimeError::IndexOutOfBounds(msg) => ParseError::InvalidValue(format!("You swam beyond the coral bounds! Index chaos: {}", msg)),
            RuntimeError::FileError(msg) => ParseError::InvalidValue(format!("Scroll drowned! File error in the shell archive: {}", msg)),
            RuntimeError::LockError(msg) => ParseError::InvalidValue(format!("Lock error: {}", msg)),
        }
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeError::Return(value) => write!(f, "GLLBLRK! Ritual interrupted. Offering returned: {:?}", value),
            RuntimeError::LexerError(msg) => write!(f, "BLRGHH! Unreadable glyphs in the kelp scroll: {}", msg),
            RuntimeError::VariableNotFound(name) => write!(f, "Lost rune '{}' — perhaps eaten by deep sea worms?", name),
            RuntimeError::UndefinedVariable(name) => write!(f, "'{}' floats undefined in the tide. Summon it, fool!", name),
            RuntimeError::UndefinedFunction(name) => write!(f, "Spell '{}' not found in the sacred bubble texts!", name),
            RuntimeError::TypeError(msg) => write!(f, "Glub! Type spirits are angry: {}", msg),
            RuntimeError::DivisionByZero => write!(f, "You dare divide by the abyss?! Void screams back!"),
            RuntimeError::InvalidOperation(msg) => write!(f, "Forbidden dance of operations: {}", msg),
            RuntimeError::AsyncError(msg) => write!(f, "Temporal rift detected in async currents: {}", msg),
            RuntimeError::IndexOutOfBounds(msg) => write!(f, "You swam beyond the coral bounds! Index chaos: {}", msg),
            RuntimeError::FileError(msg) => write!(f, "Scroll drowned! File error in the shell archive: {}", msg),
            RuntimeError::LockError(msg) => write!(f, "Lock error: {}", msg),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ReturnValue {
    None,
    Value(Value),
}

pub type RuntimeResult<T> = Result<T, ParseError>; 