use crate::ast::{Value, Type};
use crate::lexer::Token;
use num_bigint::BigInt;

#[derive(Debug)]
pub enum ParseError {
    InvalidValue(String),
    InvalidType(String),
    InvalidArrayType(String),
    UnexpectedToken(String),
    MissingToken(String),
    RuntimeError(crate::interpreter::RuntimeError),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::InvalidValue(msg) => write!(f, "Invalid value in the cosmic void: {}", msg),
            ParseError::InvalidType(msg) => write!(f, "Type mismatch in the astral plane: {}", msg),
            ParseError::InvalidArrayType(msg) => write!(f, "Array type violation in the matrix: {}", msg),
            ParseError::UnexpectedToken(msg) => write!(f, "Unexpected token in the codex: {}", msg),
            ParseError::MissingToken(msg) => write!(f, "Missing token in the ritual: {}", msg),
            ParseError::RuntimeError(err) => write!(f, "Runtime anomaly detected: {:?}", err),
        }
    }
}

impl std::error::Error for ParseError {}

pub fn parse_value(tokens: &[Token], i: &mut usize) -> Result<Value, ParseError> {
    if *i >= tokens.len() {
        return Err(ParseError::UnexpectedToken("Unexpected end of token stream".to_string()));
    }

    match &tokens[*i] {
        Token::Number(n) => {
            *i += 1;
            if let Ok(n) = n.to_string().parse::<i32>() {
                Ok(Value::Number(n))
            } else if let Ok(n) = n.to_string().parse::<i64>() {
                Ok(Value::NumberI64(n))
            } else if let Ok(n) = n.to_string().parse::<BigInt>() {
                Ok(Value::NumberBig(n))
            } else {
                Err(ParseError::InvalidValue(format!("Invalid number format: {}", n)))
            }
        },
        Token::StringLiteral(s) => {
            *i += 1;
            Ok(Value::Text(s.clone()))
        },
        Token::LeftBracket => {
            *i += 1;
            let mut elements = Vec::new();
            while *i < tokens.len() && !matches!(&tokens[*i], Token::RightBracket) {
                elements.push(parse_value(tokens, i)?);
                if let Token::Comma = &tokens[*i] {
                    *i += 1;
                } else if !matches!(&tokens[*i], Token::RightBracket) {
                    return Err(ParseError::UnexpectedToken(format!("Expected ',' or ']', found {:?}", tokens[*i])));
                }
            }
            if *i >= tokens.len() {
                return Err(ParseError::MissingToken("Missing ']' to close the array".to_string()));
            }
            *i += 1;
            Ok(Value::Array(elements))
        },
        Token::Identifier(_) => {
            Err(ParseError::InvalidValue("Identifier must be processed by expression_parser".to_string()))
        },
        _ => Err(ParseError::InvalidValue(format!("Unexpected token in value context: {:?}", tokens[*i]))),
    }
}

pub fn parse_type(tokens: &[Token], i: &mut usize) -> Result<Type, ParseError> {
    if *i >= tokens.len() {
        return Err(ParseError::UnexpectedToken("Unexpected end of token stream".to_string()));
    }

    match &tokens[*i] {
        Token::Keyword(kw) => {
            *i += 1;
            match kw.to_lowercase().as_str() {
                "number" => Ok(Type::Number),
                "text" => Ok(Type::Text),
                _ => Err(ParseError::InvalidType(format!("Invalid type keyword: {}", kw))),
            }
        },
        Token::Identifier(name) => {
            *i += 1;
            match name.to_lowercase().as_str() {
                "number" => Ok(Type::Number),
                "text" => Ok(Type::Text),
                _ => Ok(Type::Struct(name.clone())),
            }
        },
        _ => Err(ParseError::InvalidType(format!("Unexpected token in type context: {:?}", tokens[*i]))),
    }
}
