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
            Token::Keyword(kw) if kw == "in" => {
                *i += 1;
                let right = parse_logical_and(tokens, i)?;
                expr = Expression::InOperator {
                    left: Box::new(expr),
                    right: Box::new(right),
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
            Token::Equal => ComparisonOperator::Equals,
            Token::NotEqual => ComparisonOperator::NotEquals,
            Token::LessThan => ComparisonOperator::LessThan,
            Token::GreaterThan => ComparisonOperator::GreaterThan,
            Token::LessEqual => ComparisonOperator::LessThanOrEqual,
            Token::GreaterEqual => ComparisonOperator::GreaterThanOrEqual,
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
        return Err(ParseError::UnexpectedToken(format!("Unexpected end of tokens in the cosmic void at position {}", i)))
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
                Err(ParseError::InvalidValue(format!("Invalid number in the cosmic void: {} at position {}", n, i)))
            }
        },
        Token::StringLiteral(s) => {
            *i += 1;
            Ok(Expression::Literal(Value::Text(s.clone())))
        },
        Token::Keyword(kw) => {
            *i += 1;
            match kw.as_str() {
                "call" => {
                    if *i >= tokens.len() {
                        return Err(ParseError::UnexpectedToken(format!("Unexpected end after 'grrrblbl' in the ritual at position {}", i)))
                    }
                    
                    let func_name = match &tokens[*i] {
                        Token::Identifier(name) => name.clone(),
                        tok => return Err(ParseError::UnexpectedToken(format!("Expected identifier after 'grrrblbl', found {:?} in the ritual at position {}", tok, i))),
                    };
                    *i += 1;
                    
                    let mut args = Vec::new();
                    
                    let has_parens = *i < tokens.len() && matches!(&tokens[*i], Token::LeftParen);
                    if has_parens {
                        *i += 1;
                    }
                    
                    while *i < tokens.len() {
                        if has_parens && matches!(&tokens[*i], Token::RightParen) {
                            *i += 1;
                            break;
                        }
                        
                        if !has_parens && (*i >= tokens.len() || matches!(&tokens[*i], Token::Keyword(_))) {
                            break;
                        }
                        
                        match &tokens[*i] {
                            Token::Identifier(var_name) => {
                                args.push(Expression::Variable(var_name.clone()));
                                *i += 1;
                            },
                            Token::Number(num) => {
                                if let Ok(n) = num.parse::<i32>() {
                                    args.push(Expression::Literal(Value::Number(n)));
                                } else if let Ok(n) = num.parse::<i64>() {
                                    args.push(Expression::Literal(Value::NumberI64(n)));
                                } else if let Ok(n) = num.parse::<num_bigint::BigInt>() {
                                    args.push(Expression::Literal(Value::NumberBig(n)));
                                } else {
                                    return Err(ParseError::InvalidValue(format!("Invalid number in the cosmic void: {} at position {}", num, i)));
                                }
                                *i += 1;
                            },
                            Token::StringLiteral(text) => {
                                args.push(Expression::Literal(Value::Text(text.clone())));
                                *i += 1;
                            },
                            Token::Comma => {
                                *i += 1;
                            },
                            _ => break,
                        }
                    }
                    
                    Ok(Expression::FunctionCall { name: func_name, args })
                }
                _ => Err(ParseError::UnexpectedToken(format!("Unexpected keyword in the cosmic void: {} at position {}", kw, i))),
            }
        },
        Token::Identifier(name) => {
            *i += 1;
            if *i < tokens.len() && matches!(&tokens[*i], Token::Dot) {
                *i += 1;
                let field_name = match &tokens[*i] {
                    Token::Identifier(field) => field.clone(),
                    _ => return Err(ParseError::UnexpectedToken(format!("Esperado nome do campo após '.', encontrado {:?}", tokens[*i]))),
                };
                *i += 1;
                Ok(Expression::StructAccess {
                    name: name.clone(),
                    field: field_name,
                })
            } else if *i < tokens.len() && matches!(&tokens[*i], Token::LeftBrace) {
                *i += 1;
                let mut fields = Vec::new();
                
                while *i < tokens.len() {
                    if matches!(&tokens[*i], Token::RightBrace) {
                        *i += 1;
                        break;
                    }
                    
                    let field_name = match &tokens[*i] {
                        Token::Identifier(name) => name.clone(),
                        _ => return Err(ParseError::UnexpectedToken(format!("Expected field name, found {:?} in the matrix at position {}", tokens[*i], i))),
                    };
                    *i += 1;
                    
                    expect_token_type(tokens, i, "Colon")?;
                    
                    let field_value = parse_expression(tokens, i)?;
                    fields.push((field_name, field_value));
                    
                    if matches!(&tokens[*i], Token::Comma) {
                        *i += 1;
                    } else if !matches!(&tokens[*i], Token::RightBrace) {
                        return Err(ParseError::UnexpectedToken(format!("Expected ',' or '}}', found {:?} in the matrix at position {}", tokens[*i], i)));
                    }
                }
                
                Ok(Expression::StructInstance {
                    struct_name: name.clone(),
                    fields,
                })
            } else {
                Ok(Expression::Variable(name.clone()))
            }
        },
        Token::LeftParen => {
            let start_pos = *i;
            *i += 1;
            let expr = parse_expression(tokens, i)?;
            if *i < tokens.len() && matches!(&tokens[*i], Token::RightParen) {
                *i += 1;
                Ok(expr)
            } else {
                Err(ParseError::MissingToken(format!("Missing ')' to close expression in the ritual at position {}", start_pos)))
            }
        },
        _ => Err(ParseError::InvalidValue(format!("Unexpected token in the cosmic void: {:?} at position {}", tokens[*i], i))),
    }
}

fn expect_token_type(tokens: &[Token], i: &mut usize, expected_type: &str) -> Result<(), ParseError> {
    if *i >= tokens.len() {
        return Err(ParseError::UnexpectedToken(format!("Unexpected end, expected {} in the ritual at position {}", expected_type, i)))
    }

    let matches = match (&tokens[*i], expected_type) {
        (Token::LeftParen, "LeftParen") => true,
        (Token::RightParen, "RightParen") => true,
        (Token::LeftBracket, "LeftBracket") => true,
        (Token::RightBracket, "RightBracket") => true,
        (Token::LeftBrace, "LeftBrace") => true,
        (Token::RightBrace, "RightBrace") => true,
        (Token::Semicolon, "Semicolon") => true,
        (Token::Colon, "Colon") => true,
        (Token::Comma, "Comma") => true,
        (Token::Assign, "Equals") => true,
        _ => false,
    };

    if matches {
        *i += 1;
        Ok(())
    } else {
        Err(ParseError::UnexpectedToken(format!("Expected {}, found {:?} in the ritual at position {}", expected_type, tokens[*i], i)))
    }
} 