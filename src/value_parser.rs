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
            ParseError::InvalidValue(msg) => write!(f, "Valor inválido: {}", msg),
            ParseError::InvalidType(msg) => write!(f, "Tipo inválido: {}", msg),
            ParseError::InvalidArrayType(msg) => write!(f, "Tipo de array inválido: {}", msg),
            ParseError::UnexpectedToken(msg) => write!(f, "Token inesperado: {}", msg),
            ParseError::MissingToken(msg) => write!(f, "Token faltando: {}", msg),
            ParseError::RuntimeError(err) => write!(f, "Erro de execução: {:?}", err),
        }
    }
}

impl std::error::Error for ParseError {}

pub fn parse_value(tokens: &[Token], i: &mut usize) -> Result<Value, ParseError> {
    if *i >= tokens.len() {
        return Err(ParseError::UnexpectedToken("Fim inesperado dos tokens".to_string()));
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
                Err(ParseError::InvalidValue(format!("Número inválido: {}", n)))
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
                    return Err(ParseError::UnexpectedToken(format!("Esperado ',' ou ']', encontrado {:?}", tokens[*i])));
                }
            }
            if *i >= tokens.len() {
                return Err(ParseError::MissingToken("Faltando ']' para fechar o array".to_string()));
            }
            *i += 1;
            Ok(Value::Array(elements))
        },
        Token::Identifier(_) => {
            Err(ParseError::InvalidValue("Identificador deve ser processado pelo expression_parser".to_string()))
        },
        _ => Err(ParseError::InvalidValue(format!("Token inesperado: {:?}", tokens[*i]))),
    }
}

pub fn parse_type(tokens: &[Token], i: &mut usize) -> Result<Type, ParseError> {
    if *i >= tokens.len() {
        return Err(ParseError::UnexpectedToken("Fim inesperado dos tokens".to_string()));
    }

    match &tokens[*i] {
        Token::Identifier(name) => {
            *i += 1;
            match name.as_str() {
                "number" => Ok(Type::Number),
                "text" => Ok(Type::Text),
                "array" => {
                    *i += 1;
                    if let Token::LessThan = &tokens[*i] {
                        *i += 1;
                        let inner_type = parse_type(tokens, i)?;
                        if *i < tokens.len() && matches!(&tokens[*i], Token::GreaterThan) {
                            *i += 1;
                            Ok(Type::Array(Box::new(inner_type)))
                        } else {
                            Err(ParseError::MissingToken("Faltando '>' após tipo do array".to_string()))
                        }
                    } else {
                        Err(ParseError::MissingToken("Faltando '<' após 'array'".to_string()))
                    }
                },
                _ => Ok(Type::Struct(name.clone())),
            }
        },
        _ => Err(ParseError::InvalidType(format!("Token inesperado: {:?}", tokens[*i]))),
    }
}
