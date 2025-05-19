use crate::lexer::Token;
use crate::ast::{Statement, Value, Expression, ImportSpecifier};
use crate::expression_parser::parse_expression;
use crate::value_parser::{parse_value, parse_type, ParseError};

fn expect_identifier(tokens: &[Token], index: &mut usize) -> Result<String, ParseError> {
    match tokens.get(*index) {
        Some(Token::Identifier(name)) => {
            *index += 1;
            Ok(name.clone())
        },
        Some(tok) => Err(ParseError::UnexpectedToken(format!("Expected identifier, found {:?}", tok))),
        None => Err(ParseError::UnexpectedToken("Unexpected end, expected identifier".to_string())),
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
                Err(ParseError::UnexpectedToken(format!("Expected {}, found {:?}", expected_type, token)))
            }
        },
        None => Err(ParseError::UnexpectedToken(format!("Unexpected end, expected {}", expected_type))),
    }
}

fn expect_keyword(tokens: &[Token], index: &mut usize, keyword: &str) -> Result<(), ParseError> {
    match tokens.get(*index) {
        Some(Token::Keyword(kw)) if kw == keyword => {
            *index += 1;
            Ok(())
        },
        Some(tok) => Err(ParseError::UnexpectedToken(format!("Expected keyword '{}', found {:?}", keyword, tok))),
        None => Err(ParseError::UnexpectedToken(format!("Unexpected end, expected keyword '{}'", keyword))),
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
                    return Err(ParseError::InvalidValue(format!("Invalid number: {}", num)));
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
                            return Err(ParseError::InvalidValue(format!("Invalid number: -{}", num)));
                        }
                        *index += 1;
                    } else {
                        return Err(ParseError::UnexpectedToken("Expected number after minus sign".to_string()));
                    }
                }
            },
            tok => {
                return Err(ParseError::UnexpectedToken(format!("Unexpected token in function arguments: {:?}", tok)));
            },
        }
    }
    
    if !(*index > 0 && matches!(&tokens[*index-1], Token::RightParen)) {
        return Err(ParseError::UnexpectedToken("Missing closing parenthesis after function arguments".to_string()));
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
                return Err(ParseError::UnexpectedToken(format!("Unexpected token in function parameters: {:?}", tok)));
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
    
    let parent_scope = (!scope_stack.is_empty()).then(|| scope_stack.clone());
    
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
                    
                    if matches!(tokens.get(i), Some(Token::Keyword(kw)) if kw == "call") {
                        i += 1;
                        
                        let func_name = expect_identifier(&tokens, &mut i)?;
                        let args = parse_function_args(&tokens, &mut i)?;
                        
                        let call_stmt = Statement::CallFunction { name: func_name.clone(), args: args.clone() };
                        let future_stmt = Statement::SpawnAsync { future: Box::new(call_stmt), thread_name: Some(name.clone()) };
                        stmts.push(future_stmt);
                        continue;
                    } else {
                        i += 1;
                        let func_name = expect_identifier(&tokens, &mut i)?;
                        let args = parse_function_args(&tokens, &mut i)?;
                        
                        stmts.push(Statement::VarDeclarationExpr(name, Expression::FunctionCall { 
                            name: func_name.clone(), 
                            args: args.clone() 
                        }));
                        continue;
                    }
                }
                
                let expr = parse_expression(&tokens, &mut i)?;
                stmts.push(Statement::VarDeclarationExpr(name, expr));
            }

            Token::Identifier(name) => {
                let var_name = name.clone();
                i += 1;
                
                if i < tokens.len() && matches!(&tokens[i], Token::Assign) {
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

                let mut else_branch = None;

                if let Some(Token::Keyword(kw)) = tokens.get(i) {
                    if kw == "else" {
                        i += 1;
            
                        if let Some(Token::Keyword(kw)) = tokens.get(i) {
                            if kw == "if" {
                                let else_if_stmt = parse_if_statement(&tokens, &mut i, &scope_stack)?;
                                else_branch = Some(Box::new(else_if_stmt));
                            } else {
                                expect_keyword(&tokens, &mut i, "begin")?;
                                let else_body = parse_block(&tokens, &mut i, Some(&scope_stack))?;
                                expect_keyword(&tokens, &mut i, "end")?;
            
                                else_branch = Some(Box::new(Statement::IfStatement {
                                    condition: Expression::Literal(Value::Number(1)),
                                    body: else_body,
                                    else_branch: None
                                }));
                            }
                        }
                    }
                }

                stmts.push(Statement::IfStatement {
                    condition,
                    body,
                    else_branch
                });
            }

            Token::Keyword(kw) if kw == "for" => {
                i += 1;
                
                if i < tokens.len() && matches!(&tokens[i], Token::Identifier(_)) {
                    let iterator_var = expect_identifier(&tokens, &mut i)?;
                    
                    if i < tokens.len() && matches!(&tokens[i], Token::Keyword(kw) if kw == "in") {
                        i += 1;
                        let array_name = expect_identifier(&tokens, &mut i)?;
                        
                        expect_keyword(&tokens, &mut i, "begin")?;
                        let body = parse_block(&tokens, &mut i, Some(&scope_stack))?;
                        expect_keyword(&tokens, &mut i, "end")?;
                        
                        stmts.push(Statement::ForInLoop {
                            iterator_var,
                            array_name,
                            body,
                        });
                    } else {
                        expect_token_type(&tokens, &mut i, "Equals")?;
                        let init_value = parse_expression(&tokens, &mut i)?;
                        expect_token_type(&tokens, &mut i, "Semicolon")?;
                        
                        let condition = parse_expression(&tokens, &mut i)?;
                        expect_token_type(&tokens, &mut i, "Semicolon")?;
                        
                        let increment_var = expect_identifier(&tokens, &mut i)?;
                        let increment_expr = parse_expression(&tokens, &mut i)?;
                        
                        expect_keyword(&tokens, &mut i, "begin")?;
                        let body = parse_block(&tokens, &mut i, Some(&scope_stack))?;
                        expect_keyword(&tokens, &mut i, "end")?;
                        
                        stmts.push(Statement::ForLoop {
                            init_var: iterator_var,
                            init_value,
                            condition,
                            increment_var,
                            increment_expr,
                            body,
                        });
                    }
                } else {
                    return Err(ParseError::UnexpectedToken("Expected identifier after 'for'".to_string()));
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
                        _ => return Err(ParseError::UnexpectedToken(format!("Expected field name, found {:?}", token))),
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
                let body = parse_block(&tokens, &mut i, Some(&scope_stack))?;
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
                                return Err(ParseError::UnexpectedToken(format!("Unexpected token in thread list: {:?}", tok)));
                            }
                            None => return Err(ParseError::UnexpectedToken("Missing ']' to close thread list".to_string())),
                        }
                    }
                } else {
                    let thread_name = expect_identifier(&tokens, &mut i)?;
                    thread_names.push(thread_name);
                }
                
                stmts.push(Statement::Wait { thread_names });
            }
            Token::Keyword(kw) if kw == "try" => {
                i += 1;
            
                expect_keyword(&tokens, &mut i, "begin")?;
                let try_block = parse_block(&tokens, &mut i, None)?;
                expect_keyword(&tokens, &mut i, "end")?;
            
                expect_keyword(&tokens, &mut i, "catch")?;
            
                let catch_param = if matches!(tokens.get(i), Some(Token::LeftParen)) {
                    i += 1;
                    let param_name = match tokens.get(i) {
                        Some(Token::Identifier(name)) => name.clone(),
                        _ => return Err(ParseError::UnexpectedToken("Expected identifier as catch param".into())),
                    };
                    i += 1;
                    if matches!(tokens.get(i), Some(Token::RightParen)) {
                        i += 1;
                    } else {
                        return Err(ParseError::UnexpectedToken("Expected ')' after catch param".into()));
                    }
                    Some(param_name)
                } else {
                    None
                };
            
                expect_keyword(&tokens, &mut i, "begin")?;
                let catch_body = parse_block(&tokens, &mut i, None)?;
                expect_keyword(&tokens, &mut i, "end")?;
            
                stmts.push(Statement::TryBlock {
                    try_block,
                    catch_param,
                    catch_body,
                });
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
                
                let next_token = tokens.get(i);
                let is_expression = match next_token {
                    Some(Token::Keyword(_)) | Some(Token::Identifier(_)) | None => true,
                    _ => false
                };
                
                if is_expression {
                    stmts.push(Statement::Expr(Expression::FunctionCall { name, args }));
                } else {
                    stmts.push(Statement::CallFunction { name, args });
                }
            }

            Token::Keyword(kw) if kw == "return" => {
                i += 1;
                let mut expr_index = i;
                let expr = parse_expression(&tokens, &mut expr_index)?;
                i = expr_index;
                
                stmts.push(Statement::Return(expr));
            }

            Token::Keyword(kw) if kw == "import" => {
                i += 1;
                let mut imports = Vec::new();
                
                if let Some(Token::Identifier(name)) = tokens.get(i) {
                    i += 1;
                    expect_keyword(&tokens, &mut i, "from")?;
                    if let Some(Token::StringLiteral(path)) = tokens.get(i) {
                        i += 1;
                        imports.push(ImportSpecifier::Default(name.clone()));
                        stmts.push(Statement::Import {
                            path: path.clone(),
                            imports,
                        });
                    } else {
                        return Err(ParseError::UnexpectedToken("Expected string literal after 'from'".to_string()));
                    }
                } else if let Some(Token::LeftBrace) = tokens.get(i) {
                    i += 1;
                    while i < tokens.len() {
                        if let Some(Token::RightBrace) = tokens.get(i) {
                            i += 1;
                            break;
                        }
                        
                        let specifier = parse_import_specifier(&tokens, &mut i)?;
                        imports.push(specifier);
                        
                        if let Some(Token::Comma) = tokens.get(i) {
                            i += 1;
                        } else if let Some(Token::RightBrace) = tokens.get(i) {
                            i += 1;
                            break;
                        } else {
                            return Err(ParseError::UnexpectedToken("Expected ',' or '}' in import specifiers".to_string()));
                        }
                    }
                    
                    expect_keyword(&tokens, &mut i, "from")?;
                    if let Some(Token::StringLiteral(path)) = tokens.get(i) {
                        i += 1;
                        stmts.push(Statement::Import {
                            path: path.clone(),
                            imports,
                        });
                    } else {
                        return Err(ParseError::UnexpectedToken("Expected string literal after 'from'".to_string()));
                    }
                } else {
                    return Err(ParseError::UnexpectedToken("Invalid import statement".to_string()));
                }
            }

            Token::Keyword(kw) if kw == "export" => {
                i += 1;
                let is_default = if let Some(Token::Keyword(kw)) = tokens.get(i) {
                    if kw == "default" {
                        i += 1;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                };

                if let Some(Token::Identifier(name)) = tokens.get(i) {
                    i += 1;
                    stmts.push(Statement::Export {
                        name: name.clone(),
                        is_default,
                    });
                } else {
                    return Err(ParseError::UnexpectedToken("Expected identifier after 'export'".to_string()));
                }
            }

            Token::Keyword(kw) if kw == "while" => {
                i += 1;
                let condition = parse_expression(&tokens, &mut i)?;
                
                expect_keyword(&tokens, &mut i, "begin")?;
                let body = parse_block(&tokens, &mut i, Some(&scope_stack))?;
                expect_keyword(&tokens, &mut i, "end")?;

                stmts.push(Statement::WhileLoop { condition, body });
            }

            Token::Keyword(kw) if kw == "break" => {
                i += 1;
                stmts.push(Statement::Break);
            }

            Token::Keyword(kw) if kw == "continue" => {
                i += 1;
                stmts.push(Statement::Continue);
            }

            Token::Keyword(kw) if kw == "switch" => {
                i += 1;
                let value = parse_expression(&tokens, &mut i)?;
                
                expect_keyword(&tokens, &mut i, "begin")?;
                
                let mut cases = Vec::new();
                let mut default = None;
                
                while i < tokens.len() {
                    if let Token::Keyword(kw) = &tokens[i] {
                        if kw == "end" {
                            i += 1;
                            break;
                        } else if kw == "case" {
                            i += 1;
                            let case_value = parse_value(&tokens, &mut i)?;
                            expect_token_type(&tokens, &mut i, "Colon")?;
                            
                            let case_body = parse_case_block(&tokens, &mut i)?;
                            cases.push((case_value, case_body));
                        } else if kw == "default" {
                            i += 1;
                            expect_token_type(&tokens, &mut i, "Colon")?;
                            
                            let default_body = parse_case_block(&tokens, &mut i)?;
                            default = Some(default_body);
                        } else {
                            i += 1;
                        }
                    } else {
                        i += 1;
                    }
                }
                
                stmts.push(Statement::SwitchStatement {
                    value,
                    cases,
                    default,
                });
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
                        
                        let call_stmt = Statement::CallFunction { name: func_name.clone(), args: args.clone() };
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

                expect_keyword(tokens, &mut inner_index, "begin")?;
                let if_body = parse_block(tokens, &mut inner_index, None)?;
                expect_keyword(tokens, &mut inner_index, "end")?;

                let mut else_branch = None;

                if let Some(Token::Keyword(kw)) = tokens.get(inner_index) {
                    if kw == "else" {
                        inner_index += 1;
                
                        if let Some(Token::Keyword(next_kw)) = tokens.get(inner_index) {
                            if next_kw == "if" {
                                let else_if_stmt = parse_if_statement(tokens, &mut inner_index, &vec![])?;
                                else_branch = Some(Box::new(else_if_stmt));
                            } else if next_kw == "begin" {
                                inner_index += 1;
                                let else_body = parse_block(tokens, &mut inner_index, None)?;
                                expect_keyword(tokens, &mut inner_index, "end")?;
                
                                else_branch = Some(Box::new(Statement::IfStatement {
                                    condition: Expression::Literal(Value::Number(1)),
                                    body: else_body,
                                    else_branch: None
                                }));
                            } else {
                                return Err(ParseError::UnexpectedToken(
                                    format!("Esperado 'if' ou 'begin' após 'else', encontrado {:?}", tokens.get(inner_index))
                                ));
                            }
                        }
                    }
                }

                statements.push(Statement::IfStatement {
                    condition,
                    body: if_body,
                    else_branch,
                });
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
                    let iterator_var = expect_identifier(&tokens, &mut inner_index)?;
                    expect_keyword(&tokens, &mut inner_index, "in")?;
                    let array_name = expect_identifier(&tokens, &mut inner_index)?;
                    
                    expect_keyword(&tokens, &mut inner_index, "begin")?;
                    let body = parse_block(&tokens, &mut inner_index, Some(&scope_stack))?;
                    expect_keyword(&tokens, &mut inner_index, "end")?;
                    
                    statements.push(Statement::ForInLoop {
                        iterator_var,
                        array_name,
                        body,
                    });
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
                let spawn_body = parse_block(tokens, &mut inner_index, Some(&scope_stack))?;
                expect_keyword(&tokens, &mut inner_index, "end")?;
                
                statements.push(Statement::Spawn { 
                    body: spawn_body,
                    thread_name,
                });
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
                                return Err(ParseError::UnexpectedToken(format!("Unexpected token in thread list: {:?}", tok)));
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
                            return Err(ParseError::UnexpectedToken(format!("Expected identifier after 'wait', found {:?}", tok)));
                        }
                    }
                } else {
                    return Err(ParseError::UnexpectedToken("Missing identifier after 'wait'".to_string()));
                }
                
                statements.push(Statement::Wait { thread_names });
            },
            Token::Keyword(kw) if kw == "while" => {
                inner_index += 1;
                
                let mut expr_index = inner_index;
                let condition = parse_expression(tokens, &mut expr_index)?;
                inner_index = expr_index;
                
                expect_keyword(&tokens, &mut inner_index, "begin")?;
                let while_body = parse_block(tokens, &mut inner_index, None)?;
                expect_keyword(&tokens, &mut inner_index, "end")?;
                
                statements.push(Statement::WhileLoop { condition, body: while_body });
            },
            Token::Keyword(kw) if kw == "fn" => {
                inner_index += 1;
                let stmt = parse_function_or_async_function(&tokens, &mut inner_index, false, &mut scope_stack)?;
                statements.push(stmt);
            },
            Token::Keyword(kw) if kw == "break" => {
                inner_index += 1;
                statements.push(Statement::Break);
            },
            Token::Keyword(kw) if kw == "continue" => {
                inner_index += 1;
                statements.push(Statement::Continue);
            },
            Token::Keyword(kw) if kw == "switch" => {
                inner_index += 1;
                
                let mut expr_index = inner_index;
                let value = parse_expression(tokens, &mut expr_index)?;
                inner_index = expr_index;
                
                expect_keyword(&tokens, &mut inner_index, "begin")?;
                
                let mut cases = Vec::new();
                let mut default = None;
                
                while inner_index < tokens.len() {
                    if let Token::Keyword(kw) = &tokens[inner_index] {
                        if kw == "end" {
                            inner_index += 1;
                            break;
                        } else if kw == "case" {
                            inner_index += 1;
                            let case_value = parse_value(&tokens, &mut inner_index)?;
                            expect_token_type(&tokens, &mut inner_index, "Colon")?;
                            
                            let case_body = parse_case_block(&tokens, &mut inner_index)?;
                            cases.push((case_value, case_body));
                        } else if kw == "default" {
                            inner_index += 1;
                            expect_token_type(&tokens, &mut inner_index, "Colon")?;
                            
                            let default_body = parse_case_block(&tokens, &mut inner_index)?;
                            default = Some(default_body);
                        } else {
                            inner_index += 1;
                        }
                    } else {
                        inner_index += 1;
                    }
                }
                
                statements.push(Statement::SwitchStatement {
                    value,
                    cases,
                    default,
                });
            },
            _ => inner_index += 1,
        }
    }
    
    *start_index = inner_index;
    Ok(statements)
}

