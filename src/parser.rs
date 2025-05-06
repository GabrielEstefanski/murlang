use crate::lexer::Token;
use crate::ast::{Statement, Value};
use crate::expression_parser::parse_expression;
use crate::value_parser::{parse_value, parse_type, ParseError};
use crate::Expression;

fn expect_identifier(tokens: &[Token], index: &mut usize) -> Result<String, ParseError> {
    match tokens.get(*index) {
        Some(Token::Identifier(name)) => {
            *index += 1;
            Ok(name.clone())
        },
        Some(tok) => Err(ParseError::UnexpectedToken(format!("Esperado identificador, encontrado {:?}", tok))),
        None => Err(ParseError::UnexpectedToken("Fim inesperado, esperado identificador".to_string())),
    }
}

fn expect_token_type(tokens: &[Token], index: &mut usize, expected_type: &str) -> Result<(), ParseError> {
    match tokens.get(*index) {
        Some(token) => {
            let matches = match (token, expected_type) {
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
                *index += 1;
                Ok(())
            } else {
                Err(ParseError::UnexpectedToken(format!("Esperado {}, encontrado {:?}", expected_type, token)))
            }
        },
        None => Err(ParseError::UnexpectedToken(format!("Fim inesperado, esperado {}", expected_type))),
    }
}

fn expect_keyword(tokens: &[Token], index: &mut usize, keyword: &str) -> Result<(), ParseError> {
    match tokens.get(*index) {
        Some(Token::Keyword(kw)) if kw == keyword => {
            *index += 1;
            Ok(())
        },
        Some(tok) => Err(ParseError::UnexpectedToken(format!("Esperado keyword '{}', encontrado {:?}", keyword, tok))),
        None => Err(ParseError::UnexpectedToken(format!("Fim inesperado, esperado keyword '{}'", keyword))),
    }
}

fn parse_function_args(tokens: &[Token], index: &mut usize) -> Result<Vec<Expression>, ParseError> {
                        let mut args = Vec::new();
                        
    expect_token_type(tokens, index, "LeftParen")?;
    
    while *index < tokens.len() {
        if matches!(&tokens[*index], Token::RightParen) {
            *index += 1;
                                break;
                            }
                            
        match &tokens[*index] {
            Token::Identifier(var_name) => {
                                    args.push(Expression::Variable(var_name.clone()));
                *index += 1;
                                },
            Token::Number(num) => {
                                    if let Ok(n) = num.parse::<i32>() {
                                        args.push(Expression::Literal(Value::Number(n)));
                                    } else if let Ok(n) = num.parse::<i64>() {
                                        args.push(Expression::Literal(Value::NumberI64(n)));
                                    } else if let Ok(n) = num.parse::<num_bigint::BigInt>() {
                                        args.push(Expression::Literal(Value::NumberBig(n)));
                                    } else {
                                        return Err(ParseError::InvalidValue(format!("Número inválido: {}", num)));
                                    }
                *index += 1;
                                },
            Token::StringLiteral(text) => {
                                    args.push(Expression::Literal(Value::Text(text.clone())));
                *index += 1;
                                },
            Token::Comma => {
                *index += 1;
                                },
            Token::Minus => {
                *index += 1;
                if *index < tokens.len() {
                    if let Token::Number(num) = &tokens[*index] {
                                        if let Ok(n) = num.parse::<i32>() {
                                            args.push(Expression::Literal(Value::Number(-n)));
                                        } else if let Ok(n) = num.parse::<i64>() {
                                            args.push(Expression::Literal(Value::NumberI64(-n)));
                                        } else {
                                            return Err(ParseError::InvalidValue(format!("Número inválido: -{}", num)));
                                        }
                        *index += 1;
                                    } else {
                                        return Err(ParseError::UnexpectedToken("Esperado número após sinal de menos".to_string()));
                                    }
                }
                                },
            tok => {
                return Err(ParseError::UnexpectedToken(format!("Token inesperado nos argumentos da função: {:?}", tok)));
            },
        }
    }
    
    if !(*index > 0 && matches!(&tokens[*index-1], Token::RightParen)) {
        return Err(ParseError::UnexpectedToken("Faltando parêntese de fechamento após argumentos da função".to_string()));
    }
    
    Ok(args)
}

