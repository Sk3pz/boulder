use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use crate::error::CompilerError;
use crate::expression::Expression;
use crate::operator::Operator;

impl Operator {
    pub fn gen_c_code(&mut self) -> Result<String, CompilerError> {
        Ok(match self {
            Operator::Add => String::from("+"),
            Operator::Sub => String::from("-"),
            Operator::Mul => String::from("*"),
            Operator::Div => String::from("+"),
            Operator::Mod => String::from("%"),
            Operator::Xor => String::from("^"),
            Operator::And => String::from("&"),
            Operator::Or => String::from("|"),
            Operator::Assign => String::from("="),
            Operator::Eq => String::from("=="),
            Operator::Neq => String::from("!="),
            Operator::Lt => String::from("<"),
            Operator::Lte => String::from("<="),
            Operator::Gt => String::from(">"),
            Operator::Gte => String::from(">="),
            Operator::BoolAnd => String::from("&&"),
            Operator::BoolOr => String::from("||"),
            Operator::Not => String::from("!"),
            Operator::MulAssign => String::from("*="),
            Operator::DivAssign => String::from("/="),
            Operator::ModAssign => String::from("%="),
            Operator::AddAssign => String::from("+="),
            Operator::SubAssign => String::from("-="),
            Operator::XorAssign => String::from("^="),
            Operator::AndAssign => String::from("&="),
            Operator::OrAssign => String::from("|="),
            Operator::Shl => String::from("<<"),
            Operator::Shr => String::from(">>"),
            Operator::ShlAssign => String::from("<<="),
            Operator::ShrAssign => String::from(">>="),
            Operator::Shlu => String::from("<<<"),
            Operator::Shru => String::from(">>>"),
            Operator::Move => String::from("->"),
            Operator::Inc => String::from("++"),
            Operator::Dec => String::from("--"),
            Operator::Right => String::from("=>"),
            Operator::Range => String::from(".."),
            Operator::IRange => String::from("..+"),
        })
    }
}

impl Expression {

    fn gen_c_code(&mut self) -> Result<String, CompilerError> {
        match self {
            Expression::Program { exprs } => {
                let mut code = String::from("#include \"stdio.h\"\n");
                for expr in exprs {
                    code.push_str(&expr.gen_c_code()?);
                    code.push('\n');
                }
                Ok(code)
            }
            Expression::Block { exprs } => {
                let mut code = String::new();
                for expr in exprs {
                    code.push_str(&expr.gen_c_code()?);
                    code.push(';');
                    code.push('\n');
                }
                Ok(code)
            }
            Expression::Use { exprs } => {
                let mut code = String::new();
                for expr in exprs {
                    code.push_str(&expr.gen_c_code()?);
                    code.push('\n');
                }
                Ok(code)
            }
            Expression::Fn { ident, params, return_type, body } => {
                let name = match ident.as_ref().clone() {
                    Expression::Identifier { ident } => {
                        ident
                    }
                    _ => return Err(CompilerError::new("Expected identifier expression but AST provided an illegal expression."))
                };

                let (type_name, type_mods) = match return_type.as_ref().clone() {
                    Expression::Type { modifiers, type_ident } => {
                        let mut mods = Vec::new();
                        for m in modifiers {
                            let c = m.clone().gen_c_code()?;
                            mods.push(c);
                        }

                        let type_name = match type_ident.as_ref().clone() {
                            Expression::Identifier { ident } => {
                                ident
                            }
                            _ => return Err(CompilerError::new("Expected identifier expression but AST provided an illegal expression."))
                        };

                        (type_name, mods)
                    }
                    Expression::Void => {
                        ("void".to_string(), Vec::new())
                    }
                    _ => return Err(CompilerError::new("Expected type expression but AST provided an illegal expression."))
                };

                let rt = format!("{}{}", type_name, type_mods.join(""));

                let mut c_params = Vec::new();
                for p in params {
                    c_params.push(p.gen_c_code()?);
                }
                let c_params_str = c_params.join(", ");

                let body = body.gen_c_code()?;

                Ok(format!("{} {}({}) {{\n{}}}\n", rt, name, c_params_str, body))
            }
            Expression::FnCall { ident, params } => {
                let name = match ident.as_ref().clone() {
                    Expression::Identifier { ident } => {
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
            Expression::If { condition, body, else_statement } => {
                let cond = condition.gen_c_code()?;
                let body = body.gen_c_code()?;
                Ok(if else_statement.is_some() {
                    let else_body = else_statement.as_mut().unwrap().gen_c_code()?;
                    format!("if ({}) {} else {}", cond, body, else_body)
                } else {
                    format!("if ({}) {}", cond, body)
                })
            }
            Expression::Return { value } => {
                let val = value.gen_c_code()?;
                Ok(format!("return {}", val))
            }
            Expression::Unary { expr, op, leading } => {
                Ok(if leading.clone() {
                    format!("{}{}", op.gen_c_code()?, expr.gen_c_code()?)
                } else {
                    format!("{}{}", expr.gen_c_code()?, op.gen_c_code()?)
                })
            }
            Expression::Binary { left, op, right } => {
                Ok(format!("{} {} {}", left.gen_c_code()?, op.gen_c_code()?, right.gen_c_code()?))
            }
            Expression::Panic { value } => {
                todo!()
            }
            Expression::Assert { expr } => {
                todo!()
            }
            Expression::Declaration { ident, type_ident, value } => {
                let ident = match ident.as_ref().clone() {
                    Expression::Identifier { ident } => {
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
            Expression::Assignment { ident, value } => {
                let ident = match ident.as_ref().clone() {
                    Expression::Identifier { ident } => {
                        ident
                    }
                    _ => return Err(CompilerError::new("Expected identifier expression but AST provided an illegal expression."))
                };

                let val = value.gen_c_code()?;
                Ok(format!("{} = {}", ident, val))
            }
            Expression::PropertyAccess { expr, property } => {
                let expr = expr.gen_c_code()?;
                let property = property.gen_c_code()?;
                Ok(format!("{}.{}", expr, property))
            }
            Expression::ArrayAccess { ident, index } => {
                let ident = match ident.as_ref().clone() {
                    Expression::Identifier { ident } => {
                        ident
                    }
                    _ => return Err(CompilerError::new("Expected identifier expression but AST provided an illegal expression."))
                };

                let index = index.gen_c_code()?;
                Ok(format!("{}[{}]", ident, index))
            }
            Expression::Type { type_ident, modifiers } => {
                let mut code = type_ident.gen_c_code()?;
                for m in modifiers {
                    code.push_str(m.gen_c_code()?.as_str());
                }
                Ok(code)
            }
            Expression::Identifier { ident } => {
                Ok(ident.clone())
            }
            Expression::Void => {
                Ok("void".to_string())
            }
            Expression::NumberLiteral { value } => {
                Ok(value.to_string())
            }
            Expression::StringLiteral { value } => {
                Ok(format!("\"{}\"", value))
            }
            _ => todo!()
        }
    }
}

pub fn generate_c_code(path: String, ast: &mut Expression) -> Result<(), CompilerError> {
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