fn parse_import_specifier(tokens: &[Token], index: &mut usize) -> Result<ImportSpecifier, ParseError> {
    match tokens.get(*index) {
        Some(Token::Identifier(name)) => {
            *index += 1;
            
            if let Some(Token::Keyword(kw)) = tokens.get(*index) {
                if kw == "as" {
                    *index += 1;
                    if let Some(Token::Identifier(alias)) = tokens.get(*index) {
                        *index += 1;
                        return Ok(ImportSpecifier::Named(name.clone(), alias.clone()));
                    }
                }
            }
            
            Ok(ImportSpecifier::Specific(name.clone()))
        },
        Some(Token::Keyword(kw)) if kw == "*" => {
            *index += 1;
            expect_keyword(tokens, index, "as")?;
            if let Some(Token::Identifier(name)) = tokens.get(*index) {
                *index += 1;
                Ok(ImportSpecifier::Namespace(name.clone()))
            } else {
                Err(ParseError::UnexpectedToken("Expected identifier after 'as'".to_string()))
            }
        },
        _ => Err(ParseError::UnexpectedToken("Invalid import specifier".to_string()))
    }
}

fn parse_if_statement(
    tokens: &[Token],
    i: &mut usize,
    scope_stack: &Vec<String>,
) -> Result<Statement, ParseError> {
    *i += 1;

    let condition = parse_expression(tokens, i)?;
    expect_keyword(tokens, i, "begin")?;
    let body = parse_block(tokens, i, Some(scope_stack))?;
    expect_keyword(tokens, i, "end")?;

    let mut else_branch = None;

    if let Some(Token::Keyword(kw)) = tokens.get(*i) {
        if kw == "else" {
            *i += 1;
    
            match tokens.get(*i) {
                Some(Token::Keyword(next_kw)) if next_kw == "if" => {
                    let else_if_stmt = parse_if_statement(tokens, i, scope_stack)?;
                    else_branch = Some(Box::new(else_if_stmt));
                }
                Some(Token::Keyword(next_kw)) if next_kw == "begin" => {
                    *i += 1;
                    let else_body = parse_block(tokens, i, Some(scope_stack))?;
                    expect_keyword(tokens, i, "end")?;
                                else_branch = Some(Box::new(Statement::IfStatement {
                                    condition: Expression::Literal(Value::Number(1)),
                                    body: else_body,
                                    else_branch: None
                                }));
                }
                _ => {
                    return Err(ParseError::UnexpectedToken(
                        format!("Esperado 'if' ou 'begin' após 'else', encontrado {:?}", tokens.get(*i))
                    ));
                }
            }
        }
    }
    

    Ok(Statement::IfStatement {
        condition,
        body,
        else_branch,
    })
}