fn parse_function_parameters(tokens: &[Token], index: &mut usize) -> Result<Vec<String>, ParseError> {
    let mut params = Vec::new();
    
    expect_token_type(tokens, index, "LeftParen")?;
    
    while *index < tokens.len() {
        match &tokens[*index] {
            Token::Identifier(arg) => {
                params.push(arg.clone());
                *index += 1;
            }
            Token::RightParen => {
                *index += 1;
                break;
            }
            Token::Comma => {
                *index += 1;
            }
            tok => {
                return Err(ParseError::UnexpectedToken(format!("Token inesperado nos parâmetros da função: {:?}", tok)));
                            }
                        }
    }
    
    Ok(params)
}

fn parse_function_or_async_function(
    tokens: &[Token], 
    index: &mut usize,
    is_async: bool,
    scope_stack: &mut Vec<String>
) -> Result<Statement, ParseError> {
    let name = expect_identifier(tokens, index)?;
    let args = parse_function_parameters(tokens, index)?;
    
    expect_keyword(tokens, index, "begin")?;
    
    scope_stack.push(name.clone());
    let body = parse_block(tokens, index, Some(scope_stack))?;
    scope_stack.pop();
    
    expect_keyword(tokens, index, "end")?;
    
    let parent_scope = if scope_stack.is_empty() { 
        None 
    } else { 
        Some(scope_stack.clone()) 
    };
    
    if is_async {
        Ok(Statement::AsyncFunction { name, args, body, parent_scope })
    } else {
        Ok(Statement::Function { name, args, body, parent_scope })
    }
}

