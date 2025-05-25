use std::{collections::HashMap, rc::Rc};

use crate::{ast::{Ast, Expression, LiteralKind, MacroBody, MacroParams, Statement}, ast_error, ast_note, ast_warning, codegen, skye_type::{GetResult, SkyeType}, token_error, tokens::{Token, TokenType}, utils::escape_string, CompileMode};

pub struct MacroExpander {
    globals: HashMap<Rc<str>, SkyeType>,

    curr_name:    String,
    in_impl:      bool,
    in_interface: bool,
    compile_mode: CompileMode,

    pub errors: usize,
}

macro_rules! at_operator {
    ($slf: ident, $inner: expr, $expr: ident, $op: expr, $pos: expr, $ctx: expr) => {
        if let Some(SkyeType::Type(inner)) = $inner {
            if let SkyeType::Macro(_, params, _) = &*inner {
                if !matches!(params, MacroParams::None) {
                    return Some(*inner);
                }

                if let SkyeType::Macro(_, _, body) = *inner {
                    match body {
                        MacroBody::Expression(expression) => {
                            *$expr = Expression::InMacro {
                                inner: Box::new(expression),
                                source: $pos
                            };
                        }
                        MacroBody::Block(statements) => {
                            *$expr = Expression::MacroExpandedStatements {
                                inner: statements,
                                source: $pos
                            };
                        }
                        MacroBody::Binding(_) => () // ignore macro bindings, they are resolved at codegen time
                    }
                } else {
                    unreachable!()
                }

                // re-expand the expression to expand potential nested macros and return values where needed
                return $ctx.run(|ctx| $slf.expand_expression($expr, ctx)).await;
            }

            token_error!(
                $slf, $op,
                format!(
                    "'@' can only be used on macros (got {})",
                    inner.stringify_native()
                ).as_ref()
            );
        }
    };
}

impl MacroExpander {
    pub fn new(compile_mode: CompileMode) -> Self {
        let mut globals = HashMap::new();
        globals.insert(
            Rc::from("COMPILE_MODE"),
            SkyeType::Type(
                Box::new(SkyeType::Macro(
                    Rc::from("COMPILE_MODE"),
                    MacroParams::None,
                    MacroBody::Expression({
                        let lit = {
                            match compile_mode {
                                CompileMode::Debug         => "0",
                                CompileMode::Release       => "1",
                                CompileMode::ReleaseUnsafe => "2"
                            }
                        };

                        Expression::Literal { value: Rc::from(lit), tok: Token::dummy(Rc::from("")), kind: LiteralKind::U8 }
                    })
                ))
            )
        );


        MacroExpander {
            globals, compile_mode,
            curr_name: String::new(),
            in_impl: false,
            in_interface: false,
            errors: 0
        }
    }

    fn get_name(&self, name: &Rc<str>) -> Rc<str> {
        if self.curr_name == "" {
            Rc::clone(&name)
        } else {
            Rc::from(format!("{}_DOT_{}", self.curr_name, name))
        }
    }

    fn handle_builtin_macros(&mut self, macro_name: &Rc<str>, arguments: &Vec<Expression>, callee_expr: &Expression) -> Option<Expression> {
        match macro_name.as_ref() {
            "concat" => {
                if arguments.len() == 1 {
                    if let Expression::Literal { value, tok, .. } = arguments[0].get_inner() {
                        ast_warning!(arguments[0], "@concat macro is being used with no effect"); // +W-useless-concat
                        ast_note!(callee_expr, "The @concat macro is used to concatenate multiple values together as a string. Calling it with one argument is unnecessary");
                        ast_note!(callee_expr, "Remove this macro call");
                        Some(Expression::Literal { value, tok, kind: LiteralKind::String })
                    } else {
                        ast_error!(self, arguments[0], "Argument for @concat macro must be a literal");
                        ast_note!(arguments[0], "The value must be known at compile time");
                        Some(Expression::Literal { value: Rc::from(""), tok: Token::dummy(Rc::from("")), kind: LiteralKind::String })
                    }
                } else {
                    let mut result = String::new();

                    for argument in arguments {
                        if let Expression::Literal { value, .. } = argument.get_inner() {
                            result.push_str(&value);
                        } else {
                            ast_error!(self, argument, "Argument for @concat macro must be a literal");
                            ast_note!(argument, "The value must be known at compile time");
                        }
                    }

                    let pos = callee_expr.get_pos();
                    let lexeme = Rc::from(result.as_ref());
                    let tok = Token::new(pos.source, pos.filename, TokenType::String, Rc::clone(&lexeme), pos.start, pos.end, pos.line);
                    Some(Expression::Literal { value: Rc::clone(&lexeme), tok, kind: LiteralKind::String })
                }
            }
            _ => None
        }
    }

