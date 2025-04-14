use crate::lexer::Token;
use crate::ast::{Statement, Value};
use crate::expression_parser::parse_expression;
use crate::value_parser::{parse_value, parse_type, ParseError};
use crate::Expression;

pub fn parse(tokens: Vec<Token>) -> Result<Vec<Statement>, ParseError> {
    let mut stmts = Vec::new();
    let mut i = 0;
 
    while i < tokens.len() {
        println!("Processando token[{}]: {:?}", i, tokens[i]);
        
        if let Token::Identifier(name) = &tokens[i] {
            println!("Encontrado identificador: {} em i={}", name, i);
            let is_param = i > 0 && matches!(&tokens[i-1], Token::LeftParen | Token::Comma);
            if is_param {
                println!("  -> Parece ser um parâmetro de função");
            }

            if i + 1 < tokens.len() {
                println!("  -> Próximo token: {:?}", tokens[i+1]);
            }
        }
        
        match &tokens[i] {
            Token::Keyword(kw) if kw == "Glrm" => {
                i += 1;
                let name = match tokens.get(i) {
                    Some(Token::Identifier(name)) => name.clone(),
                    Some(tok) => return Err(ParseError::UnexpectedToken(format!("Esperado identificador, encontrado {:?}", tok))),
                    None => return Err(ParseError::UnexpectedToken("Faltando identificador após 'grrr'".to_string())),
                };
                i += 1;
                if !matches!(tokens.get(i), Some(Token::Equals)) {
                    return Err(ParseError::UnexpectedToken(format!("Esperado '=', encontrado {:?}", tokens.get(i))));
                }
                i += 1;
                let expr = parse_expression(&tokens, &mut i)?;
                stmts.push(Statement::VarDeclarationExpr(name, expr));
            }

            Token::Identifier(name) => {
                let var_name = name.clone();
                i += 1;
                
                if i < tokens.len() && matches!(tokens.get(i), Some(Token::Equals)) {
                    i += 1;
                    let expr = parse_expression(&tokens, &mut i)?;
                    stmts.push(Statement::Assignment(var_name, expr));
                } else {
                    println!("Identificador sem atribuição em i={}: {:?}", i-1, var_name);
                    continue;
                }
            }

            Token::Keyword(kw) if kw == "Mrglif" => {
                i += 1;
                let condition = parse_expression(&tokens, &mut i)?;
                
                if !matches!(tokens.get(i), Some(Token::Keyword(k)) if k == "Mrgl") {
                    return Err(ParseError::UnexpectedToken("Faltando 'mrgl' após condição".to_string()));
                }
                i += 1;

                let body = parse_block(&tokens, &mut i)?;

                if !matches!(tokens.get(i), Some(Token::Keyword(k)) if k == "Grl") {
                    return Err(ParseError::UnexpectedToken("Faltando 'grl' para fechar o bloco".to_string()));
                }
                i += 1;

                stmts.push(Statement::IfStatement { condition, body });
            }

            Token::Keyword(kw) if kw == "Mrrg" => {
                i += 1;
            
                if let Some(Token::Identifier(_)) = tokens.get(i) {
                    if let Some(Token::Equals) = tokens.get(i + 1) {
                        let init_var = match tokens.get(i) {
                            Some(Token::Identifier(name)) => name.clone(),
                            _ => return Err(ParseError::UnexpectedToken("Esperado identificador após 'mrrg'".into())),
                        };
                        i += 1;
            
                        if !matches!(tokens.get(i), Some(Token::Equals)) {
                            return Err(ParseError::UnexpectedToken("Esperado '=' após variável".into()));
                        }
                        i += 1;
            
                        let init_value = parse_expression(&tokens, &mut i)?;
            
                        if !matches!(tokens.get(i), Some(Token::Semicolon)) {
                            return Err(ParseError::UnexpectedToken("Faltando ';' após inicialização".to_string()));
                        }
                        i += 1;
            
                        let condition = parse_expression(&tokens, &mut i)?;
            
                        if !matches!(tokens.get(i), Some(Token::Semicolon)) {
                            return Err(ParseError::UnexpectedToken("Faltando ';' após condição".to_string()));
                        }
                        i += 1;
            
                        let increment_var = match tokens.get(i) {
                            Some(Token::Identifier(name)) => name.clone(),
                            _ => return Err(ParseError::UnexpectedToken("Esperado identificador no incremento".into())),
                        };
                        i += 1;
            
                        if !matches!(tokens.get(i), Some(Token::Equals)) {
                            return Err(ParseError::UnexpectedToken("Esperado '=' no incremento".into()));
                        }
                        i += 1;
            
                        let increment_expr = parse_expression(&tokens, &mut i)?;
            
                        if !matches!(tokens.get(i), Some(Token::Keyword(k)) if k == "Mrgl") {
                            return Err(ParseError::UnexpectedToken("Esperado 'mrgl' para iniciar bloco do loop".into()));
                        }
                        i += 1;
            
                        let body = parse_block(&tokens, &mut i)?;
            
                        if !matches!(tokens.get(i), Some(Token::Keyword(k)) if k == "Grl") {
                            return Err(ParseError::UnexpectedToken("Esperado 'grl' para fechar bloco do loop".into()));
                        }
                        i += 1;
            
                        stmts.push(Statement::ForLoop {
                            init_var,
                            init_value,
                            condition,
                            increment_var,
                            increment_expr,
                            body,
                        });
                    }
                } else {
                    let expr = parse_expression(&tokens, &mut i)?;
                    stmts.push(Statement::Expr(expr));
                }
            }

            Token::Keyword(kw) if kw == "Mrglstruct" => {
                i += 1;
                let name = match tokens.get(i) {
                    Some(Token::Identifier(name)) => name.clone(),
                    Some(tok) => return Err(ParseError::UnexpectedToken(format!("Esperado identificador, encontrado {:?}", tok))),
                    None => return Err(ParseError::UnexpectedToken("Faltando identificador após 'mrrgstruct'".to_string())),
                };
                i += 1;

                if !matches!(tokens.get(i), Some(Token::Keyword(kw)) if kw == "Mrgl") {
                    return Err(ParseError::UnexpectedToken("Faltando 'mrgl' após identificador".to_string()));
                }
                i += 1;

                let mut fields = Vec::new();
                while let Some(token) = tokens.get(i) {
                    if matches!(token, Token::Keyword(kw)) && kw == "Grl" {
                        break;
                    }
                    let field_name = match token {
                        Token::Identifier(name) => name.clone(),
                        _ => return Err(ParseError::UnexpectedToken(format!("Esperado nome de campo, encontrado {:?}", token))),
                    };
                    i += 1;
                    if !matches!(tokens.get(i), Some(Token::Colon)) {
                        return Err(ParseError::UnexpectedToken("Faltando ':' após nome do campo".to_string()));
                    }
                    i += 1;
                    let field_type = parse_type(&tokens, &mut i)?;
                    fields.push((field_name, field_type));

                    if matches!(tokens.get(i), Some(Token::Comma)) {
                        i += 1;
                    }
                }

                if !matches!(tokens.get(i), Some(Token::Keyword(kw)) if kw == "Grl") {
                    return Err(ParseError::UnexpectedToken("Faltando 'grl' para fechar a struct".to_string()));
                }
                i += 1;

                stmts.push(Statement::StructDeclaration { name, fields });
            }

            Token::Keyword(kw) if kw == "Mrglspawn" => {
                i += 1;
                let thread_name = if let Some(Token::Identifier(name)) = tokens.get(i) {
                    i += 1;
                    Some(name.clone())
                } else {
                    None
                };

                if !matches!(tokens.get(i), Some(Token::Keyword(kw)) if kw == "Mrgl") {
                    return Err(ParseError::UnexpectedToken("Faltando 'mrgl' após mrglspawn".to_string()));
                }
                i += 1;

                let body = parse_block(&tokens, &mut i)?;

                if !matches!(tokens.get(i), Some(Token::Keyword(kw)) if kw == "Grl") {
                    return Err(ParseError::UnexpectedToken("Faltando 'grl' para fechar o bloco".to_string()));
                }
                i += 1;

                stmts.push(Statement::Spawn { 
                    body,
                    thread_name,
                });
            }

            Token::Keyword(kw) if kw == "Mrglcatch" => {
                i += 1;
                if !matches!(tokens.get(i), Some(Token::Keyword(kw)) if kw == "Mrgl") {
                    return Err(ParseError::UnexpectedToken("Faltando 'mrgl' após mrglcatch".to_string()));
                }
                i += 1;

                let try_block = parse_block(&tokens, &mut i)?;

                if !matches!(tokens.get(i), Some(Token::Keyword(kw)) if kw == "Grl") {
                    return Err(ParseError::UnexpectedToken("Faltando 'grl' para fechar o bloco try".to_string()));
                }
                i += 1;

                let mut catch_blocks = Vec::new();
                while matches!(tokens.get(i), Some(Token::Keyword(kw)) if kw == "Mrgl") {
                    i += 1;
                    let error_type = match tokens.get(i) {
                        Some(Token::StringLiteral(msg)) => msg.clone(),
                        _ => return Err(ParseError::UnexpectedToken("Esperado mensagem de erro após catch".to_string())),
                    };
                    i += 1;

                    if !matches!(tokens.get(i), Some(Token::Keyword(kw)) if kw == "Mrgl") {
                        return Err(ParseError::UnexpectedToken("Faltando 'mrgl' após mensagem de erro".to_string()));
                    }
                    i += 1;

                    let catch_body = parse_block(&tokens, &mut i)?;

                    if !matches!(tokens.get(i), Some(Token::Keyword(kw)) if kw == "Grl") {
                        return Err(ParseError::UnexpectedToken("Faltando 'grl' para fechar o bloco catch".to_string()));
                    }
                    i += 1;

                    catch_blocks.push((error_type, catch_body));
                }

                stmts.push(Statement::CatchBlock { try_block, catch_blocks });
            }

            Token::Keyword(kw) if kw == "Mrglschool" => {
                i += 1;
                let name = match tokens.get(i) {
                    Some(Token::StringLiteral(name)) => name.clone(),
                    _ => return Err(ParseError::UnexpectedToken("Esperado nome do grupo após mrglschool".to_string())),
                };
                i += 1;

                if !matches!(tokens.get(i), Some(Token::Keyword(kw)) if kw == "Mrgl") {
                    return Err(ParseError::UnexpectedToken("Faltando 'mrgl' após nome do grupo".to_string()));
                }
                i += 1;

                let members = parse_block(&tokens, &mut i)?;

                if !matches!(tokens.get(i), Some(Token::Keyword(kw)) if kw == "Grl") {
                    return Err(ParseError::UnexpectedToken("Faltando 'grl' para fechar o grupo".to_string()));
                }
                i += 1;

                stmts.push(Statement::SchoolBlock { name, members });
            }

            Token::Keyword(kw) if kw == "Mrglprint" => {
                i += 1;
                let expr = parse_expression(&tokens, &mut i)?;
                stmts.push(Statement::Print(expr));
            }

            Token::Keyword(kw) if kw == "Mrglarray" => {
                i += 1;
                let name = match tokens.get(i) {
                    Some(Token::Identifier(name)) => name.clone(),
                    Some(tok) => return Err(ParseError::UnexpectedToken(format!("Esperado identificador, encontrado {:?}", tok))),
                    None => return Err(ParseError::UnexpectedToken("Faltando identificador após 'grrarray'".to_string())),
                };
                i += 1;
                
                if !matches!(tokens.get(i), Some(Token::LeftBracket)) {
                    return Err(ParseError::UnexpectedToken("Faltando '[' após nome do array".to_string()));
                }
                i += 1;
                
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
                
                if !matches!(tokens.get(i), Some(Token::RightBracket)) {
                    return Err(ParseError::UnexpectedToken("Faltando ']' para fechar o array".to_string()));
                }
                i += 1;
                
                stmts.push(Statement::ArrayDeclaration { name, elements });
            }

            Token::Keyword(kw) if kw == "Mrglfn" => {
                i += 1;
                let name = match tokens.get(i) {
                    Some(Token::Identifier(name)) => name.clone(),
                    Some(tok) => return Err(ParseError::UnexpectedToken(format!("Esperado identificador após 'grrrfnrrg', encontrado {:?}", tok))),
                    None => return Err(ParseError::UnexpectedToken("Faltando nome da função'".to_string())),
                };
                i += 1;
                
                if !matches!(tokens.get(i), Some(Token::LeftParen)) {
                    return Err(ParseError::UnexpectedToken("Esperado '(' após nome da função".to_string()));
                }
                i += 1;
                
                let mut args = Vec::new();
                while i < tokens.len() {
                    match tokens.get(i) {
                        Some(Token::Identifier(arg)) => {
                            args.push(arg.clone());
                            i += 1;
                        }
                        Some(Token::RightParen) => {
                            i += 1;
                            break;
                        }
                        Some(Token::Comma) => {
                            i += 1;
                        }
                        Some(tok) => {
                            return Err(ParseError::UnexpectedToken(format!("Token inesperado no argumento da função: {:?}", tok)));
                        }
                        None => return Err(ParseError::UnexpectedToken("Fechamento de parênteses esperado no argumento da função".to_string())),
                    }
                }
            
                if !matches!(tokens.get(i), Some(Token::Keyword(kw)) if kw == "Mrgl") {
                    return Err(ParseError::UnexpectedToken("Faltando 'mrgl' para iniciar o corpo da função".to_string()));
                }
                i += 1;
            
                let body = parse_block(&tokens, &mut i)?;
            
                if !matches!(tokens.get(i), Some(Token::Keyword(kw)) if kw == "Grl") {
                    return Err(ParseError::UnexpectedToken("Faltando 'grl' para fechar o corpo da função".to_string()));
                }
                i += 1;
            
                stmts.push(Statement::Function { name, args, body });
            }

            Token::Keyword(kw) if kw == "Mrglcall" => {
                i += 1;
                let name = match tokens.get(i) {
                    Some(Token::Identifier(name)) => name.clone(),
                    Some(tok) => return Err(ParseError::UnexpectedToken(format!("Esperado identificador após 'grrrblbl', encontrado {:?}", tok))),
                    None => return Err(ParseError::UnexpectedToken("Faltando nome da função após 'grrrblbl'".to_string())),
                };
                i += 1;
                
                let mut args = Vec::new();
                
                if i < tokens.len() {
                    if let Some(Token::Identifier(id)) = tokens.get(i) {
                        if id == "mrglarg" {
                            i += 1;
                        }
                    }
                }
                
                while i < tokens.len() {
                    if i >= tokens.len() || matches!(tokens.get(i), Some(Token::Keyword(_))) {
                        break;
                    }
                    
                    match tokens.get(i) {
                        Some(Token::Identifier(var_name)) => {
                            args.push(Expression::Variable(var_name.clone()));
                            i += 1;
                        },
                        Some(Token::Number(num)) => {
                            if let Ok(n) = num.parse::<i32>() {
                                args.push(Expression::Literal(Value::Number(n)));
                            } else if let Ok(n) = num.parse::<i64>() {
                                args.push(Expression::Literal(Value::NumberI64(n)));
                            } else if let Ok(n) = num.parse::<num_bigint::BigInt>() {
                                args.push(Expression::Literal(Value::NumberBig(n)));
                            } else {
                                return Err(ParseError::InvalidValue(format!("Número inválido: {}", num)));
                            }
                            i += 1;
                        },
                        Some(Token::StringLiteral(text)) => {
                            args.push(Expression::Literal(Value::Text(text.clone())));
                            i += 1;
                        },
                        Some(Token::Comma) => {
                            i += 1;
                        },
                        _ => break,
                    }
                }
                
                stmts.push(Statement::CallFunction { name, args });
            }

            Token::Keyword(kw) if kw == "Mrglreturn" => {
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

pub fn parse_block(tokens: &[Token], start_index: &mut usize) -> Result<Vec<Statement>, ParseError> {
    let mut statements = Vec::new();
    let mut block_depth = 1;
    let mut inner_index = *start_index;

    println!("Executando parse_block, começando em {}, token {:?}", inner_index, tokens.get(inner_index));

    while inner_index < tokens.len() {
        println!("parse_block loop: token[{}]: {:?}", inner_index, tokens[inner_index]);
        match &tokens[inner_index] {
            Token::Keyword(kw) if kw == "Grl" => {
                block_depth -= 1;
                if block_depth == 0 {
                    println!("Fim do bloco em {}, depth={}", inner_index, block_depth);
                    break;
                }
                inner_index += 1;
            }
            Token::Keyword(kw) if kw == "Mrgl" => {
                block_depth += 1;
                inner_index += 1;
                println!("Inicio de sub-bloco em {}, depth={}", inner_index, block_depth);
            }
            Token::Keyword(kw) if kw == "Glrm" => {
                inner_index += 1;
                
                if inner_index >= tokens.len() {
                    return Err(ParseError::UnexpectedToken("Fim inesperado após 'grrr'".to_string()));
                }
                
                let name = match &tokens[inner_index] {
                    Token::Identifier(name) => name.clone(),
                    tok => return Err(ParseError::UnexpectedToken(format!("Esperado identificador, encontrado {:?}", tok))),
                };
                inner_index += 1;
                
                if inner_index >= tokens.len() || !matches!(&tokens[inner_index], Token::Equals) {
                    return Err(ParseError::UnexpectedToken("Faltando '=' após identificador".to_string()));
                }
                inner_index += 1;
                
                let mut expr_index = inner_index;
                let expr = parse_expression(tokens, &mut expr_index)?;
                inner_index = expr_index;
                
                statements.push(Statement::VarDeclarationExpr(name, expr));
            }
            Token::Identifier(var_name) => {
                inner_index += 1;
                
                if inner_index < tokens.len() && matches!(&tokens[inner_index], Token::Equals) {
                    inner_index += 1;
                    
                    let mut expr_index = inner_index;
                    let expr = parse_expression(tokens, &mut expr_index)?;
                    inner_index = expr_index;
                    
                    statements.push(Statement::Assignment(var_name.clone(), expr));
                } else {
                    println!("Encontrado identificador {} em índice {}", var_name, inner_index-1);
                }
            }
            Token::Keyword(kw) if kw == "Mrglprint" => {
                inner_index += 1;
                
                if inner_index >= tokens.len() {
                    return Err(ParseError::UnexpectedToken("Fim inesperado após 'grrprint'".to_string()));
                }
                
                let mut expr_index = inner_index;
                let expr = parse_expression(tokens, &mut expr_index)?;
                inner_index = expr_index;
                
                statements.push(Statement::Print(expr));
            }
            Token::Keyword(kw) if kw == "Mrglif" => {
                inner_index += 1;
                
                let mut expr_index = inner_index;
                let condition = parse_expression(tokens, &mut expr_index)?;
                inner_index = expr_index;
                
                if inner_index >= tokens.len()
                    || !matches!(&tokens[inner_index], Token::Keyword(kw) if kw == "Mrgl")
                {
                    return Err(ParseError::UnexpectedToken("Faltando 'mrgl' após condição".to_string()));
                }
                inner_index += 1;
                
                let if_body = parse_block(tokens, &mut inner_index)?;
                
                if inner_index >= tokens.len()
                    || !matches!(&tokens[inner_index], Token::Keyword(kw) if kw == "Grl")
                {
                    return Err(ParseError::UnexpectedToken("Faltando 'grl' para fechar o bloco if".to_string()));
                }
                inner_index += 1;
                
                statements.push(Statement::IfStatement { condition, body: if_body });
            }
            Token::Keyword(kw) if kw == "Mrrg" => {
                inner_index += 1;
                
                let mut has_equals = false;
                let mut lookahead = inner_index;
                
                while lookahead < tokens.len() && !matches!(&tokens[lookahead], Token::Semicolon) {
                    if matches!(&tokens[lookahead], Token::Equals) {
                        has_equals = true;
                        break;
                    }
                    lookahead += 1;
                }
                
                if has_equals {
                    let init_var = match &tokens[inner_index] {
                        Token::Identifier(name) => name.clone(),
                        tok => return Err(ParseError::UnexpectedToken(format!("Esperado identificador, encontrado {:?}", tok))),
                    };
                    inner_index += 1;
                    
                    if inner_index >= tokens.len() || !matches!(&tokens[inner_index], Token::Equals) {
                        return Err(ParseError::UnexpectedToken("Faltando '=' após identificador".to_string()));
                    }
                    inner_index += 1;
                    
                    let mut expr_index = inner_index;
                    let init_value = parse_expression(tokens, &mut expr_index)?;
                    inner_index = expr_index;
                    
                    if inner_index >= tokens.len() || !matches!(&tokens[inner_index], Token::Semicolon) {
                        return Err(ParseError::UnexpectedToken("Faltando ';' após inicialização".to_string()));
                    }
                    inner_index += 1;
                    
                    expr_index = inner_index;
                    let condition = parse_expression(tokens, &mut expr_index)?;
                    inner_index = expr_index;
                    
                    if inner_index >= tokens.len() || !matches!(&tokens[inner_index], Token::Semicolon) {
                        return Err(ParseError::UnexpectedToken("Faltando ';' após condição".to_string()));
                    }
                    inner_index += 1;
                    
                    let increment_var = match &tokens[inner_index] {
                        Token::Identifier(name) => name.clone(),
                        tok => return Err(ParseError::UnexpectedToken(format!("Esperado identificador, encontrado {:?}", tok))),
                    };
                    inner_index += 1;
                    
                    if inner_index >= tokens.len() || !matches!(&tokens[inner_index], Token::Equals) {
                        return Err(ParseError::UnexpectedToken("Faltando '=' após identificador".to_string()));
                    }
                    inner_index += 1;
                    
                    expr_index = inner_index;
                    let increment_expr = parse_expression(tokens, &mut expr_index)?;
                    inner_index = expr_index;
                    
                    if inner_index >= tokens.len() || !matches!(&tokens[inner_index], Token::Keyword(kw) if kw == "Mrgl") {
                        return Err(ParseError::UnexpectedToken("Faltando 'mrgl' para iniciar o bloco do loop".to_string()));
                    }
                    inner_index += 1;
                    
                    let for_body = parse_block(tokens, &mut inner_index)?;
                    
                    if inner_index >= tokens.len() || !matches!(&tokens[inner_index], Token::Keyword(kw) if kw == "Grl") {
                        return Err(ParseError::UnexpectedToken("Faltando 'grl' para fechar o bloco for".to_string()));
                    }
                    inner_index += 1;
                    
                    statements.push(Statement::ForLoop {
                        init_var,
                        init_value,
                        condition,
                        increment_var,
                        increment_expr,
                        body: for_body
                    });
                } else {
                    let mut expr_index = inner_index;
                    let condition = parse_expression(tokens, &mut expr_index)?;
                    inner_index = expr_index;
                    
                    if inner_index >= tokens.len() || !matches!(&tokens[inner_index], Token::Keyword(kw) if kw == "Mrgl") {
                        return Err(ParseError::UnexpectedToken("Faltando 'mrgl' após condição".to_string()));
                    }
                    inner_index += 1;
                    
                    let body = parse_block(tokens, &mut inner_index)?;
                    
                    if inner_index >= tokens.len() || !matches!(&tokens[inner_index], Token::Keyword(kw) if kw == "Grl") {
                        return Err(ParseError::UnexpectedToken("Faltando 'grl' para fechar o bloco do if implícito".to_string()));
                    }
                    inner_index += 1;
                    
                    statements.push(Statement::IfStatement { condition, body });
                }
            }
            Token::Keyword(kw) if kw == "Mrglcall" => {
                inner_index += 1;
                
                if inner_index >= tokens.len() {
                    return Err(ParseError::UnexpectedToken("Fim inesperado após 'grrrblbl'".to_string()));
                }
                
                let name = match &tokens[inner_index] {
                    Token::Identifier(name) => name.clone(),
                    tok => return Err(ParseError::UnexpectedToken(format!("Esperado identificador, encontrado {:?}", tok))),
                };
                inner_index += 1;
                
                let mut args = Vec::new();
                if inner_index < tokens.len() {
                    if let Token::Identifier(id) = &tokens[inner_index] {
                        if id == "mrglarg" {
                            inner_index += 1;
                        }
                    }
                }
                
                println!("CHAMADA DE FUNÇÃO: {} em inner_index={}", name, inner_index);
                while inner_index < tokens.len() {
                    if inner_index >= tokens.len() || matches!(&tokens[inner_index], Token::Keyword(_)) {
                        break;
                    }
                    
                    match &tokens[inner_index] {
                        Token::Identifier(var_name) => {
                            println!("  ARGUMENTO: Identifier({})", var_name);
                            args.push(Expression::Variable(var_name.clone()));
                            inner_index += 1;
                        },
                        Token::Number(num) => {
                            println!("  ARGUMENTO: Number({})", num);
                            if let Ok(n) = num.parse::<i32>() {
                                args.push(Expression::Literal(Value::Number(n)));
                            } else if let Ok(n) = num.parse::<i64>() {
                                args.push(Expression::Literal(Value::NumberI64(n)));
                            } else if let Ok(n) = num.parse::<num_bigint::BigInt>() {
                                args.push(Expression::Literal(Value::NumberBig(n)));
                            } else {
                                return Err(ParseError::InvalidValue(format!("Número inválido: {}", num)));
                            }
                            inner_index += 1;
                        },
                        Token::StringLiteral(text) => {
                            println!("  ARGUMENTO: StringLiteral({})", text);
                            args.push(Expression::Literal(Value::Text(text.clone())));
                            inner_index += 1;
                        },
                        Token::Comma => {
                            println!("  SEPARADOR: Comma");
                            inner_index += 1;
                        },
                        _ => {
                            println!("  TOKEN DESCONHECIDO: {:?}", tokens[inner_index]);
                            break;
                        },
                    }
                }
                println!("FIM DA CHAMADA DE FUNÇÃO: {}, argumentos: {}", name, args.len());
                statements.push(Statement::CallFunction { name, args });
            }
            Token::Keyword(kw) if kw == "Mrglreturn" => {
                inner_index += 1;
                let mut expr_index = inner_index;
                let expr = parse_expression(tokens, &mut expr_index)?;
                inner_index = expr_index;
                
                statements.push(Statement::Return(expr));
            }
            _ => inner_index += 1,
        }
    }
    
    *start_index = inner_index;
    Ok(statements)
}
