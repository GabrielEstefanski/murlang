use crate::ast::{Expression, BinaryOperator, ComparisonOperator, LogicalOperator, Value};
use crate::lexer::Token;
use crate::value_parser::ParseError;

pub fn parse_expression(tokens: &[Token], i: &mut usize) -> Result<Expression, ParseError> {
    parse_logical_or(tokens, i)
}

fn parse_logical_or(tokens: &[Token], i: &mut usize) -> Result<Expression, ParseError> {
    let mut expr = parse_logical_and(tokens, i)?;

    while *i < tokens.len() {
        match &tokens[*i] {
            Token::Or => {
                *i += 1;
                let right = parse_logical_and(tokens, i)?;
                expr = Expression::LogicalOp {
                    left: Box::new(expr),
                    right: Some(Box::new(right)),
                    op: LogicalOperator::Or,
                };
            }
            _ => break,
        }
    }

    Ok(expr)
}

fn parse_logical_and(tokens: &[Token], i: &mut usize) -> Result<Expression, ParseError> {
    let mut expr = parse_comparison(tokens, i)?;

    while *i < tokens.len() {
        match &tokens[*i] {
            Token::And => {
                *i += 1;
                let right = parse_comparison(tokens, i)?;
                expr = Expression::LogicalOp {
                    left: Box::new(expr),
                    right: Some(Box::new(right)),
                    op: LogicalOperator::And,
                };
            }
            _ => break,
        }
    }

    Ok(expr)
}

fn parse_comparison(tokens: &[Token], i: &mut usize) -> Result<Expression, ParseError> {
    let mut expr = parse_addition(tokens, i)?;

    while *i < tokens.len() {
        let op = match &tokens[*i] {
            Token::EqualsEquals => ComparisonOperator::Equals,
            Token::NotEquals => ComparisonOperator::NotEquals,
            Token::LessThan => ComparisonOperator::LessThan,
            Token::GreaterThan => ComparisonOperator::GreaterThan,
            Token::LessThanOrEqual => ComparisonOperator::LessThanOrEqual,
            Token::GreaterThanOrEqual => ComparisonOperator::GreaterThanOrEqual,
            _ => break,
        };
        *i += 1;
        let right = parse_addition(tokens, i)?;
        expr = Expression::Comparison {
            left: Box::new(expr),
            right: Box::new(right),
            op,
        };
    }

    Ok(expr)
}

fn parse_addition(tokens: &[Token], i: &mut usize) -> Result<Expression, ParseError> {
    let mut expr = parse_multiplication(tokens, i)?;

    while *i < tokens.len() {
        let op = match &tokens[*i] {
            Token::Plus => BinaryOperator::Add,
            Token::Minus => BinaryOperator::Subtract,
            _ => break,
        };
        *i += 1;
        let right = parse_multiplication(tokens, i)?;
        expr = Expression::BinaryOp {
            left: Box::new(expr),
            right: Box::new(right),
            op,
        };
    }

    Ok(expr)
}

fn parse_multiplication(tokens: &[Token], i: &mut usize) -> Result<Expression, ParseError> {
    let mut expr = parse_unary(tokens, i)?;

    while *i < tokens.len() {
        let op = match &tokens[*i] {
            Token::Multiply => BinaryOperator::Multiply,
            Token::Divide => BinaryOperator::Divide,
            Token::Modulo => BinaryOperator::Modulo,
            _ => break,
        };
        *i += 1;
        let right = parse_unary(tokens, i)?;
        expr = Expression::BinaryOp {
            left: Box::new(expr),
            right: Box::new(right),
            op,
        };
    }

    Ok(expr)
}

fn parse_unary(tokens: &[Token], i: &mut usize) -> Result<Expression, ParseError> {
    match &tokens[*i] {
        Token::Not => {
            *i += 1;
            let expr = parse_primary(tokens, i)?;
            Ok(Expression::LogicalOp {
                left: Box::new(expr),
                right: None,
                op: LogicalOperator::Not,
            })
        }
        Token::Minus => {
            *i += 1;
            let expr = parse_primary(tokens, i)?;
            Ok(Expression::BinaryOp {
                left: Box::new(Expression::Literal(Value::Number(0))),
                right: Box::new(expr),
                op: BinaryOperator::Subtract,
            })
        }
        _ => parse_primary(tokens, i),
    }
}

fn parse_primary(tokens: &[Token], i: &mut usize) -> Result<Expression, ParseError> {
    if *i >= tokens.len() {
        return Err(ParseError::UnexpectedToken("Fim inesperado dos tokens".to_string()));
    }

    match &tokens[*i] {
        Token::Number(n) => {
            *i += 1;
            if let Ok(num) = n.parse::<i32>() {
                Ok(Expression::Literal(Value::Number(num)))
            } else if let Ok(num) = n.parse::<i64>() {
                Ok(Expression::Literal(Value::NumberI64(num)))
            } else if let Ok(num) = n.parse::<num_bigint::BigInt>() {
                Ok(Expression::Literal(Value::NumberBig(num)))
            } else {
                Err(ParseError::InvalidValue(format!("Número inválido: {}", n)))
            }
        },
        Token::StringLiteral(s) => {
            *i += 1;
            Ok(Expression::Literal(Value::Text(s.clone())))
        },
        Token::Identifier(name) => {
            *i += 1;
            if *i < tokens.len() {
                match &tokens[*i] {
                    Token::LeftBracket => {
                        *i += 1;
                        let index = parse_expression(tokens, i)?;
                        if *i < tokens.len() && matches!(&tokens[*i], Token::RightBracket) {
                            *i += 1;
                            Ok(Expression::ArrayAccess {
                                name: name.clone(),
                                index: Box::new(index),
                            })
                        } else {
                            Err(ParseError::MissingToken("Faltando ']' para fechar o acesso ao array".to_string()))
                        }
                    },
                    Token::Dot => {
                        *i += 1;
                        if *i >= tokens.len() {
                            return Err(ParseError::MissingToken("Faltando nome do campo após '.'".to_string()));
                        }
                        if let Token::Identifier(field) = &tokens[*i] {
                            *i += 1;
                            Ok(Expression::StructAccess {
                                name: name.clone(),
                                field: field.clone(),
                            })
                        } else {
                            Err(ParseError::InvalidValue(format!("Esperado identificador após '.', encontrado {:?}", tokens[*i])))
                        }
                    },
                    _ => Ok(Expression::Variable(name.clone())),
                }
            } else {
                Ok(Expression::Variable(name.clone()))
            }
        },
        Token::LeftParen => {
            *i += 1;
            let expr = parse_expression(tokens, i)?;
            if *i < tokens.len() && matches!(&tokens[*i], Token::RightParen) {
                *i += 1;
                Ok(expr)
            } else {
                Err(ParseError::MissingToken("Faltando ')' para fechar a expressão".to_string()))
            }
        },
        _ => Err(ParseError::InvalidValue(format!("Token inesperado: {:?}", tokens[*i]))),
    }
} 