    async fn expand_expression(&mut self, expr: &mut Expression, ctx: &mut reblessive::Stk) -> Option<SkyeType> {
        let expr_pos = expr.get_pos();

        match expr {
            Expression::Literal { .. } => (),
            Expression::Binary { left, right, .. } |
            Expression::Assign { target: left, value: right, .. } => {
                ctx.run(|ctx| self.expand_expression(left, ctx)).await;
                ctx.run(|ctx| self.expand_expression(right, ctx)).await;
            }
            Expression::Grouping(expr) |
            Expression::Get(expr, _) |
            Expression::InMacro { inner: expr, .. } => {
                return ctx.run(|ctx| self.expand_expression(expr, ctx)).await;
            }
            Expression::Variable(name) => {
                let value = self.globals.get(&name.lexeme).cloned();

                if let Some(value) = &value {
                    if let SkyeType::Type(inner) = value {
                        if let SkyeType::Macro(name, ..) = &**inner {
                            if codegen::BUILTIN_MACROS.contains(name.as_ref()) {
                                return None;
                            }
                        }
                    }
                }

                return value;
            }
            Expression::MacroExpandedStatements { inner, .. } => {
                for statement in inner {
                    ctx.run(|ctx| self.expand_statement(statement, ctx)).await;
                }
            }
            Expression::FnPtr { return_type, params, .. } => {
                ctx.run(|ctx| self.expand_expression(return_type, ctx)).await;

                for param in params {
                    ctx.run(|ctx| self.expand_expression(&mut param.type_, ctx)).await;
                }
            }
            Expression::Ternary { condition, then_expr, else_expr, .. } => {
                ctx.run(|ctx| self.expand_expression(condition, ctx)).await;
                ctx.run(|ctx| self.expand_expression(then_expr, ctx)).await;
                ctx.run(|ctx| self.expand_expression(else_expr, ctx)).await;
            }
            Expression::CompoundLiteral { type_, fields, .. } => {
                ctx.run(|ctx| self.expand_expression(type_, ctx)).await;

                for field in fields {
                    ctx.run(|ctx| self.expand_expression(&mut field.expr, ctx)).await;
                }
            }
            Expression::Subscript { subscripted, args, .. } => {
                ctx.run(|ctx| self.expand_expression(subscripted, ctx)).await;

                for arg in args {
                    ctx.run(|ctx| self.expand_expression(arg, ctx)).await;
                }
            }
            Expression::Slice { items, .. } => {
                for item in items {
                    ctx.run(|ctx| self.expand_expression(item, ctx)).await;
                }
            }
            Expression::Unary { expr: inner_expr, op, is_prefix } => {
                let inner = ctx.run(|ctx| self.expand_expression(inner_expr, ctx)).await;
                if *is_prefix && op.type_ == TokenType::At {
                    at_operator!(self, inner, expr, op, expr_pos, ctx);
                }
            }
            Expression::StaticGet(object_expr, name, gets_macro) => {
                let object = ctx.run(|ctx| self.expand_expression(object_expr, ctx)).await;

                if let Some(object) = object {
                    match object.static_get(&name) {
                        GetResult::Ok(value, ..) => {
                            if let Some(var) = self.globals.get(&value) {
                                if *gets_macro {
                                    at_operator!(self, Some(var.clone()), expr, name, expr_pos, ctx);
                                }

                                return Some(var.clone());
                            }
                        }
                        GetResult::InvalidType => {
                            ast_error!(
                                self, object_expr,
                                format!(
                                    "Can only statically access namespaces, structs, enums and instances (got {})",
                                    object.stringify_native()
                                ).as_ref()
                            );
                        }
                        GetResult::FieldNotFound => unreachable!()
                    }
                }
            }
            Expression::Call(callee_expr, _, args) => {
                let callee = ctx.run(|ctx| self.expand_expression(callee_expr, ctx)).await;

                for arg in args.iter_mut() {
                    ctx.run(|ctx| self.expand_expression(arg, ctx)).await;
                }

                if let Some(SkyeType::Macro(name, params, body)) = callee {
                    assert!(!matches!(params, MacroParams::None)); // covered by unary '@' evaluation

                    match &params {
                        MacroParams::Some(params) => {
                            if params.len() != args.len() {
                                ast_error!(
                                    self, callee_expr,
                                    format!(
                                        "Expecting {} arguments for macro call but got {}",
                                        params.len(), args.len()
                                    ).as_str()
                                );

                                return None;
                            }
                        }
                        MacroParams::OneN(_) => {
                            if args.len() == 0 {
                                ast_error!(self, expr, "Expecting at least one argument for macro call but got none");
                                return None;
                            }
                        }
                        _ => ()
                    }

                    if let Some(expression) = self.handle_builtin_macros(&name, &args, &callee_expr) {
                        *expr = Expression::InMacro {
                            inner: Box::new(expression),
                            source: expr_pos
                        };

                        // re-expand the expression to expand potential nested macros and return values where needed
                        return ctx.run(|ctx| self.expand_expression(expr, ctx)).await;
                    }

                    match body {
                        MacroBody::Expression(mut return_expr) => {
                            match params {
                                MacroParams::Some(params) => {
                                    for i in 0 .. args.len() {
                                        return_expr = return_expr.replace_variable(&params[i].lexeme, &args[i]);
                                    }

                                    if name.as_ref() == "panic" {
                                        // panic also includes position information

                                        if matches!(self.compile_mode, CompileMode::Debug) {
                                            let panic_pos = callee_expr.get_pos();

                                            return_expr = return_expr.replace_variable(
                                                &Rc::from("PANIC_POS"),
                                                &Expression::Literal { value: Rc::from(format!(
                                                    "{}: line {}, pos {}",
                                                    escape_string(&panic_pos.filename), panic_pos.line + 1, panic_pos.start
                                                )), tok: Token::dummy(Rc::from("")), kind: LiteralKind::String }
                                            );
                                        } else {
                                            return_expr = return_expr.replace_variable(
                                                &Rc::from("PANIC_POS"),
                                                &Expression::Literal { value: Rc::from(""), tok: Token::dummy(Rc::from("")), kind: LiteralKind::String }
                                            );
                                        }
                                    }
                                }
                                MacroParams::OneN(var_name) | MacroParams::ZeroN(var_name) => {
                                    return_expr = return_expr.replace_variable(
                                        &var_name.lexeme,
                                        &Expression::Slice { opening_brace: var_name.clone(), items: args.clone() }
                                    );
                                }
                                MacroParams::None => unreachable!()
                            }

                            *expr = Expression::InMacro {
                                inner: Box::new(return_expr),
                                source: expr_pos
                            };

                            // re-expand the expression to expand potential nested macros and return values where needed
                            return ctx.run(|ctx| self.expand_expression(expr, ctx)).await;
                        }
                        MacroBody::Block(mut body) => {
                            for statement in body.iter_mut() {
                                match &params {
                                    MacroParams::Some(params) => {
                                        for i in 0 .. args.len() {
                                            *statement = statement.replace_variable(&params[i].lexeme, &args[i]);
                                        }
                                    }
                                    MacroParams::OneN(var_name) | MacroParams::ZeroN(var_name) => {
                                        *statement = statement.replace_variable(
                                            &var_name.lexeme,
                                            &Expression::Slice { opening_brace: var_name.clone(), items: args.clone() }
                                        );
                                    }
                                    MacroParams::None => unreachable!()
                                }
                            }

                            *expr = Expression::MacroExpandedStatements {
                                inner: body,
                                source: expr_pos
                            };

                            // re-expand the expression to expand potential nested macros
                            ctx.run(|ctx| self.expand_expression(expr, ctx)).await;
                        }
                        MacroBody::Binding(_) => () // ignore macro bindings, they are resolved at codegen time
                    }
                }
            }
        }

        None
    }

