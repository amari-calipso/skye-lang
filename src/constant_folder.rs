use crate::{ast::{Ast, Bits, Expression, MacroBody, Statement}, ast_error, ast_note, astpos_note, tokens::TokenType};

pub struct ConstantFolder {
    pub errors: usize
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
            Expression::Call(callee, _, args) => {
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
                            Expression::SignedIntLiteral { value, .. } => {
                                *value = !*value;
                                *expr = inner;
                            }
                            Expression::UnsignedIntLiteral { value, .. } => {
                                *value = !*value;
                                *expr = inner;
                            }
                            _ => ()
                        }
                    }
                    // TODO: properly account for integer width
                    TokenType::Minus => {
                        match &mut inner {
                            Expression::SignedIntLiteral { value, bits, .. } => {
                                if *bits == Bits::Any && *value > -(i64::MIN as i128) {
                                    ast_error!(self, expr, "Cannot apply '-' operator to unsigned integer");
                                    ast_note!(expr, format!("This operation will overflow to {}", (*value as u64).wrapping_neg()).as_str());
                                } else {
                                    *value = -*value;
                                    *expr = inner;
                                }
                            }
                            Expression::UnsignedIntLiteral { value, .. } => {
                                ast_error!(self, expr, "Cannot apply '-' operator to unsigned integer");
                                ast_note!(expr, format!("This operation will overflow to {}", value.wrapping_neg()).as_str());
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

                // TODO
                match op.type_ {
                    TokenType::Plus => (), 
                    TokenType::Minus => (),
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
            Statement::TransparentBlock(body) |
            Statement::Namespace { body, .. } => {
                for statement in body {
                    ctx.run(|ctx| self.fold_statement(statement, ctx)).await;
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
                    ctx.run(|ctx| self.fold_expression(&mut variant.expr, ctx)).await;
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
            Statement::Switch { kw, expr, cases } => (), // TODO
            Statement::If { kw, condition, then_branch, else_branch } => (), // TODO
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