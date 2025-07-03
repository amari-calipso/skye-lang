use crate::{ast::{Ast, Bits, Expression, MacroBody, Statement}, ast_error, ast_note, astpos_note, parser::tokens::TokenType};

pub struct ConstantFolder {
    pub errors: usize
}

macro_rules! signed_op_for_bits {
    ($slf: expr, $left_value: expr, $right_value: expr, $expr: expr, $operator: tt, $op_type: ty) => {
        {
            if let Ok(right_value) = $right_value.try_into() {
                if let Some(value) = ($left_value as $op_type).$operator(right_value) {
                    value as i128
                } else {
                    ast_error!($slf, $expr, "Cannot perform overflowing operation");
                    0
                }
            } else {
                ast_error!($slf, $expr, "Cannot perform overflowing operation");
                0
            }    
        }
    };
}

macro_rules! unsigned_op_signed_for_bits {
    ($slf: expr, $left_value: expr, $right_value: expr, $expr: expr, $operator: tt, $op_type: ty) => {
        {
            if let Some(value) = ($left_value as u128).$operator($right_value) {
                if let Ok(value) = value.try_into() {
                    let _: $op_type = value;
                    value as u64
                } else {
                    ast_error!($slf, $expr, "Cannot perform overflowing operation");
                    0
                }
            } else {
                ast_error!($slf, $expr, "Cannot perform overflowing operation");
                0
            }
        }
    };
}

macro_rules! unsigned_op_for_bits {
    ($slf: expr, $left_value: expr, $right_value: expr, $expr: expr, $operator: tt, $op_type: ty) => {
        {
            if let Ok(right_value) = $right_value.try_into() {
                if let Some(value) = ($left_value as $op_type).$operator(right_value) {
                    value as u64
                } else {
                    ast_error!($slf, $expr, "Cannot perform overflowing operation");
                    0
                }
            } else {
                ast_error!($slf, $expr, "Cannot perform overflowing operation");
                0
            }  
        }
    };
}

impl ConstantFolder {
    pub fn new() -> Self {
        ConstantFolder { errors: 0 }
    }

    pub fn reset(&mut self) {
        self.errors = 0;
    }

