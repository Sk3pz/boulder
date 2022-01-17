use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use crate::error::CompilerError;
use crate::statement::Statement;
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
            Statement::Unary { expr, op, leading } => {
                Ok(if leading.clone() {
                    format!("{}{}", op.gen_c_code()?, expr.gen_c_code()?)
                } else {
                    format!("{}{}", expr.gen_c_code()?, op.gen_c_code()?)
                })
            }
            Statement::Binary { left, op, right } => {
                Ok(format!("{} {} {}", left.gen_c_code()?, op.gen_c_code()?, right.gen_c_code()?))
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
            _ => todo!()
        }
    }
}

pub fn generate_c_code(path: String, ast: &mut Statement) -> Result<(), CompilerError> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(Path::new(&path))
        .expect("Failed to open file");

    file.write_all(ast.gen_c_code()?.as_bytes())
    .expect("Failed to write to file");

    Ok(())
}