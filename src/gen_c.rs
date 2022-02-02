use crate::error::CompilerError;
use crate::statement::{Statement, ShuntedStackItem};
use crate::operator::Operator;

impl Operator {
    pub fn gen_c_code(&mut self) -> Result<String, CompilerError> {
        Ok(self.as_raw())
    }
}

impl Statement {
    fn gen_c_code(&mut self) -> Result<String, CompilerError> {
        match self {
            Statement::Program { exprs } => {
                let mut code = String::from("#include \"stdio.h\"\n");
                for expr in exprs {
                    code.push_str(&expr.gen_c_code()?);
                    code.push('\n');
                }
                Ok(code)
            }
            Statement::Block { exprs } => {
                let mut code = String::new();
                for expr in exprs {
                    code.push_str(&expr.gen_c_code()?);
                    code.push(';');
                    code.push('\n');
                }
                Ok(code)
            }
            Statement::Use { exprs } => {
                let mut code = String::new();
                for expr in exprs {
                    code.push_str(&expr.gen_c_code()?);
                    code.push('\n');
                }
                Ok(code)
            }
            Statement::Fn { ident, params, return_type, body } => {
                let name = match ident.as_ref().clone() {
                    Statement::Identifier { ident } => {
                        ident
                    }
                    _ => return Err(CompilerError::new("Expected identifier expression but AST provided an illegal expression."))
                };

                let (type_name, type_mods) = match return_type.as_ref().clone() {
                    Statement::Type { modifiers, type_ident } => {
                        let mut mods = Vec::new();
                        for m in modifiers {
                            let c = m.clone().gen_c_code()?;
                            mods.push(c);
                        }

                        let type_name = match type_ident.as_ref().clone() {
                            Statement::Identifier { ident } => {
                                ident
                            }
                            _ => return Err(CompilerError::new("Expected identifier expression but AST provided an illegal expression."))
                        };

                        (type_name, mods)
                    }
                    Statement::Void => {
                        ("void".to_string(), Vec::new())
                    }
                    _ => return Err(CompilerError::new("Expected type expression but AST provided an illegal expression."))
                };

                let rt = format!("{}{}", type_name, type_mods.join(""));

                let mut c_params = Vec::new();
                for p in params {
                    if let Statement::Declaration {
                        value, ..
                    } = p {
                        if value.is_some() {
                            // todo: handle default values here
                        }
                    }
                    c_params.push(p.gen_c_code()?);
                }
                let c_params_str = c_params.join(", ");

                let body = body.gen_c_code()?;

                Ok(format!("{} {}({}) {{\n{}}}\n", rt, name, c_params_str, body))
            }
            Statement::FnCall { ident, params } => {
                let name = match ident.as_ref().clone() {
                    Statement::Identifier { ident } => {
                        ident
                    }
                    _ => return Err(CompilerError::new("Expected identifier expression but AST provided an illegal expression."))
                };

                let mut c_params = Vec::new();
                for p in params {
                    c_params.push(p.gen_c_code()?);
                }
                let c_params_str = c_params.join(", ");

                Ok(format!("{}({})", name, c_params_str))
            }
            Statement::If { condition, body, else_statement } => {
                let cond = condition.gen_c_code()?;
                let body = body.gen_c_code()?;
                Ok(if else_statement.is_some() {
                    let else_body = else_statement.as_mut().unwrap().gen_c_code()?;
                    format!("if ({}) {} else {}", cond, body, else_body)
                } else {
                    format!("if ({}) {}", cond, body)
                })
            }
            Statement::Return { value } => {
                let val = value.gen_c_code()?;
                Ok(format!("return {}", val))
            }
            Statement::Panic { .. } => {
                todo!()
            }
            Statement::Assert { .. } => {
                todo!()
            }
            Statement::Declaration { ident, type_ident, value } => {
                let ident = match ident.as_ref().clone() {
                    Statement::Identifier { ident } => {
                        ident
                    }
                    _ => return Err(CompilerError::new("Expected identifier expression but AST provided an illegal expression."))
                };

                let type_ident = type_ident.as_mut().unwrap().gen_c_code()?;

                Ok(if value.is_some() {
                    let val = value.as_mut().unwrap().gen_c_code()?;
                    format!("{} {} = {}", type_ident, ident, val)
                } else {
                    format!("{} {}", type_ident, ident)
                })
            }
            Statement::Assignment { ident, value } => {
                let ident = match ident.as_ref().clone() {
                    Statement::Identifier { ident } => {
                        ident
                    }
                    _ => return Err(CompilerError::new("Expected identifier expression but AST provided an illegal expression."))
                };

                let val = value.gen_c_code()?;
                Ok(format!("{} = {}", ident, val))
            }
            Statement::Postfix { postfix } => {
                let mut c_code = String::new();
                let mut operand_stack: Vec<ShuntedStackItem> = Vec::new();

                // values for verifying the types of operators allowed
                let mut boolean_mode = false;
                let mut algebra_mode = false;

                for ssi in postfix {
                    if ssi.is_operand() {
                        operand_stack.push(ssi);
                    } else {
                        let op = ssi.get_operator().unwrap();
                        if op.precedence().is_none() {
                            return Err(CompilerError::new(format!("Expected expression operator but found invalid operator `{}`.",
                                        op)));
                        }
                        if !boolean_mode && !algebra_mode {
                            if op.is_boolean() {
                                boolean_mode = true;
                            } else {
                                algebra_mode = true;
                            }
                        }

                        // todo(eric): handle operators
                    }
                }
                
                Ok(c_code)
            }
            Statement::PropertyAccess { expr, property } => {
                let expr = expr.gen_c_code()?;
                let property = property.gen_c_code()?;
                Ok(format!("{}.{}", expr, property))
            }
            Statement::ArrayAccess { ident, index } => {
                let ident = match ident.as_ref().clone() {
                    Statement::Identifier { ident } => {
                        ident
                    }
                    _ => return Err(CompilerError::new("Expected identifier expression but AST provided an illegal expression."))
                };

                let index = index.gen_c_code()?;
                Ok(format!("{}[{}]", ident, index))
            }
            Statement::Type { type_ident, modifiers } => {
                let mut code = type_ident.gen_c_code()?;
                for m in modifiers {
                    code.push_str(m.gen_c_code()?.as_str());
                }
                Ok(code)
            }
            Statement::Identifier { ident } => {
                Ok(ident.clone())
            }
            Statement::Void => {
                Ok("void".to_string())
            }
            Statement::NumberLiteral { value } => {
                Ok(value.to_string())
            }
            Statement::StringLiteral { value } => {
                Ok(format!("\"{}\"", value))
            }
            _ => {
                // todo(eric): implement all other patterns to convert to c
                Ok(format!("NOT_YET_IMPLEMENTED_BOULDER_FEATURE"))
            }
        }
    }
}

pub fn generate_c_code(ast: &mut Statement) -> Result<String, CompilerError> {
    ast.gen_c_code()
}