    async fn expand_statement(&mut self, stmt: &mut Statement, ctx: &mut reblessive::Stk) {
        match stmt {
            Statement::Expression(expression) => {
                ctx.run(|ctx| self.expand_expression(expression, ctx)).await;
            }
            Statement::Block(_, statements) | Statement::TransparentBlock(statements) => {
                for statement in statements {
                    ctx.run(|ctx| self.expand_statement(statement, ctx)).await;
                }
            }
            Statement::While { condition, body, .. } |
            Statement::DoWhile { condition, body, .. } |
            Statement::Foreach { iterator: condition, body, .. } => {
                ctx.run(|ctx| self.expand_expression(condition, ctx)).await;
                ctx.run(|ctx| self.expand_statement(body, ctx)).await;
            }
            Statement::Return { value, .. } => {
                if let Some(value) = value {
                    ctx.run(|ctx| self.expand_expression(value, ctx)).await;
                }
            }

            Statement::Defer { statement, .. } => {
                ctx.run(|ctx| self.expand_statement(statement, ctx)).await;
            }
            Statement::VarDecl { initializer, type_, .. } => {
                if let Some(initializer) = initializer {
                    ctx.run(|ctx| self.expand_expression(initializer, ctx)).await;
                }

                if let Some(type_) = type_ {
                    ctx.run(|ctx| self.expand_expression(type_, ctx)).await;
                }
            }
            Statement::If { condition, then_branch, else_branch, .. } => {
                ctx.run(|ctx| self.expand_expression(condition, ctx)).await;
                ctx.run(|ctx| self.expand_statement(then_branch, ctx)).await;

                if let Some(else_branch) = else_branch {
                    ctx.run(|ctx| self.expand_statement(else_branch, ctx)).await;
                }
            }
            Statement::For { initializer, condition, increments, body, .. } => {
                ctx.run(|ctx| self.expand_expression(condition, ctx)).await;
                ctx.run(|ctx| self.expand_statement(body, ctx)).await;

                if let Some(initializer) = initializer {
                    ctx.run(|ctx| self.expand_statement(initializer, ctx)).await;
                }

                for increment in increments {
                    ctx.run(|ctx| self.expand_expression(increment, ctx)).await;
                }
            }
            Statement::Function { params, return_type, body, .. } => {
                ctx.run(|ctx| self.expand_expression(return_type, ctx)).await;

                for param in params {
                    ctx.run(|ctx| self.expand_expression(&mut param.type_, ctx)).await;
                }

                if let Some(body) = body {
                    for statement in body {
                        ctx.run(|ctx| self.expand_statement(statement, ctx)).await;
                    }
                }
            }
            Statement::Template { declaration, generics, .. } => {
                ctx.run(|ctx| self.expand_statement(declaration, ctx)).await;

                for generic in generics {
                    if let Some(bounds) = &mut generic.bounds {
                        ctx.run(|ctx| self.expand_expression(bounds, ctx)).await;
                    }

                    if let Some(default) = &mut generic.default {
                        ctx.run(|ctx| self.expand_expression(default, ctx)).await;
                    }
                }
            }
            Statement::Interface { declarations, types, .. } => {
                if let Some(types) = types {
                    for type_ in types {
                        ctx.run(|ctx| self.expand_expression(type_, ctx)).await;
                    }
                }

                if let Some(declarations) = declarations {
                    let old_in_interface = self.in_interface;
                    self.in_interface = true;

                    for declaration in declarations {
                        ctx.run(|ctx| self.expand_statement(declaration, ctx)).await;
                    }

                    self.in_interface = old_in_interface;
                }
            }
            Statement::Impl { object, declarations } => {
                ctx.run(|ctx| self.expand_expression(object, ctx)).await;

                let old_in_impl = self.in_impl;
                self.in_impl = true;

                for declaration in declarations {
                    ctx.run(|ctx| self.expand_statement(declaration, ctx)).await;
                }

                self.in_impl = old_in_impl;
            }
            Statement::Switch { expr, cases, .. } => {
                ctx.run(|ctx| self.expand_expression(expr, ctx)).await;

                for branch in cases {
                    for statement in branch.code.iter_mut() {
                        ctx.run(|ctx| self.expand_statement(statement, ctx)).await;
                    }

                    if let Some(cases) = &mut branch.cases {
                        for case in cases {
                            ctx.run(|ctx| self.expand_expression(case, ctx)).await;
                        }
                    }
                }
            }
            Statement::Use { use_expr, as_name, .. } => {
                let value = ctx.run(|ctx| self.expand_expression(use_expr, ctx)).await;

                if let Some(value) = value {
                    self.globals.insert(Rc::clone(&as_name.lexeme), value);
                }
            }
            Statement::Namespace { name, body } => {
                let full_name = self.get_name(&name.lexeme);
                self.globals.insert(
                    Rc::clone(&full_name),
                    SkyeType::Namespace(Rc::clone(&full_name))
                );

                let previous_name = self.curr_name.clone();
                self.curr_name = full_name.to_string();

                for statement in body {
                    ctx.run(|ctx| self.expand_statement(statement, ctx)).await;
                }

                self.curr_name = previous_name;
            }
            Statement::Macro { name, params, body } => {
                if matches!(body, MacroBody::Binding(..)) { // ignore macro bindings, they are resolved at codegen time
                    return;
                }

                if self.in_impl {
                    token_error!(self, name, "Cannot declare macro in impl block");
                    return;
                }

                if self.in_interface {
                    token_error!(self, name, "Cannot declare macro inside an interface");
                    return;
                }

                let full_name = self.get_name(&name.lexeme);
                self.globals.insert(
                    Rc::clone(&full_name),
                    SkyeType::Type(Box::new(
                        SkyeType::Macro(
                            full_name,
                            params.clone(),
                            body.clone()
                        )
                    ))
                );
            }
            _ => ()
        }
    }

    pub fn expand(&mut self, statements: &mut Vec<Statement>) {
        let mut stack = reblessive::Stack::new();

        for statement in statements {
            stack.enter(|ctx| self.expand_statement(statement, ctx)).finish();
        }
    }
}