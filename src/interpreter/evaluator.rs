use std::collections::HashMap;
use crate::ast::{Value, Expression, BinaryOperator, ComparisonOperator, LogicalOperator, Type, Statement};
use crate::value_parser::ParseError;
use crate::interpreter::error::{RuntimeError, RuntimeResult};
use crate::interpreter::runtime::MurlocRuntime;

pub fn evaluate_condition(condition: &Expression, env: &HashMap<String, Value>, runtime: Option<&MurlocRuntime>) -> bool {
    if let Ok(Value::Number(n)) = evaluate_expression(condition, env, runtime) {
        n != 0
    } else {
        panic!("Invalid condition in the ritual");
    }
}

pub fn evaluate_expression(expr: &Expression, env: &HashMap<String, Value>, runtime: Option<&MurlocRuntime>) -> RuntimeResult<Value> {
    println!("[DEBUG] evaluate_expression: avaliando {:?}", expr);
    match expr {
        Expression::Equals(_name, value) => {
            println!("[DEBUG] evaluate_expression: Equals");
            Ok(Value::Number(*value))
        },
        Expression::BinaryOp { left, right, op } => {
            println!("[DEBUG] evaluate_expression: BinaryOp");
            let left_val = evaluate_expression(left, env, runtime)?;
            let right_val = evaluate_expression(right, env, runtime)?;
            eval_binary_operation(&left_val, &right_val, op)
        },
        Expression::Comparison { left, right, op } => {
            println!("[DEBUG] evaluate_expression: Comparison");
            let left_val = evaluate_expression(left, env, runtime)?;
            let right_val = evaluate_expression(right, env, runtime)?;
            
            let result = match op {
                ComparisonOperator::Equals => left_val == right_val,
                ComparisonOperator::NotEquals => left_val != right_val,
                ComparisonOperator::LessThan => left_val < right_val,
                ComparisonOperator::GreaterThan => left_val > right_val,
                ComparisonOperator::LessThanOrEqual => left_val <= right_val,
                ComparisonOperator::GreaterThanOrEqual => left_val >= right_val,
            };
            
            Ok(Value::Number(if result { 1 } else { 0 }))
        },
        Expression::LogicalOp { left, right, op } => {
            println!("[DEBUG] evaluate_expression: LogicalOp");
            let left_val = evaluate_expression(left, env, runtime)?;
            
            match op {
                LogicalOperator::Not => {
                    if right.is_some() {
                        return Err(ParseError::InvalidValue("'Not' operator must not have a right operand".to_string()));
                    }
                    match left_val {
                        Value::Number(a) => Ok(Value::Number(if a == 0 { 1 } else { 0 })),
                        _ => Err(ParseError::InvalidValue("'Not' logical operand must be a number".to_string())),
                    }
                },
                _ => {
                    let right_val = evaluate_expression(right.as_ref().unwrap(), env, runtime)?;
                    match (left_val, right_val) {
                        (Value::Number(a), Value::Number(b)) => {
                            let result = match op {
                                LogicalOperator::And => a != 0 && b != 0,
                                LogicalOperator::Or => a != 0 || b != 0,
                                LogicalOperator::Not => unreachable!(),
                            };
                            Ok(Value::Number(if result { 1 } else { 0 }))
                        },
                        _ => Err(ParseError::InvalidValue("Logical operands must be numbers".to_string())),
                    }
                }
            }
        },
        Expression::Literal(value) => {
            println!("[DEBUG] evaluate_expression: Literal {:?}", value);
            Ok(value.clone())
        },
        Expression::Variable(name) => {
            println!("[DEBUG] evaluate_expression: Variable {}", name);
            if let Some(value) = env.get(name) {
                Ok(value.clone())
            } else {
                Err(ParseError::InvalidValue(format!("Variable '{}' not found in the cosmic void", name)))
            }
        },
        Expression::ArrayAccess { name, index } => {
            if let Some(Value::Array(arr)) = env.get(name) {
                if let Ok(Value::Number(idx)) = evaluate_expression(index, env, runtime) {
                    let idx = idx as usize;
                    if idx < arr.len() {
                        Ok(arr[idx].clone())
                    } else {
                        Err(ParseError::InvalidValue(format!("Array index {} out of bounds in the matrix", idx)))
                    }
                } else {
                    Err(ParseError::InvalidValue("Invalid array index format".to_string()))
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
                    Err(ParseError::InvalidValue(format!("Field '{}' not found in struct '{}'", field, name)))
                }
            } else {
                Err(ParseError::InvalidValue(format!("Struct '{}' not found in the cosmic void", name)))
            }
        },
        Expression::FunctionCall { name, args } => {
            println!("[DEBUG] evaluate_expression: FunctionCall {} com {} argumentos", name, args.len());
            if let Some(rt) = runtime {
                rt.call_function_from_expression(name, args.clone())
            } else {
                Err(ParseError::InvalidValue(format!(
                    "Function '{}' requires runtime for execution",
                    name
                )))
            }
        },
        Expression::StructInstance { struct_name, fields } => {
            if let Some(rt) = runtime {
                let structs = rt.env.structs.lock().unwrap();
                if let Some(struct_fields) = structs.get(struct_name) {
                    let mut new_fields = Vec::new();
                    
                    for (field_name, field_expr) in fields {
                        if !struct_fields.iter().any(|(name, _)| name == field_name) {
                            return Err(ParseError::InvalidValue(format!(
                                "Field '{}' does not exist in struct '{}'",
                                field_name, struct_name
                            )));
                        }
                        
                        let field_value = evaluate_expression(field_expr, env, runtime)?;
                        
                        if let Some((_, expected_type)) = struct_fields.iter().find(|(name, _)| name == field_name) {
                            let type_matches = match (&field_value, expected_type) {
                                (Value::Number(_), Type::Number) => true,
                                (Value::Text(_), Type::Text) => true,
                                (Value::Array(_), Type::Array(_)) => true,
                                (Value::Struct(_, _), Type::Struct(_)) => true,
                                _ => false
                            };
                            
                            if !type_matches {
                                return Err(ParseError::InvalidValue(format!(
                                    "Type mismatch in struct '{}' field '{}': expected {}, found {}",
                                    struct_name, field_name, expected_type, field_value
                                )));
                            }
                        }
                        
                        new_fields.push((field_name.clone(), field_value));
                    }
                    
                    Ok(Value::Struct(struct_name.clone(), new_fields))
                } else {
                    Err(ParseError::InvalidValue(format!("Type '{}' not found in the cosmic void", struct_name)))
                }
            } else {
                Err(ParseError::InvalidValue("Runtime required to create struct instance".to_string()))
            }
        },
        Expression::InOperator { left, right } => {
            let left_val = evaluate_expression(left, env, runtime)?;
            let right_val = evaluate_expression(right, env, runtime)?;
            
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

pub fn fish_value_sort(values: &mut [Value]) {
    values.sort_by(|a, b| a.to_string().cmp(&b.to_string()));
}

pub fn eval_binary_operation(left_val: &Value, right_val: &Value, op: &BinaryOperator) -> RuntimeResult<Value> {
    match (left_val, right_val) {
        (Value::Number(a), Value::Number(b)) => {
            match op {
                BinaryOperator::Add => Ok(Value::Number(a + b)),
                BinaryOperator::Subtract => Ok(Value::Number(a - b)),
                BinaryOperator::Multiply => Ok(Value::Number(a * b)),
                BinaryOperator::Divide => {
                    if *b == 0 {
                        Err(RuntimeError::DivisionByZero.into())
                    } else {
                        Ok(Value::Number(a / b))
                    }
                },
                BinaryOperator::Modulo => Ok(Value::Number(a % b)),
            }
        },
        (Value::Text(a), Value::Text(b)) if matches!(op, BinaryOperator::Add) => 
            Ok(Value::Text(format!("{}{}", a, b))),
        (Value::Text(a), Value::Number(b)) if matches!(op, BinaryOperator::Add) => 
            Ok(Value::Text(format!("{}{}", a, b))),
        (Value::Number(a), Value::Text(b)) if matches!(op, BinaryOperator::Add) => 
            Ok(Value::Text(format!("{}{}", a, b))),
        (Value::Text(a), Value::Struct(_, fields)) if matches!(op, BinaryOperator::Add) => {
            Ok(Value::Text(format!("{}{}", a, Value::Struct(String::new(), fields.clone()))))
        },
        (Value::Struct(_, fields), Value::Text(b)) if matches!(op, BinaryOperator::Add) => {
            Ok(Value::Text(format!("{}{}", Value::Struct(String::new(), fields.clone()), b)))
        },
        _ => Err(RuntimeError::InvalidOperation(format!("Invalid operation in the cosmic void: cannot perform {:?} between {:?} and {:?}", op, left_val, right_val)).into()),
    }
} 