pub fn parse(tokens: Vec<Token>) -> Result<Vec<Statement>, ParseError> {
    let mut stmts = Vec::new();
    let mut i = 0;
    let mut scope_stack = Vec::new();
 
    while i < tokens.len() {
        match &tokens[i] {
            Token::Keyword(kw) if kw == "var" => {
                i += 1;
                let name = expect_identifier(&tokens, &mut i)?;
                expect_token_type(&tokens, &mut i, "Equals")?;
                
                if matches!(tokens.get(i), Some(Token::Keyword(kw)) if kw == "async") {
                    i += 1;
                    
                    if i < tokens.len() && matches!(tokens.get(i), Some(Token::Keyword(kw)) if kw == "call") {
                        i += 1;
                        
                        let func_name = expect_identifier(&tokens, &mut i)?;
                        let args = parse_function_args(&tokens, &mut i)?;
                        
                        let call_stmt = Statement::CallFunction { name: func_name, args };
                        let future_stmt = Statement::SpawnAsync { future: Box::new(call_stmt), thread_name: Some(name.clone()) };
                        stmts.push(future_stmt);
                        continue;
                    }
                }
                
                let expr = parse_expression(&tokens, &mut i)?;
                stmts.push(Statement::VarDeclarationExpr(name, expr));
            }

            Token::Identifier(name) => {
                let var_name = name.clone();
                i += 1;
                
                if i < tokens.len() && matches!(tokens.get(i), Some(Token::Assign)) {
                    i += 1;
                    let expr = parse_expression(&tokens, &mut i)?;
                    stmts.push(Statement::Assignment(var_name, expr));
                } else {
                    continue;
                }
            }

            Token::Keyword(kw) if kw == "if" => {
                i += 1;
                let condition = parse_expression(&tokens, &mut i)?;
                
                expect_keyword(&tokens, &mut i, "begin")?;
                let body = parse_block(&tokens, &mut i, Some(&scope_stack))?;
                expect_keyword(&tokens, &mut i, "end")?;

                stmts.push(Statement::IfStatement { condition, body });
            }

            Token::Keyword(kw) if kw == "for" => {
                i += 1;
            
                let mut has_equals = false;
                let mut lookahead = i;
                
                while lookahead < tokens.len() && !matches!(&tokens[lookahead], Token::Semicolon) {
                    if matches!(&tokens[lookahead], Token::Assign) {
                        has_equals = true;
                        break;
                    }
                    lookahead += 1;
                }
                
                if has_equals {
                    let init_var = expect_identifier(&tokens, &mut i)?;
                    expect_token_type(&tokens, &mut i, "Equals")?;
                    
                    let mut expr_index = i;
                    let init_value = parse_expression(&tokens, &mut expr_index)?;
                    i = expr_index;
                    
                    expect_token_type(&tokens, &mut i, "Semicolon")?;
                    
                    expr_index = i;
                    let condition = parse_expression(&tokens, &mut expr_index)?;
                    i = expr_index;
                    
                    expect_token_type(&tokens, &mut i, "Semicolon")?;
            
                    let increment_var = expect_identifier(&tokens, &mut i)?;
                    expect_token_type(&tokens, &mut i, "Equals")?;
                    
                    expr_index = i;
                    let increment_expr = parse_expression(&tokens, &mut expr_index)?;
                    i = expr_index;
                    
                    expect_keyword(&tokens, &mut i, "begin")?;
                    let for_body = parse_block(&tokens, &mut i, None)?;
                    expect_keyword(&tokens, &mut i, "end")?;
            
                        stmts.push(Statement::ForLoop {
                            init_var,
                            init_value,
                            condition,
                            increment_var,
                            increment_expr,
                        body: for_body
                        });
                } else {
                    let expr = parse_expression(&tokens, &mut i)?;
                    stmts.push(Statement::Expr(expr));
                }
            }

            Token::Keyword(kw) if kw == "struct" => {
                i += 1;
                let name = expect_identifier(&tokens, &mut i)?;
                expect_keyword(&tokens, &mut i, "begin")?;

                let mut fields = Vec::new();
                while let Some(token) = tokens.get(i) {
                    if let Token::Keyword(kw) = token {
                        if kw == "end" {
                            break;
                        }
                    }

                    let field_name = match token {
                        Token::Identifier(name) => name.clone(),
                        _ => return Err(ParseError::UnexpectedToken(format!("Esperado nome de campo, encontrado {:?}", token))),
                    };
                    i += 1;
                    expect_token_type(&tokens, &mut i, "Colon")?;
                    let field_type = parse_type(&tokens, &mut i)?;
                    fields.push((field_name, field_type));

                    if matches!(tokens.get(i), Some(Token::Comma)) {
                        i += 1;
                        if matches!(tokens.get(i), Some(Token::Keyword(kw)) if kw == "end") {
                            break;
                        }
                    }
                }

                expect_keyword(&tokens, &mut i, "end")?;
                stmts.push(Statement::StructDeclaration { name, fields });
            }

            Token::Keyword(kw) if kw == "spawn" => {
                i += 1;
                let thread_name = if let Some(Token::Identifier(name)) = tokens.get(i) {
                    i += 1;
                    Some(name.clone())
                } else {
                    None
                };

                expect_keyword(&tokens, &mut i, "begin")?;
                let body = parse_block(&tokens, &mut i, None)?;
                expect_keyword(&tokens, &mut i, "end")?;

                stmts.push(Statement::Spawn { 
                    body,
                    thread_name,
                });
            }

            Token::Keyword(kw) if kw == "wait" => {
                i += 1;
                
                let mut thread_names = Vec::new();
                
                if matches!(tokens.get(i), Some(Token::LeftBracket)) {
                    i += 1;
                    
                    while i < tokens.len() {
                        match tokens.get(i) {
                            Some(Token::Identifier(name)) => {
                                thread_names.push(name.clone());
                                i += 1;
                            }
                            Some(Token::RightBracket) => {
                                i += 1;
                                break;
                            }
                            Some(Token::Comma) => {
                                i += 1;
                            }
                            Some(tok) => {
                                return Err(ParseError::UnexpectedToken(format!("Token inesperado na lista de threads: {:?}", tok)));
                            }
                            None => return Err(ParseError::UnexpectedToken("Faltando ']' para fechar a lista de threads".to_string())),
                        }
                    }
                } else {
                    let thread_name = expect_identifier(&tokens, &mut i)?;
                    thread_names.push(thread_name);
                }
                
                stmts.push(Statement::Wait { thread_names });
            }

            Token::Keyword(kw) if kw == "catch" => {
                i += 1;
                expect_keyword(&tokens, &mut i, "begin")?;
                let try_block = parse_block(&tokens, &mut i, None)?;
                expect_keyword(&tokens, &mut i, "end")?;

                let mut catch_blocks = Vec::new();
                while matches!(tokens.get(i), Some(Token::Keyword(kw)) if kw == "begin") {
                    i += 1;
                    let error_type = match tokens.get(i) {
                        Some(Token::StringLiteral(msg)) => msg.clone(),
                        _ => return Err(ParseError::UnexpectedToken("Esperado mensagem de erro após catch".to_string())),
                    };
                    i += 1;

                    expect_keyword(&tokens, &mut i, "begin")?;
                    let catch_body = parse_block(&tokens, &mut i, None)?;
                    expect_keyword(&tokens, &mut i, "end")?;

                    catch_blocks.push((error_type, catch_body));
                }

                stmts.push(Statement::CatchBlock { try_block, catch_blocks });
            }

            Token::Keyword(kw) if kw == "print" => {
                i += 1;
                let expr = parse_expression(&tokens, &mut i)?;
                stmts.push(Statement::Print(expr));
            }

            Token::Keyword(kw) if kw == "await" => {
                i += 1;
                if i < tokens.len() && matches!(tokens.get(i), Some(Token::Keyword(kw)) if kw == "call") {
                    i += 1;
                    let name = expect_identifier(&tokens, &mut i)?;
                    let args = parse_function_args(&tokens, &mut i)?;
                    
                    let call_stmt = Statement::CallFunction { name, args };
                    stmts.push(Statement::Await { future: Box::new(call_stmt) });
                } else {
                    let future_name = expect_identifier(&tokens, &mut i)?;
                    
                    let var_expr = Expression::Variable(future_name);
                    let stmt = Statement::Expr(var_expr);
                    stmts.push(Statement::Await { future: Box::new(stmt) });
                }
            }

            Token::Keyword(kw) if kw == "array" => {
                i += 1;
                let name = expect_identifier(&tokens, &mut i)?;
                
                expect_token_type(&tokens, &mut i, "LeftBracket")?;
                
                let mut elements = Vec::new();
                while i < tokens.len() && !matches!(tokens.get(i), Some(Token::RightBracket)) {
                    if matches!(tokens.get(i), Some(Token::Comma)) {
                        i += 1;
                        continue;
                    }
                    
                    let value = parse_value(&tokens, &mut i)?;
                    elements.push(value);
                    
                    if i < tokens.len() && matches!(tokens.get(i), Some(Token::Comma)) {
                        i += 1;
                    }
                }
                
                expect_token_type(&tokens, &mut i, "RightBracket")?;
                
                stmts.push(Statement::ArrayDeclaration { name, elements });
            }

            Token::Keyword(kw) if kw == "fn" => {
                i += 1;
                let stmt = parse_function_or_async_function(&tokens, &mut i, false, &mut scope_stack)?;
                stmts.push(stmt);
                }

            Token::Keyword(kw) if kw == "async" => {
                            i += 1;
                
                if matches!(tokens.get(i), Some(Token::Keyword(kw)) if kw == "fn") {
                i += 1;
                }
                
                let stmt = parse_function_or_async_function(&tokens, &mut i, true, &mut scope_stack)?;
                stmts.push(stmt);
            }

            Token::Keyword(kw) if kw == "call" => {
                i += 1;
                let name = expect_identifier(&tokens, &mut i)?;
                let args = parse_function_args(&tokens, &mut i)?;
                stmts.push(Statement::CallFunction { name, args });
            }

            Token::Keyword(kw) if kw == "return" => {
                i += 1;
                let mut expr_index = i;
                let expr = parse_expression(&tokens, &mut expr_index)?;
                i = expr_index;
                
                stmts.push(Statement::Return(expr));
            }

            _ => {
                i += 1;
                continue;
            }
        }
    }

    Ok(stmts)
}