    async fn fold_expression(&mut self, expr: &mut Expression, ctx: &mut reblessive::Stk) {
        match expr {
            Expression::Grouping(inner) |
            Expression::Get(inner, _) | 
            Expression::StaticGet(inner, ..) => {
                ctx.run(|ctx| self.fold_expression(inner, ctx)).await;
            }
            Expression::Assign { target: left, value: right, .. } | 
            Expression::Array { item: left, size: right, .. } => {
                ctx.run(|ctx| self.fold_expression(left, ctx)).await;
                ctx.run(|ctx| self.fold_expression(right, ctx)).await;
            }
            Expression::Slice { items, .. } |
            Expression::ArrayLiteral { items, .. } => {
                for item in items {
                    ctx.run(|ctx| self.fold_expression(item, ctx)).await;
                }
            }
            Expression::Call(callee, _, args, _) => {
                ctx.run(|ctx| self.fold_expression(callee, ctx)).await;

                for arg in args {
                    ctx.run(|ctx| self.fold_expression(arg, ctx)).await;
                }
            }
            Expression::CompoundLiteral { type_, fields, .. } => {
                ctx.run(|ctx| self.fold_expression(type_, ctx)).await;

                for field in fields {
                    ctx.run(|ctx| self.fold_expression(&mut field.expr, ctx)).await;
                }
            }
            Expression::FnPtr { return_type, params, .. } => {
                ctx.run(|ctx| self.fold_expression(return_type, ctx)).await;

                for param in params {
                    ctx.run(|ctx| self.fold_expression(&mut param.type_, ctx)).await;
                }
            }
            Expression::InMacro { inner, source } => {
                let old_errors = self.errors;
                ctx.run(|ctx| self.fold_expression(inner, ctx)).await;

                if self.errors != old_errors {
                    astpos_note!(source, "This error is a result of this macro expansion");
                }
            }
            Expression::MacroExpandedStatements { inner, source } => {
                let old_errors = self.errors;

                for statement in inner {
                    ctx.run(|ctx| self.fold_statement(statement, ctx)).await;
                }

                if self.errors != old_errors {
                    astpos_note!(source, "This error is a result of this macro expansion");
                }
            }
            Expression::Ternary { condition, then_expr, else_expr, .. } => {
                ctx.run(|ctx| self.fold_expression(condition, ctx)).await;
                ctx.run(|ctx| self.fold_expression(then_expr, ctx)).await;
                ctx.run(|ctx| self.fold_expression(else_expr, ctx)).await;

                let condition_inner = condition.get_inner();
                match condition_inner {
                    Expression::SignedIntLiteral { value, .. } => {
                        if value == 0 {
                            *expr = *else_expr.clone();
                        } else {
                            *expr = *then_expr.clone();
                        }
                    }
                    Expression::UnsignedIntLiteral { value, .. } => {
                        if value == 0 {
                            *expr = *else_expr.clone();
                        } else {
                            *expr = *then_expr.clone();
                        }
                    }
                    _ => ()
                }
            }
            Expression::Unary { op, expr: inner, is_prefix } => {
                ctx.run(|ctx| self.fold_expression(inner, ctx)).await;

                if !*is_prefix {
                    // only ++ and --, we don't care for constant folding
                    return;
                }

                let mut inner = inner.get_inner();

                match op.type_ {
                    TokenType::Plus => {
                        if matches!(inner, Expression::SignedIntLiteral { .. } | Expression::UnsignedIntLiteral { .. } | Expression::FloatLiteral { .. }) {
                            *expr = inner;
                        }
                    }
                    TokenType::Tilde => {
                        match &mut inner {
                            Expression::SignedIntLiteral { value, bits, .. } => {
                                *value = {
                                    match bits {
                                        Bits::B8  => (!(*value as  i8)) as i128,
                                        Bits::B16 => (!(*value as i16)) as i128,
                                        Bits::B32 => (!(*value as i32)) as i128,
                                        Bits::B64 => (!(*value as i64)) as i128,
                                        Bits::Any => {
                                            if *value > -(i64::MIN as i128) {
                                                (!(*value as u64)) as i128
                                            } else {
                                                (!(*value as i64)) as i128
                                            }
                                        }
                                        _ => unreachable!()
                                    }
                                };

                                *expr = inner;
                            }
                            Expression::UnsignedIntLiteral { value, bits, .. } => {
                                match bits {
                                    Bits::B8  => *value = (!(*value as  u8)) as u64,
                                    Bits::B16 => *value = (!(*value as u16)) as u64,
                                    Bits::B32 => *value = (!(*value as u32)) as u64,
                                    Bits::B64 => *value = !*value,
                                    Bits::Bsz => return, // if the type is usz, we cannot determine this at comptime
                                    _ => unreachable!()
                                }

                                *expr = inner;
                            }
                            _ => ()
                        }
                    }
                    TokenType::Minus => {
                        match &mut inner {
                            Expression::SignedIntLiteral { value, bits, .. } => {
                                if *bits == Bits::Any && *value > -(i64::MIN as i128) {
                                    ast_error!(self, expr, "Cannot apply '-' operator to unsigned integer");
                                    ast_note!(expr, "This operation will overflow");
                                } else {
                                    *value = -*value;
                                    *expr = inner;
                                }
                            }
                            Expression::UnsignedIntLiteral { .. } => {
                                ast_error!(self, expr, "Cannot apply '-' operator to unsigned integer");
                                ast_note!(expr, "This operation will overflow");
                            }
                            Expression::FloatLiteral { value, .. } => {
                                *value = -*value;
                                *expr = inner;
                            }
                            _ => ()
                        }
                    }
                    TokenType::Bang => {
                        match inner {
                            Expression::SignedIntLiteral { value, tok, .. } => {
                                *expr = Expression::UnsignedIntLiteral { 
                                    value: (value == 0) as u64, 
                                    tok: tok.clone(), 
                                    bits: Bits::B8 
                                };
                            }
                            Expression::UnsignedIntLiteral { value, tok, .. } => {
                                *expr = Expression::UnsignedIntLiteral { 
                                    value: (value == 0) as u64, 
                                    tok: tok.clone(), 
                                    bits: Bits::B8 
                                };
                            }
                            _ => ()
                        }
                    }
                    _ => ()
                }
            }
            Expression::Subscript { subscripted, args, .. } => {
                ctx.run(|ctx| self.fold_expression(subscripted, ctx)).await;

                for arg in args.iter_mut() {
                    ctx.run(|ctx| self.fold_expression(arg, ctx)).await;
                }

                if args.len() != 1 {
                    return;
                }

                let subscripted_inner = subscripted.get_inner();
                if !matches!(subscripted_inner, Expression::Slice { .. } | Expression::ArrayLiteral { .. }) {
                    return;
                }

                let arg_inner = args[0].get_inner();
                let index = {
                    match arg_inner {
                        Expression::SignedIntLiteral { value, .. } => {
                            if value < 0 {
                                ast_error!(self, args[0], "Array indices cannot be negative");
                                return;
                            }

                            value as usize
                        }
                        Expression::UnsignedIntLiteral { value, .. } => value as usize,
                        _ => return
                    }
                };

                if let Expression::Slice { items, .. } | Expression::ArrayLiteral { items, .. } = subscripted_inner {
                    if index > items.len() {
                        ast_error!(
                            self, args[0], 
                            format!(
                                "Index {} is out of bounds for length {}",
                                index, items.len()
                            ).as_str()
                        );

                        ast_note!(
                            subscripted,
                            format!("This collection has length {}", items.len()).as_str()
                        );
                        
                        return;
                    }

                    *expr = items[index].clone();
                } else {
                    unreachable!()
                }
            }
            Expression::Binary { left, op, right } => {
                ctx.run(|ctx| self.fold_expression(left, ctx)).await;
                ctx.run(|ctx| self.fold_expression(right, ctx)).await;

                let left_inner = left.get_inner();
                let right_inner = right.get_inner();

                match op.type_ {
                    TokenType::Plus => {
                        match left_inner {
                            Expression::SignedIntLiteral { value: left_value, bits: left_bits, tok } => {
                                match right_inner {
                                    Expression::SignedIntLiteral { value: right_value, .. } => {
                                        let value = 'value_block: {
                                            match left_bits {
                                                Bits::B8  => signed_op_for_bits!(self, left_value, right_value, expr, checked_add,  i8),
                                                Bits::B16 => signed_op_for_bits!(self, left_value, right_value, expr, checked_add, i16),
                                                Bits::B32 => signed_op_for_bits!(self, left_value, right_value, expr, checked_add, i32),
                                                Bits::B64 => signed_op_for_bits!(self, left_value, right_value, expr, checked_add, i64),
                                                Bits::Any => {
                                                    if left_value > -(i64::MIN as i128) {
                                                        if let Some(value) = (left_value as u128).checked_add_signed(right_value) {
                                                            let value: Result<u64, _> = value.try_into();
                                                            if let Ok(value) = value {
                                                                break 'value_block value as i128;
                                                            }
                                                        }
                                                        
                                                        ast_error!(self, expr, "Cannot perform overflowing operation");
                                                        0
                                                    } else {
                                                        signed_op_for_bits!(self, left_value, right_value, expr, checked_add, i64)
                                                    }
                                                }
                                                _ => unreachable!()
                                            }
                                        };

                                        if value > -(i64::MIN as i128) {
                                            *expr = Expression::UnsignedIntLiteral {
                                                value: value as u64, tok,
                                                bits: left_bits
                                            };
                                        } else {
                                            *expr = Expression::SignedIntLiteral {
                                                value, tok,
                                                bits: left_bits
                                            };
                                        }
                                    }
                                    Expression::UnsignedIntLiteral { value: right_value, .. } => {
                                        let value = {
                                            match left_bits {
                                                Bits::B8  => signed_op_for_bits!(self, left_value, right_value, expr, checked_add,  i8),
                                                Bits::B16 => signed_op_for_bits!(self, left_value, right_value, expr, checked_add, i16),
                                                Bits::B32 => signed_op_for_bits!(self, left_value, right_value, expr, checked_add, i32),
                                                Bits::B64 => signed_op_for_bits!(self, left_value, right_value, expr, checked_add, i64),
                                                Bits::Any => {
                                                    if left_value > -(i64::MIN as i128) {
                                                        if let Some(value) = (left_value as u64).checked_add(right_value) {
                                                            value as i128
                                                        } else {
                                                            ast_error!(self, expr, "Cannot perform overflowing operation");
                                                            0
                                                        }
                                                    } else {
                                                        signed_op_for_bits!(self, left_value, right_value, expr, checked_add, i64)
                                                    }
                                                }
                                                _ => unreachable!()
                                            }
                                        };

                                        if value > -(i64::MIN as i128) {
                                            *expr = Expression::UnsignedIntLiteral {
                                                value: value as u64, tok,
                                                bits: left_bits
                                            };
                                        } else {
                                            *expr = Expression::SignedIntLiteral {
                                                value, tok,
                                                bits: left_bits
                                            };
                                        }
                                    }
                                    _ => ()
                                }
                            }
                            Expression::UnsignedIntLiteral { value: left_value, bits: left_bits, tok } => {
                                match right_inner {
                                    Expression::SignedIntLiteral { value: right_value, .. } => {
                                        let value = {
                                            match left_bits {
                                                Bits::B8  => unsigned_op_signed_for_bits!(self, left_value, right_value, expr, checked_add_signed,  u8),
                                                Bits::B16 => unsigned_op_signed_for_bits!(self, left_value, right_value, expr, checked_add_signed, u16),
                                                Bits::B32 => unsigned_op_signed_for_bits!(self, left_value, right_value, expr, checked_add_signed, u32),
                                                Bits::B64 => unsigned_op_signed_for_bits!(self, left_value, right_value, expr, checked_add_signed, u64),
                                                Bits::Bsz => return, // if the type is usz, we cannot determine this at comptime
                                                _ => unreachable!()
                                            }
                                        };

                                        *expr = Expression::UnsignedIntLiteral {
                                            value, tok,
                                            bits: left_bits
                                        };
                                    }
                                    Expression::UnsignedIntLiteral { value: right_value, .. } => {
                                        let value = {
                                            match left_bits {
                                                Bits::B8  => unsigned_op_for_bits!(self, left_value, right_value, expr, checked_add,  u8),
                                                Bits::B16 => unsigned_op_for_bits!(self, left_value, right_value, expr, checked_add, u16),
                                                Bits::B32 => unsigned_op_for_bits!(self, left_value, right_value, expr, checked_add, u32),
                                                Bits::B64 => {
                                                    if let Some(value) = left_value.checked_add(right_value) {
                                                        value
                                                    } else {
                                                        ast_error!(self, expr, "Cannot perform overflowing operation");
                                                        0
                                                    }
                                                }
                                                Bits::Bsz => return, // if the type is usz, we cannot determine this at comptime
                                                _ => unreachable!()
                                            }
                                        };

                                        *expr = Expression::UnsignedIntLiteral {
                                            value, tok,
                                            bits: left_bits
                                        };
                                    }
                                    _ => ()
                                }
                            }
                            Expression::FloatLiteral { value: left_value, bits, tok, .. } => {
                                if let Expression::FloatLiteral { value: right_value, .. } = right_inner {
                                    *expr = Expression::FloatLiteral {
                                        value: left_value + right_value, 
                                        tok, bits
                                    };
                                }
                            }
                            _ => ()
                        }
                    }
                    TokenType::Minus => {
                        match left_inner {
                            Expression::SignedIntLiteral { value: left_value, bits: left_bits, tok } => {
                                match right_inner {
                                    Expression::SignedIntLiteral { value: right_value, .. } => {
                                        let value = {
                                            match left_bits {
                                                Bits::B8  => signed_op_for_bits!(self, left_value, right_value, expr, checked_sub,  i8),
                                                Bits::B16 => signed_op_for_bits!(self, left_value, right_value, expr, checked_sub, i16),
                                                Bits::B32 => signed_op_for_bits!(self, left_value, right_value, expr, checked_sub, i32),
                                                Bits::B64 => signed_op_for_bits!(self, left_value, right_value, expr, checked_sub, i64),
                                                Bits::Any => {
                                                    if left_value > -(i64::MIN as i128) {
                                                        // TODO: checked_sub_signed is unstable rust
                                                        
                                                        // if let Some(value) = (left_value as u128).checked_sub_signed(right_value) {
                                                        //     let value: Result<u64, _> = value.try_into();
                                                        //     if let Ok(value) = value {
                                                        //         break 'value_block value as i128;
                                                        //     }
                                                        // }
                                                        
                                                        // ast_error!(self, expr, "Cannot perform overflowing operation");
                                                        // 0
                                                        return;
                                                    } else {
                                                        signed_op_for_bits!(self, left_value, right_value, expr, checked_sub, i64)
                                                    }
                                                }
                                                _ => unreachable!()
                                            }
                                        };

                                        if value > -(i64::MIN as i128) {
                                            *expr = Expression::UnsignedIntLiteral {
                                                value: value as u64, tok,
                                                bits: left_bits
                                            };
                                        } else {
                                            *expr = Expression::SignedIntLiteral {
                                                value, tok,
                                                bits: left_bits
                                            };
                                        }
                                    }
                                    Expression::UnsignedIntLiteral { value: right_value, .. } => {
                                        let value = {
                                            match left_bits {
                                                Bits::B8  => signed_op_for_bits!(self, left_value, right_value, expr, checked_sub,  i8),
                                                Bits::B16 => signed_op_for_bits!(self, left_value, right_value, expr, checked_sub, i16),
                                                Bits::B32 => signed_op_for_bits!(self, left_value, right_value, expr, checked_sub, i32),
                                                Bits::B64 => signed_op_for_bits!(self, left_value, right_value, expr, checked_sub, i64),
                                                Bits::Any => {
                                                    if left_value > -(i64::MIN as i128) {
                                                        if let Some(value) = (left_value as u64).checked_sub(right_value) {
                                                            value as i128
                                                        } else {
                                                            ast_error!(self, expr, "Cannot perform overflowing operation");
                                                            0
                                                        }
                                                    } else {
                                                        signed_op_for_bits!(self, left_value, right_value, expr, checked_sub, i64)
                                                    }
                                                }
                                                _ => unreachable!()
                                            }
                                        };

                                        if value > -(i64::MIN as i128) {
                                            *expr = Expression::UnsignedIntLiteral {
                                                value: value as u64, tok,
                                                bits: left_bits
                                            };
                                        } else {
                                            *expr = Expression::SignedIntLiteral {
                                                value, tok,
                                                bits: left_bits
                                            };
                                        }
                                    }
                                    _ => ()
                                }
                            }
                            Expression::UnsignedIntLiteral { value: left_value, bits: left_bits, tok } => {
                                match right_inner {
                                    // TODO: checked_sub_signed is unstable rust

                                    // Expression::SignedIntLiteral { value: right_value, .. } => {
                                    //     let value = {
                                    //         match left_bits {
                                    //             Bits::B8  => unsigned_op_signed_for_bits!(self, left_value, right_value, expr, checked_sub_signed,  u8),
                                    //             Bits::B16 => unsigned_op_signed_for_bits!(self, left_value, right_value, expr, checked_sub_signed, u16),
                                    //             Bits::B32 => unsigned_op_signed_for_bits!(self, left_value, right_value, expr, checked_sub_signed, u32),
                                    //             Bits::B64 => unsigned_op_signed_for_bits!(self, left_value, right_value, expr, checked_sub_signed, u64),
                                    //             Bits::Bsz => return, // if the type is usz, we cannot determine this at comptime
                                    //             _ => unreachable!()
                                    //         }
                                    //     };

                                    //     *expr = Expression::UnsignedIntLiteral {
                                    //         value, tok,
                                    //         bits: left_bits
                                    //     };
                                    // }
                                    Expression::UnsignedIntLiteral { value: right_value, .. } => {
                                        let value = {
                                            match left_bits {
                                                Bits::B8  => unsigned_op_for_bits!(self, left_value, right_value, expr, checked_sub,  u8),
                                                Bits::B16 => unsigned_op_for_bits!(self, left_value, right_value, expr, checked_sub, u16),
                                                Bits::B32 => unsigned_op_for_bits!(self, left_value, right_value, expr, checked_sub, u32),
                                                Bits::B64 => {
                                                    if let Some(value) = left_value.checked_sub(right_value) {
                                                        value
                                                    } else {
                                                        ast_error!(self, expr, "Cannot perform overflowing operation");
                                                        0
                                                    }
                                                }
                                                Bits::Bsz => return, // if the type is usz, we cannot determine this at comptime
                                                _ => unreachable!()
                                            }
                                        };

                                        *expr = Expression::UnsignedIntLiteral {
                                            value, tok,
                                            bits: left_bits
                                        };
                                    }
                                    _ => ()
                                }
                            }
                            Expression::FloatLiteral { value: left_value, bits, tok, .. } => {
                                if let Expression::FloatLiteral { value: right_value, .. } = right_inner {
                                    *expr = Expression::FloatLiteral {
                                        value: left_value - right_value, 
                                        tok, bits
                                    };
                                }
                            }
                            _ => ()
                        }
                    }
                    TokenType::Slash => (),
                    TokenType::Star => (),
                    TokenType::Mod => (),
                    TokenType::ShiftLeft => (),
                    TokenType::ShiftRight => (),
                    TokenType::LogicOr => (),
                    TokenType::LogicAnd => (),
                    TokenType::BitwiseXor => (),
                    TokenType::BitwiseOr => (),
                    TokenType::BitwiseAnd => (),
                    TokenType::Greater => (),
                    TokenType::GreaterEqual => (),
                    TokenType::Less => (),
                    TokenType::LessEqual => (),
                    TokenType::EqualEqual => (),
                    TokenType::BangEqual => (),
                    _ => ()
                }
            }
            _ => ()
        }
    }

    async fn fold_statement(&mut self, stmt: &mut Statement, ctx: &mut reblessive::Stk) {
        match stmt {
            Statement::Expression(expr) |
            // TODO: we could store `use` constants and fetch variables for more powerful constant folding
            Statement::Use { use_expr: expr, .. } => { 
                ctx.run(|ctx| self.fold_expression(expr, ctx)).await;
            }
            Statement::Defer { statement, .. } => {
                ctx.run(|ctx| self.fold_statement(statement, ctx)).await;
            }
            Statement::While { condition, body, .. } |
            Statement::DoWhile { condition, body, .. } |
            Statement::Foreach { iterator: condition, body, .. } => {
                ctx.run(|ctx| self.fold_expression(condition, ctx)).await;
                ctx.run(|ctx| self.fold_statement(body, ctx)).await;
            }
            Statement::Block(_, body) | 
            Statement::Namespace { body, .. } => {
                for statement in body {
                    ctx.run(|ctx| self.fold_statement(statement, ctx)).await;
                }
            }
            Statement::ImportedBlock { statements, source } => {
                let old_errors = self.errors;

                for statement in statements {
                    ctx.run(|ctx| self.fold_statement(statement, ctx)).await;
                }

                if self.errors != old_errors {
                    astpos_note!(source, "The error(s) were a result of this import");
                }
            }
            Statement::Struct { fields, .. } |
            Statement::Union { fields, .. } => {
                for field in fields {
                    ctx.run(|ctx| self.fold_expression(&mut field.expr, ctx)).await;
                }
            }
            Statement::Return { value, .. } => {
                if let Some(value) = value {
                    ctx.run(|ctx| self.fold_expression(value, ctx)).await;
                }
            }
            Statement::Impl { object, declarations } => {
                ctx.run(|ctx| self.fold_expression(object, ctx)).await;

                for declaration in declarations {
                    ctx.run(|ctx| self.fold_statement(declaration, ctx)).await;
                }
            }
            Statement::Enum { kind_type, variants, .. } => {
                ctx.run(|ctx| self.fold_expression(kind_type, ctx)).await;

                for variant in variants {
                    ctx.run(|ctx| self.fold_expression(&mut variant.type_, ctx)).await;
                }
            }
            Statement::VarDecl { initializer, type_, .. } => {
                if let Some(initializer) = initializer {
                    ctx.run(|ctx| self.fold_expression(initializer, ctx)).await;
                }

                if let Some(type_) = type_ {
                    ctx.run(|ctx| self.fold_expression(type_, ctx)).await;
                }
            }
            Statement::Macro { body, .. } => {
                match body {
                    MacroBody::Binding(expression) |
                    MacroBody::Expression(expression) => {
                        ctx.run(|ctx| self.fold_expression(expression, ctx)).await;
                    }
                    MacroBody::Block(statements) => {
                        for statement in statements {
                            ctx.run(|ctx| self.fold_statement(statement, ctx)).await;
                        }
                    }
                }
            }
            Statement::Template { declaration, generics, .. } => {
                ctx.run(|ctx| self.fold_statement(declaration, ctx)).await;

                for generic in generics {
                    if let Some(bounds) = &mut generic.bounds {
                        ctx.run(|ctx| self.fold_expression(bounds, ctx)).await;
                    }

                    if let Some(default) = &mut generic.default {
                        ctx.run(|ctx| self.fold_expression(default, ctx)).await;
                    }   
                }
            }
            Statement::For { initializer, condition, increments, body, .. } => {
                ctx.run(|ctx| self.fold_expression(condition, ctx)).await;
                ctx.run(|ctx| self.fold_statement(body, ctx)).await;

                if let Some(initializer) = initializer {
                    ctx.run(|ctx| self.fold_statement(initializer, ctx)).await;
                }

                for increment in increments {
                    ctx.run(|ctx| self.fold_expression(increment, ctx)).await;
                }
            }
            Statement::Function { params, return_type, body, .. } => {
                ctx.run(|ctx| self.fold_expression(return_type, ctx)).await;

                for param in params {
                    ctx.run(|ctx| self.fold_expression(&mut param.type_, ctx)).await;
                }

                if let Some(body) = body {
                    for statement in body {
                        ctx.run(|ctx| self.fold_statement(statement, ctx)).await;
                    }
                }
            }
            Statement::Interface { declarations, types, .. } => {
                if let Some(declarations) = declarations {
                    for declaration in declarations {
                        ctx.run(|ctx| self.fold_statement(declaration, ctx)).await;
                    }
                }

                if let Some(types) = types {
                    for type_ in types {
                        ctx.run(|ctx| self.fold_expression(type_, ctx)).await;
                    }
                }
            }
            Statement::If { condition, then_branch, else_branch, .. } => {
                ctx.run(|ctx| self.fold_expression(condition, ctx)).await;
                ctx.run(|ctx| self.fold_statement(then_branch, ctx)).await;

                if let Some(else_branch) = else_branch {
                    ctx.run(|ctx| self.fold_statement(else_branch, ctx)).await;
                }

                let condition_inner = condition.get_inner();
                match condition_inner {
                    Expression::SignedIntLiteral { value, .. } => {
                        if value == 0 {
                            if let Some(else_branch) = else_branch {
                                *stmt = *else_branch.clone();
                            } else {
                                *stmt = Statement::Empty;
                            }
                        } else {
                            *stmt = *then_branch.clone();
                        }
                    }
                    Expression::UnsignedIntLiteral { value, .. } => {
                        if value == 0 {
                            if let Some(else_branch) = else_branch {
                                *stmt = *else_branch.clone();
                            } else {
                                *stmt = Statement::Empty;
                            }
                        } else {
                            *stmt = *then_branch.clone();
                        }
                    }
                    _ => ()
                }
            }
            Statement::Switch { expr, cases, .. } => {
                ctx.run(|ctx| self.fold_expression(expr, ctx)).await;

                for branch in cases {
                    if let Some(cases) = &mut branch.cases {
                        for case in cases {
                            ctx.run(|ctx| self.fold_expression(case, ctx)).await;
                        }
                    }

                    for statement in branch.code.iter_mut() {
                        ctx.run(|ctx| self.fold_statement(statement, ctx)).await;
                    }
                }

                // TODO: do constant folding here
            }
            _ => ()
        }
    }

    pub fn fold(&mut self, statements: &mut Vec<Statement>) {
        let mut stack = reblessive::Stack::new();

        for statement in statements {
            stack.enter(|ctx| self.fold_statement(statement, ctx)).finish();
        }
    }
}