fn parse_case_block(tokens: &[Token], index: &mut usize) -> Result<Vec<Statement>, ParseError> {
    let mut statements = Vec::new();
    
    while *index < tokens.len() {
        if let Token::Keyword(kw) = &tokens[*index] {
            if kw == "case" || kw == "default" || kw == "end" {
                break;
            }
        }
        
        match &tokens[*index] {
            Token::Keyword(kw) if kw == "var" => {
                *index += 1;
                let name = expect_identifier(&tokens, index)?;
                expect_token_type(&tokens, index, "Equals")?;
                let expr = parse_expression(&tokens, index)?;
                statements.push(Statement::VarDeclarationExpr(name, expr));
            },
            Token::Identifier(name) => {
                let var_name = name.clone();
                *index += 1;
                
                if *index < tokens.len() && matches!(&tokens[*index], Token::Assign) {
                    *index += 1;
                    let expr = parse_expression(&tokens, index)?;
                    statements.push(Statement::Assignment(var_name, expr));
                }
            },
            Token::Keyword(kw) if kw == "break" => {
                *index += 1;
                statements.push(Statement::Break);
            },
            Token::Keyword(kw) if kw == "continue" => {
                *index += 1;
                statements.push(Statement::Continue);
            },
            Token::Keyword(kw) if kw == "return" => {
                *index += 1;
                let expr = parse_expression(&tokens, index)?;
                statements.push(Statement::Return(expr));
            },
            Token::Keyword(kw) if kw == "if" => {
                let statement = parse_if_statement(&tokens, index, &vec![])?;
                statements.push(statement);
            },
            Token::Keyword(kw) if kw == "call" => {
                *index += 1;
                let name = expect_identifier(&tokens, index)?;
                let args = parse_function_args(&tokens, index)?;
                statements.push(Statement::CallFunction { name, args });
            },
            Token::Keyword(kw) if kw == "print" => {
                *index += 1;
                let expr = parse_expression(&tokens, index)?;
                statements.push(Statement::Print(expr));
            },
            _ => *index += 1,
        }
    }
    
    Ok(statements)
}