pub fn parse_block(tokens: &[Token], start_index: &mut usize, current_scope: Option<&Vec<String>>) -> Result<Vec<Statement>, ParseError> {
    let mut statements = Vec::new();
    let mut block_depth = 1;
    let mut inner_index = *start_index;
    let mut scope_stack = match current_scope {
        Some(scope) => scope.clone(),
        None => Vec::new(),
    };

    while inner_index < tokens.len() {
        match &tokens[inner_index] {
            Token::Keyword(kw) if kw == "end" => {
                block_depth -= 1;
                if block_depth == 0 {
                    break;
                }
                inner_index += 1;
            }
            Token::Keyword(kw) if kw == "begin" => {
                block_depth += 1;
                inner_index += 1;
            }
            Token::Keyword(kw) if kw == "var" => {
                inner_index += 1;
                let name = expect_identifier(&tokens, &mut inner_index)?;
                expect_token_type(&tokens, &mut inner_index, "Equals")?;
                
                if inner_index < tokens.len() && matches!(&tokens[inner_index], Token::Keyword(kw) if kw == "async") {
                inner_index += 1;
                
                    if inner_index < tokens.len() && matches!(&tokens[inner_index], Token::Keyword(kw) if kw == "call") {
                inner_index += 1;
                        let func_name = expect_identifier(&tokens, &mut inner_index)?;
                        let args = parse_function_args(&tokens, &mut inner_index)?;
                        
                        let call_stmt = Statement::CallFunction { name: func_name, args };
                        let future_stmt = Statement::SpawnAsync { future: Box::new(call_stmt), thread_name: Some(name.clone()) };
                        statements.push(future_stmt);
                        continue;
                    }
                }
                
                let mut expr_index = inner_index;
                let expr = parse_expression(tokens, &mut expr_index)?;
                inner_index = expr_index;
                
                statements.push(Statement::VarDeclarationExpr(name, expr));
            }
            Token::Identifier(var_name) => {
                inner_index += 1;
                
                if inner_index < tokens.len() && matches!(&tokens[inner_index], Token::Assign) {
                    inner_index += 1;
                    
                    let mut expr_index = inner_index;
                    let expr = parse_expression(tokens, &mut expr_index)?;
                    inner_index = expr_index;
                    
                    statements.push(Statement::Assignment(var_name.clone(), expr));
                }
            }
            Token::Keyword(kw) if kw == "print" => {
                inner_index += 1;
                
                let mut expr_index = inner_index;
                let expr = parse_expression(tokens, &mut expr_index)?;
                inner_index = expr_index;
                
                statements.push(Statement::Print(expr));
            }
            Token::Keyword(kw) if kw == "await" => {
                inner_index += 1;
                
                if inner_index < tokens.len() && matches!(&tokens[inner_index], Token::Keyword(kw) if kw == "call") {
                    inner_index += 1;
                    let name = expect_identifier(&tokens, &mut inner_index)?;
                    let args = parse_function_args(&tokens, &mut inner_index)?;
                    
                    let call_stmt = Statement::CallFunction { name, args };
                    statements.push(Statement::Await { future: Box::new(call_stmt) });
                } else {
                    let future_name = expect_identifier(&tokens, &mut inner_index)?;
                    
                    let var_expr = Expression::Variable(future_name);
                    let stmt = Statement::Expr(var_expr);
                    statements.push(Statement::Await { future: Box::new(stmt) });
                }
            }
            Token::Keyword(kw) if kw == "if" => {
                inner_index += 1;
                
                let mut expr_index = inner_index;
                let condition = parse_expression(tokens, &mut expr_index)?;
                inner_index = expr_index;
                
                expect_keyword(&tokens, &mut inner_index, "begin")?;
                let if_body = parse_block(tokens, &mut inner_index, None)?;
                expect_keyword(&tokens, &mut inner_index, "end")?;
                
                statements.push(Statement::IfStatement { condition, body: if_body });
            }
            Token::Keyword(kw) if kw == "for" => {
                inner_index += 1;
                
                let mut has_equals = false;
                let mut lookahead = inner_index;
                
                while lookahead < tokens.len() && !matches!(&tokens[lookahead], Token::Semicolon) {
                    if matches!(&tokens[lookahead], Token::Assign) {
                        has_equals = true;
                        break;
                    }
                    lookahead += 1;
                }
                
                if has_equals {
                    let init_var = expect_identifier(&tokens, &mut inner_index)?;
                    expect_token_type(&tokens, &mut inner_index, "Equals")?;
                    
                    let mut expr_index = inner_index;
                    let init_value = parse_expression(&tokens, &mut expr_index)?;
                    inner_index = expr_index;
                    
                    expect_token_type(&tokens, &mut inner_index, "Semicolon")?;
                    
                    expr_index = inner_index;
                    let condition = parse_expression(&tokens, &mut expr_index)?;
                    inner_index = expr_index;
                    
                    expect_token_type(&tokens, &mut inner_index, "Semicolon")?;
                    
                    let increment_var = expect_identifier(&tokens, &mut inner_index)?;
                    expect_token_type(&tokens, &mut inner_index, "Equals")?;
                    
                    expr_index = inner_index;
                    let increment_expr = parse_expression(&tokens, &mut expr_index)?;
                    inner_index = expr_index;
                    
                    expect_keyword(&tokens, &mut inner_index, "begin")?;
                    let for_body = parse_block(&tokens, &mut inner_index, None)?;
                    expect_keyword(&tokens, &mut inner_index, "end")?;
                    
                    statements.push(Statement::ForLoop {
                        init_var,
                        init_value,
                        condition,
                        increment_var,
                        increment_expr,
                        body: for_body
                    });
                } else {
                    let expr = parse_expression(&tokens, &mut inner_index)?;
                    statements.push(Statement::Expr(expr));
                    }
            }
            Token::Keyword(kw) if kw == "call" => {
                    inner_index += 1;
                let name = expect_identifier(&tokens, &mut inner_index)?;
                let args = parse_function_args(&tokens, &mut inner_index)?;
                statements.push(Statement::CallFunction { name, args });
            }
            Token::Keyword(kw) if kw == "return" => {
                inner_index += 1;
                let mut expr_index = inner_index;
                let expr = parse_expression(tokens, &mut expr_index)?;
                inner_index = expr_index;
                
                statements.push(Statement::Return(expr));
            }
            Token::Keyword(kw) if kw == "async" => {
                inner_index += 1;
                
                if inner_index < tokens.len() && matches!(&tokens[inner_index], Token::Keyword(kw) if kw == "fn") {
                    inner_index += 1;
                }
                
                let stmt = parse_function_or_async_function(&tokens, &mut inner_index, true, &mut scope_stack)?;
                statements.push(stmt);
            }
            Token::Keyword(kw) if kw == "spawn" => {
                inner_index += 1;
                
                let thread_name = if let Some(Token::Identifier(name)) = tokens.get(inner_index) {
                    inner_index += 1;
                    Some(name.clone())
                } else {
                    None
                };
                
                expect_keyword(&tokens, &mut inner_index, "begin")?;
                let spawn_body = parse_block(tokens, &mut inner_index, None)?;
                expect_keyword(&tokens, &mut inner_index, "end")?;
                
                statements.push(Statement::Spawn { body: spawn_body, thread_name });
            }
            Token::Keyword(kw) if kw == "wait" => {
                inner_index += 1;
                
                let mut thread_names = Vec::new();
                
                if inner_index < tokens.len() && matches!(&tokens[inner_index], Token::LeftBracket) {
                    inner_index += 1;
                    
                    while inner_index < tokens.len() {
                        match &tokens[inner_index] {
                            Token::Identifier(name) => {
                                thread_names.push(name.clone());
                                inner_index += 1;
                            }
                            Token::RightBracket => {
                                inner_index += 1;
                                break;
                            }
                            Token::Comma => {
                                inner_index += 1;
                            }
                            tok => {
                                return Err(ParseError::UnexpectedToken(format!("Token inesperado na lista de threads: {:?}", tok)));
                            }
                        }
                    }
                } else if inner_index < tokens.len() {
                    match &tokens[inner_index] {
                        Token::Identifier(name) => {
                            thread_names.push(name.clone());
                            inner_index += 1;
                        }
                        tok => {
                            return Err(ParseError::UnexpectedToken(format!("Esperado identificador após 'wait', encontrado {:?}", tok)));
                        }
                    }
                } else {
                    return Err(ParseError::UnexpectedToken("Faltando identificador após 'wait'".to_string()));
                }
                
                statements.push(Statement::Wait { thread_names });
            }
            Token::Keyword(kw) if kw == "fn" => {
                inner_index += 1;
                let stmt = parse_function_or_async_function(&tokens, &mut inner_index, false, &mut scope_stack)?;
                statements.push(stmt);
            }
            _ => inner_index += 1,
        }
    }
    
    *start_index = inner_index;
    Ok(statements)
}
