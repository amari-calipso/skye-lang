use std::{collections::HashMap, rc::Rc};

use alanglib::{error, report::note};

use crate::{
    ast::{Bits, EnumVariant, Expression, FunctionInfo, FunctionParam, Generic, ImportType, MacroBody, MacroParams, Statement, StaticGetTarget, StringKind, StructField, SwitchCase},
    tokens::{Token, TokenType}
};

macro_rules! match_number_literal {
    ($parser: expr, $type_: tt, $bits: tt, $expr_type: tt, $parse: expr) => {
        if $parser.match_(&[TokenType::$type_]) {
            let tok = $parser.previous().clone();
            return Some(Expression::$expr_type {
                value: {
                    if let Some(value) = ($parse)(&tok.lexeme) {
                        value
                    } else {
                        alanglib::error!($parser, tok, "Invalid integer literal");
                        Default::default()
                    }
                },
                tok, bits: Bits::$bits
            });
        }
    };
}

macro_rules! match_sint_literal {
    ($parser: expr, $type_: tt, $bits: tt) => {
        match_number_literal!($parser, $type_, $bits, SignedIntLiteral, |x: &str| {
            let result = {
                if x.starts_with("0x") {
                    i128::from_str_radix(&x[2..], 16).ok()?
                } else if x.starts_with("0o") {
                    i128::from_str_radix(&x[2..], 8).ok()?
                } else if x.starts_with("0b") {
                    i128::from_str_radix(&x[2..], 2).ok()?
                } else {
                    x.parse().ok()?
                }
            };

            if (result > 0 && matches!(TryInto::<u64>::try_into(result), Err(_))) || result < i64::MIN as i128 {
                None
            } else {
                Some(result)
            }
        })
    };
}

macro_rules! match_uint_literal {
    ($parser: expr, $type_: tt, $bits: tt) => {
        match_number_literal!($parser, $type_, $bits, UnsignedIntLiteral, |x: &str| {
            if x.starts_with("0x") {
                u64::from_str_radix(&x[2..], 16).ok()
            } else if x.starts_with("0o") {
                u64::from_str_radix(&x[2..], 8).ok()
            } else if x.starts_with("0b") {
                u64::from_str_radix(&x[2..], 2).ok()
            } else {
                x.parse().ok()
            }
        })
    };
}

macro_rules! match_float_literal {
    ($parser: expr, $type_: tt, $bits: tt) => {
        match_number_literal!(
            $parser, $type_, $bits, FloatLiteral, 
            |x: &str| x.parse().ok()
        )
    };
}

macro_rules! match_string_literal {
    ($parser: expr, $type_: tt, $kind: tt) => {
        if $parser.match_(&[TokenType::$type_]) {
            let prev = $parser.previous();
            return Some(Expression::StringLiteral {
                value: prev.lexeme.clone(),
                tok: prev.clone(),
                kind: StringKind::$kind
            });
        }
    };
}

macro_rules! left_associativity_binary {
    ($name: tt, $next: tt, $types: expr) => {
        fn $name(&mut self) -> Option<Expression> {
            let mut expr = self.$next()?;

            while self.match_($types) {
                let op = self.previous().clone();
                let right = self.$next()?;
                expr = Expression::Binary {
                    left: Box::new(expr),
                    op,
                    right: Box::new(right)
                };
            }

            Some(expr)
        }
    };
}

macro_rules! prefix_unary {
    ($name: tt, $next: tt, $types: expr) => {
        fn $name(&mut self) -> Option<Expression> {
            if self.match_($types) {
                let op = self.previous().clone();
                let right = self.$name()?;
                return Some(Expression::Unary {
                    op,
                    expr: Box::new(right),
                    is_prefix: true
                });
            }

            self.$next()
        }
    };
}

macro_rules! suffix_unary {
    ($name: tt, $next: tt, $types: expr) => {
        fn $name(&mut self) -> Option<Expression> {
            let expr = self.$next()?;

            if self.match_($types) {
                let op = self.previous().clone();
                return Some(Expression::Unary {
                    op,
                    expr: Box::new(expr),
                    is_prefix: false
                });
            }

            Some(expr)
        }
    };
}

pub struct Qualifier {
    pub name: Token,
    pub arg: Option<Token>
}

pub struct Parser {
    tokens: Vec<Token>,
    curr_qualifiers: HashMap<Rc<str>, Qualifier>,
    curr: usize,
    pub errors: usize
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, curr_qualifiers: HashMap::new(), curr: 0, errors: 0 }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.curr]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.curr - 1]
    }

    fn is_at_end(&self) -> bool {
        self.peek().type_ == TokenType::EOF
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.curr += 1;
        }

        self.previous()
    }

    fn check(&self, type_: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }

        self.peek().type_ == type_
    }

    fn match_(&mut self, types: &[TokenType]) -> bool {
        for type_ in types {
            if self.check(*type_) {
                self.advance();
                return true;
            }
        }

        false
    }

    fn syncronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().type_ == TokenType::Semicolon {
                return;
            }

            match self.peek().type_ {
                TokenType::Struct | TokenType::Fn | TokenType::If |
                TokenType::Return | TokenType::Let | TokenType::While |
                TokenType::Enum | TokenType::Import | TokenType::Defer |
                TokenType::Impl | TokenType::Namespace | TokenType::Switch |
                TokenType::Continue | TokenType::Break | TokenType::Do |
                TokenType::Macro | TokenType::Use => return,
                _ => ()
            }

            self.advance();
        }
    }

    fn consume(&mut self, type_: TokenType, msg: &str) -> Option<&Token> {
        if self.check(type_) {
            Some(self.advance())
        } else {
            let tok = self.peek();
            error!(self, tok, msg);
            None
        }
    }

    fn fn_ptr(&mut self) -> Option<Expression> {
        let kw = self.previous().clone();
        self.consume(TokenType::LeftParen, "Expecting '(' after 'fn' in function pointer type")?;

        let mut params = Vec::new();

        while !self.check(TokenType::RightParen) {
            let is_const = self.match_(&[TokenType::Const]);
            let type_ = self.expression()?;
            params.push(FunctionParam::new(None, type_, is_const));

            if !self.match_(&[TokenType::Comma]) {
                break;
            }
        }

        self.consume(TokenType::RightParen, "Expecting ')' after function parameters")?;

        let return_type = self.type_expression()?;
        Some(Expression::FnPtr { kw, return_type: Box::new(return_type), params })
    }

    fn get_nonzero_expressions(&mut self, type_: TokenType) -> Option<Vec<Expression>> {
        let mut expressions = Vec::new();

        while !self.check(type_) {
            expressions.push(self.expression()?);

            if !self.match_(&[TokenType::Comma]) {
                break;
            }
        }

        Some(expressions)
    }

    fn primary(&mut self) -> Option<Expression> {
        if self.match_(&[TokenType::Void]) {
            return Some(Expression::VoidLiteral(self.previous().clone()));
        }

        if self.match_(&[TokenType::Fn]) {
            return self.fn_ptr();
        }

        if self.match_(&[TokenType::Identifier]) {
            return Some(Expression::Variable(self.previous().clone()));
        }

        if self.match_(&[TokenType::Super]) {
            self.consume(TokenType::ColonColon, "Expecting '::' after 'super'")?;
            let gets_macro = self.match_(&[TokenType::At]);
            let name = self.consume(TokenType::Identifier, "Expecting identifier after ::")?.clone();
            return Some(Expression::StaticGet(StaticGetTarget::Super, name, gets_macro));
        }

        if self.match_(&[TokenType::ColonColon]) {
            let gets_macro = self.match_(&[TokenType::At]);
            let name = self.consume(TokenType::Identifier, "Expecting identifier after ::")?.clone();
            return Some(Expression::StaticGet(StaticGetTarget::Global, name, gets_macro));
        }

        if self.match_(&[TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expecting ')' after expression")?;
            return Some(Expression::Grouping(Box::new(expr)));
        }

        if self.match_(&[TokenType::LeftBrace]) {
            let opening_brace = self.previous().clone();
            let items = self.get_nonzero_expressions(TokenType::RightBrace)?;
            self.consume(TokenType::RightBrace, "Expecting '}' after slice")?;
            return Some(Expression::Slice { opening_brace, items });
        }

        if self.match_(&[TokenType::LeftSquare]) {
            let opening_brace = self.previous().clone();
            let first = self.expression()?;

            if self.match_(&[TokenType::Semicolon]) {
                let size = self.expression()?;
                self.consume(TokenType::RightSquare, "Expecting ']' after array declaration")?;
                return Some(Expression::Array { opening_brace, item: Box::new(first), size: Box::new(size) });
            } else {
                self.match_(&[TokenType::Comma]); // consumes comma after first expression, if any
                let mut items = self.get_expressions(TokenType::RightSquare)?;
                self.consume(TokenType::RightSquare, "Expecting ']' after array")?;
                items.insert(0, first);
                return Some(Expression::ArrayLiteral { opening_brace, items });
            }
        }

        match_sint_literal!(self,     I8,  B8);
        match_sint_literal!(self,    I16, B16);
        match_sint_literal!(self,    I32, B32);
        match_sint_literal!(self,    I64, B64);
        match_sint_literal!(self, AnyInt, Any);

        match_uint_literal!(self,  U8,  B8);
        match_uint_literal!(self, U16, B16);
        match_uint_literal!(self, U32, B32);
        match_uint_literal!(self, U64, B64);
        match_uint_literal!(self, Usz, Bsz);

        match_float_literal!(self,      F32, B32);
        match_float_literal!(self,      F64, B64);
        match_float_literal!(self, AnyFloat, Any);

        match_string_literal!(self,    String, Slice);
        match_string_literal!(self, RawString,   Raw);
        match_string_literal!(self,      Char,  Char);

        let last = self.peek();
        error!(self, last, "Expecting expression");

        None
    }

    fn static_access(&mut self) -> Option<Expression> {
        let mut expr = self.primary()?;

        while self.match_(&[TokenType::ColonColon]) {
            let gets_macro = self.match_(&[TokenType::At]);
            let name = self.consume(TokenType::Identifier, "Expecting property name after '::'")?.clone();
            expr = Expression::StaticGet(StaticGetTarget::Expression(Box::new(expr)), name, gets_macro);
        }

        Some(expr)
    }

    fn macro_call(&mut self) -> Option<Expression> {
        if self.match_(&[TokenType::At]) {
            let op = self.previous().clone();
            let name = self.consume(TokenType::Identifier, "Expecting macro name after '@'")?.clone();
            return Some(Expression::Unary { op, expr: Box::new(Expression::Variable(name)), is_prefix: true });
        }

        self.static_access()
    }

    fn get_expressions(&mut self, type_: TokenType) -> Option<Vec<Expression>> {
        let mut expressions = Vec::new();

        while !self.check(type_) {
            expressions.push(self.expression()?);

            if !self.match_(&[TokenType::Comma]) {
                break;
            }
        }

        Some(expressions)
    }

    fn finish_call(&mut self, callee: Expression) -> Option<Expression> {
        let unpack = self.match_(&[TokenType::DotDot]);
        let arguments = {
            if unpack {
                vec![self.expression()?]
            } else {
                self.get_expressions(TokenType::RightParen)?
            }
        };

        let paren = self.consume(TokenType::RightParen, "Expecting ')' after arguments.")?.clone();
        Some(Expression::Call(Box::new(callee), paren, arguments, unpack))
    }

    fn finish_subscript(&mut self, subscripted: Expression) -> Option<Expression> {
        let arguments = self.get_expressions(TokenType::RightParen)?;
        let paren = self.consume(TokenType::RightSquare, "Expecting ']' after subscript operation")?.clone();
        Some(Expression::Subscript { subscripted: Box::new(subscripted), paren, args: arguments })
    }

    fn finish_struct_literal(&mut self, expr: Expression) -> Option<Expression> {
        let mut fields = Vec::new();
        while !self.check(TokenType::RightBrace) {
            let name = self.consume(TokenType::Identifier, "Expecting field name")?.clone();

            if self.match_(&[TokenType::Colon]) {
                fields.push(StructField::new(name, self.expression()?, None, true));
            } else {
                fields.push(StructField::new(name.clone(), Expression::Variable(name), None, true));
            }

            if !self.match_(&[TokenType::Comma]) {
                break;
            }
        }

        let paren = self.consume(TokenType::RightBrace, "Expecting '}' after struct literal fields")?.clone();
        Some(Expression::CompoundLiteral { type_: Box::new(expr), closing_brace: paren, fields })
    }

    fn call(&mut self) -> Option<Expression> {
        let mut expr = self.macro_call()?;

        loop {
            if self.match_(&[TokenType::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else if self.match_(&[TokenType::LeftSquare]) {
                expr = self.finish_subscript(expr)?;
            } else if self.match_(&[TokenType::Dot]) {
                if self.match_(&[TokenType::LeftBrace]) {
                    expr = self.finish_struct_literal(expr)?;
                } else {
                    let name = self.consume(TokenType::Identifier, "Expecting property name after '.'")?.clone();
                    expr = Expression::Get(Box::new(expr), name);
                }
            } else {
                break;
            }
        }

        Some(expr)
    }

    suffix_unary!(suffix_unary, call, &[TokenType::PlusPlus, TokenType::MinusMinus]);

    fn prefix_unary(&mut self) -> Option<Expression> {
        if self.match_(&[
            TokenType::PlusPlus, TokenType::MinusMinus,
            TokenType::Minus, TokenType::Tilde, TokenType::Star,
            TokenType::Bang, TokenType::BitwiseAnd
        ]) {
            let mut op = self.previous().clone();

            let star = op.type_ == TokenType::Star;
            let and = op.type_ == TokenType::BitwiseAnd;

            if (star || and) && self.match_(&[TokenType::Const]) {
                if star {
                    op.set_type(TokenType::StarConst);
                } else if and {
                    op.set_type(TokenType::RefConst);
                }
            }

            let right = self.prefix_unary()?;
            return Some(Expression::Unary { op, expr: Box::new(right), is_prefix: true });
        }

        self.suffix_unary()
    }

    left_associativity_binary!(factor, prefix_unary, &[TokenType::Slash, TokenType::Star, TokenType::Mod]);
    left_associativity_binary!(term, factor, &[TokenType::Minus, TokenType::Plus]);
    left_associativity_binary!(shift, term, &[TokenType::ShiftLeft, TokenType::ShiftRight]);
    left_associativity_binary!(comparison, shift, &[TokenType::Greater, TokenType::GreaterEqual, TokenType::Less, TokenType::LessEqual]);
    left_associativity_binary!(equality, comparison, &[TokenType::BangEqual, TokenType::EqualEqual]);
    left_associativity_binary!(bitwise_and, equality, &[TokenType::BitwiseAnd]);
    left_associativity_binary!(bitwise_xor, bitwise_and, &[TokenType::BitwiseXor]);
    left_associativity_binary!(result, bitwise_xor, &[TokenType::Bang]);

    prefix_unary!(optional, result, &[TokenType::Question]);
    prefix_unary!(try_operator, optional, &[TokenType::Try]);

    left_associativity_binary!(bitwise_or, try_operator, &[TokenType::BitwiseOr]);
    left_associativity_binary!(logic_and, bitwise_or, &[TokenType::LogicAnd]);
    left_associativity_binary!(logic_or, logic_and, &[TokenType::LogicOr]);

    fn ternary(&mut self) -> Option<Expression> {
        let cond = self.logic_or()?;

        if !self.match_(&[TokenType::Question]) {
            return Some(cond);
        }

        let question = self.previous().clone();
        let then = self.expression()?;
        self.consume(TokenType::Colon, "Expecting ':' after 'then' branch of ternary expression")?;
        let else_ = self.expression()?;

        Some(Expression::Ternary { tok: question, condition: Box::new(cond), then_expr: Box::new(then), else_expr: Box::new(else_) })
    }

    fn assignment(&mut self) -> Option<Expression> {
        let expr = self.ternary()?;

        if self.match_(&[
            TokenType::Equal, TokenType::PlusEquals, TokenType::MinusEquals,
            TokenType::StarEquals, TokenType::SlashEquals, TokenType::ModEquals,
            TokenType::ShiftLeftEquals, TokenType::ShiftRightEquals, TokenType::AndEquals,
            TokenType::XorEquals, TokenType::OrEquals
        ]) {
            let op = self.previous().clone();
            let value = self.assignment()?;

            if !expr.is_valid_assignment_target() {
                error!(self, op, "Invalid assignment target");
                return None;
            }

            return Some(Expression::Assign { target: Box::new(expr), op, value: Box::new(value) });
        }

        Some(expr)
    }

    pub fn expression(&mut self) -> Option<Expression> {
        self.assignment()
    }

    pub fn type_expression(&mut self) -> Option<Expression> {
        self.bitwise_or()
    }

    pub fn switch_case_expression(&mut self) -> Option<Expression> {
        self.bitwise_xor()
    }

    fn expression_statement(&mut self) -> Option<Statement> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "Expecting ';' after expression")?;
        Some(Statement::Expression(expr))
    }

    fn block(&mut self) -> Option<Vec<Statement>> {
        let mut statements = Vec::new();

        while (!self.check(TokenType::RightBrace)) && (!self.is_at_end()) {
            statements.push(self.declaration(false, &Vec::new(), &Vec::new())?);
        }

        self.consume(TokenType::RightBrace, "Expecting '}' after block")?;
        Some(statements)
    }

    fn get_block(&mut self) -> Option<Statement> {
        let bracket = self.previous().clone();
        Some(Statement::Block(bracket, self.block()?))
    }

    fn if_statement(&mut self) -> Option<Statement> {
        let kw = self.previous().clone();

        let cond = self.expression()?;

        let then_branch = {
            if let Expression::Grouping(_) = cond {
                self.statement()?
            } else {
                self.consume(TokenType::LeftBrace, "Expecting '{' after if condition")?;
                self.get_block()?
            }
        };

        let else_branch = {
            if self.match_(&[TokenType::Else]) {
                Some(Box::new(self.statement()?))
            } else {
                None
            }
        };

        Some(Statement::If { kw, condition: cond, then_branch: Box::new(then_branch), else_branch })
    }

    fn while_statement(&mut self) -> Option<Statement> {
        let kw = self.previous().clone();

        let cond = self.expression()?;

        let body = {
            if let Expression::Grouping(_) = cond {
                if self.match_(&[TokenType::Semicolon]) {
                    Statement::Block(self.previous().clone(), Vec::new())
                } else {
                    self.statement()?
                }
            } else {
                self.consume(TokenType::LeftBrace, "Expecting '{' after while condition")?;
                self.get_block()?
            }
        };

        Some(Statement::While { kw, condition: cond, body: Box::new(body) })
    }

    fn do_while_statement(&mut self) -> Option<Statement> {
        let kw = self.previous().clone();

        let body = self.statement()?;
        self.consume(TokenType::While, "Expecting 'while' after 'do' body")?;

        let cond = self.expression()?;
        self.consume(TokenType::Semicolon, "Expecting ';' after condition")?;

        Some(Statement::DoWhile { kw, condition: cond, body: Box::new(body) })
    }

    fn for_statement(&mut self) -> Option<Statement> {
        let kw = self.previous().clone();

        let has_paren = self.match_(&[TokenType::LeftParen]);

        let initializer = {
            if self.match_(&[TokenType::Semicolon]) {
                None
            } else if self.match_(&[TokenType::Let, TokenType::Const]) {
                Some(Box::new(self.var_decl()?))
            } else {
                Some(Box::new(self.expression_statement()?))
            }
        };

        let cond = {
            if self.check(TokenType::Semicolon) {
                Expression::UnsignedIntLiteral { value: 1, tok: self.previous().clone(), bits: Bits::B8 }
            } else {
                self.expression()?
            }
        };

        if self.match_(&[TokenType::Semicolon]) { // C-like for
            let increment = {
                if has_paren {
                    let r = {
                        if self.check(TokenType::RightParen) {
                            Vec::new()
                        } else {
                            self.get_expressions(TokenType::RightParen)?
                        }
                    };

                    self.consume(TokenType::RightParen, "Expecting ')' after for increment")?;
                    r
                } else if self.check(TokenType::LeftBrace) {
                    Vec::new()
                } else {
                    self.get_expressions(TokenType::RightParen)?
                }
            };

            let body = {
                if has_paren {
                    if self.match_(&[TokenType::Semicolon]) {
                        Statement::Block(kw.clone(), Vec::new())
                    } else {
                        self.statement()?
                    }
                } else {
                    self.consume(TokenType::LeftBrace, "Expecting '{' after for")?;
                    self.get_block()?
                }
            };

            Some(Statement::For { kw, initializer, condition: cond, increments: increment, body: Box::new(body) })
        } else { // foreach
            let body = {
                if has_paren {
                    self.consume(TokenType::RightParen, "Expecting ')' after for iterator")?;

                    if self.match_(&[TokenType::Semicolon]) {
                        Statement::Block(kw.clone(), Vec::new())
                    } else {
                        self.statement()?
                    }
                } else {
                    self.consume(TokenType::LeftBrace, "Expecting '{' after for")?;
                    self.get_block()?
                }
            };

            if let Some(init) = &initializer {
                if let Statement::Expression(Expression::Variable(var_name)) = &**init {
                    return Some(Statement::Foreach { kw, variable_name: var_name.clone(), iterator: cond, body: Box::new(body) })
                } else {
                    error!(self, init, "Expecting variable name in foreach loop");
                }
            } else {
                error!(self, kw, "Expecting variable name in foreach loop");
            }

            None
        }
    }

    fn return_statement(&mut self) -> Option<Statement> {
        let kw = self.previous().clone();

        let value = {
            if self.check(TokenType::Semicolon) {
                None
            } else {
                Some(self.expression()?)
            }
        };

        self.consume(TokenType::Semicolon, "Expecting ';' after return value")?;
        Some(Statement::Return { kw, value })
    }

    fn defer_statement(&mut self) -> Option<Statement> {
        let kw = self.previous().clone();

        if self.match_(&[TokenType::Let]) {
            let tok = self.consume(TokenType::Identifier, "Expecting '_' after defer let")?.clone();

            if tok.lexeme.as_ref() != "_" {
                error!(self, self.previous(), "Expecting '_' after defer let");
                note(
                    self.previous(),
                    concat!(
                        "Variable declaration is not allowed in defer statements. ",
                        "The \"let\" keyword in this context is used for discarding errors"
                    )
                );
            }

            self.consume(TokenType::Equal, "Expecting '=' before defer let expression")?;
            let expr = self.expression()?;
            self.consume(TokenType::Semicolon, "Expecting ';' after expression")?;

            Some(Statement::Defer { kw, statement: Box::new(Statement::VarDecl { name: tok, initializer: Some(expr), type_: None, is_const: false, qualifiers: Vec::new() }) })
        } else {
            Some(Statement::Defer { kw, statement: Box::new(self.statement()?) })
        }
    }

    fn switch_statement(&mut self) -> Option<Statement> {
        let kw = self.previous().clone();
        let expr = self.expression()?;

        self.consume(TokenType::LeftBrace, "Expecting '{' before switch body")?;
        let mut cases = Vec::new();
        if !self.check(TokenType::RightBrace) {
            loop {
                let expressions = {
                    if self.match_(&[TokenType::Else]) {
                        None
                    } else {
                        let mut exprs = Vec::new();
                        loop {
                            exprs.push(self.switch_case_expression()?);

                            if !self.match_(&[TokenType::BitwiseOr]) {
                                break;
                            }
                        }

                        Some(exprs)
                    }
                };

                if self.match_(&[TokenType::Arrow]) {
                    cases.push(SwitchCase::new(expressions, vec![self.statement()?]));
                } else {
                    self.consume(TokenType::LeftBrace, "Expecting '{' after case expression")?;
                    cases.push(SwitchCase::new(expressions, self.block()?));
                }

                if self.check(TokenType::RightBrace) {
                    break;
                }
            }
        }

        self.consume(TokenType::RightBrace, "Expecting '}' after switch body")?;
        Some(Statement::Switch { kw, expr, cases })
    }

    fn break_statement(&mut self) -> Option<Statement> {
        let kw = self.previous().clone();
        self.consume(TokenType::Semicolon, "Expecting ';' after break")?;
        Some(Statement::Break(kw))
    }

    fn continue_statement(&mut self) -> Option<Statement> {
        let kw = self.previous().clone();
        self.consume(TokenType::Semicolon, "Expecting ';' after continue")?;
        Some(Statement::Continue(kw))
    }

    fn statement(&mut self) -> Option<Statement> {
        if self.curr_qualifiers.len() != 0 {
            let kw = self.peek();
            error!(self, kw, "Can only use qualifiers on declarations");

            for (_, qualifier) in self.curr_qualifiers.iter() {
                note(&qualifier.name, "Qualifier specified here");
            }

            self.curr_qualifiers.clear();
        }

        if self.match_(&[TokenType::If]) {
            return self.if_statement();
        }

        if self.match_(&[TokenType::While]) {
            return self.while_statement();
        }

        if self.match_(&[TokenType::Do]) {
            return self.do_while_statement();
        }

        if self.match_(&[TokenType::For]) {
            return self.for_statement();
        }

        if self.match_(&[TokenType::Return]) {
            return self.return_statement();
        }

        if self.match_(&[TokenType::Defer]) {
            return self.defer_statement();
        }

        if self.match_(&[TokenType::Switch]) {
            return self.switch_statement();
        }

        if self.match_(&[TokenType::Break]) {
            return self.break_statement();
        }

        if self.match_(&[TokenType::Continue]) {
            return self.continue_statement();
        }

        if self.match_(&[TokenType::LeftBrace]) {
            return self.get_block();
        }

        self.expression_statement()
    }

    fn var_decl(&mut self) -> Option<Statement> {
        let mut qualifiers = Vec::new();

        for (name, qualifier) in self.curr_qualifiers.iter() {
            match name.as_ref() {
                "static" | "extern" | "volatile" => {
                    if let Some(arg) = &qualifier.arg {
                        error!(
                            self, arg, 
                            format!("\"{}\" qualifier doesn't accept an argument", qualifier.name.lexeme).as_str()
                        );
                    }   

                    qualifiers.push(qualifier.name.clone());
                }
                _ => error!(self, qualifier.name, "Unsupported qualifier for variable declaration")
            }
        }

        self.curr_qualifiers.clear();

        let kw = self.previous().clone();
        let name = self.consume(TokenType::Identifier, "Expecting variable name")?.clone();

        let type_ = {
            if self.match_(&[TokenType::Colon]) {
                Some(self.type_expression()?)
            } else {
                None
            }
        };

        let initializer = {
            if self.match_(&[TokenType::Equal]) {
                Some(self.expression()?)
            } else {
                None
            }
        };

        self.consume(TokenType::Semicolon, "Expecting ';' after variable declaration")?;
        Some(Statement::VarDecl { name, initializer, type_, is_const: kw.type_ == TokenType::Const, qualifiers })
    }

    fn parse_generics(&mut self, incoming_generics: &Vec<Generic>) -> Option<Vec<Generic>> {
        let mut generics = Vec::new();
        generics.append(&mut incoming_generics.clone());

        let mut had_default = false;
        let mut has_default = false;
        if self.match_(&[TokenType::LeftSquare]) {
            while !self.check(TokenType::RightSquare) {
                let name = self.consume(TokenType::Identifier, "Expecting generic type name")?.clone();

                let bounds = {
                    if self.match_(&[TokenType::Colon]) {
                        Some(self.type_expression()?)
                    } else {
                        None
                    }
                };

                let default = {
                    if self.match_(&[TokenType::Equal]) {
                        if had_default && !has_default {
                            error!(self, name, "Cannot alternate generic with no default type with generic with default type");
                            None
                        } else {
                            has_default = true;
                            had_default = true;
                            Some(self.expression()?)
                        }
                    } else {
                        has_default = false;
                        None
                    }
                };

                generics.push(Generic::new(name, bounds, default));

                if !self.match_(&[TokenType::Comma]) {
                    break;
                }
            }

            self.consume(TokenType::RightSquare, "Expecting ']' after generic types declaration")?;
        }

        Some(generics)
    }

    fn function(&mut self, method: bool, incoming_generics: &Vec<Generic>, self_generics: &Vec<Expression>) -> Option<Statement> {
        let mut qualifiers = Vec::new();
        let mut bind = false;
        let mut init = false;
        let mut link_name = None;

        for (name, qualifier) in self.curr_qualifiers.iter() {
            if qualifier.arg.is_some() && matches!(name.as_ref(), "static" | "extern" | "inline" | "init" | "bind") {
                error!(
                    self, qualifier.arg.as_ref().unwrap(), 
                    format!("\"{}\" qualifier doesn't accept an argument", qualifier.name.lexeme).as_str()
                );
            }

            match name.as_ref() {
                "static" | "extern" | "inline" => qualifiers.push(qualifier.name.clone()),
                "init" => init = true,
                "bind" => bind = true,
                "linkName" => {
                    if let Some(arg) = &qualifier.arg {
                        link_name = Some(arg.clone());
                    } else {
                        error!(self, qualifier.name, "Missing argument for #linkName qualifier");
                    }
                }
                _ => error!(self, qualifier.name, "Unsupported qualifier for function definition")
            }
        }

        self.curr_qualifiers.clear();

        let name = self.consume(TokenType::Identifier, "Expecting function name")?.clone();
        let generics = self.parse_generics(incoming_generics)?;

        if generics.len() != 0 {
            if bind {
                error!(self, self.previous(), "Generics are not allowed in function bindings");
            }

            if init {
                error!(self, self.previous(), "Generics are not allowed in #init functions");
            }

            if link_name.is_some() {
                error!(self, self.previous(), "Cannot set link name of a function with generics");
            }
        }

        self.consume(TokenType::LeftParen, "Expecting '(' after function name")?;

        let mut params = Vec::new();
        while !self.check(TokenType::RightParen) {
            let is_const = self.match_(&[TokenType::Const]);
            let p_name = self.consume(TokenType::Identifier, "Expecting parameter name")?.clone();

            if method && p_name.lexeme.as_ref() == "self" {
                if self.match_(&[TokenType::Colon]) {
                    let type_expr = self.expression()?;
                    error!(self, type_expr, "Type is not required for \"self\" inside methods");
                    note(&type_expr, "Remove the type");
                }

                let mut custom_tok = p_name.clone();
                custom_tok.set_lexeme("Self");
                let mut type_ = Expression::Variable(custom_tok.clone());

                if self_generics.len() != 0 {
                    type_ = Expression::Subscript { subscripted: Box::new(type_), paren: custom_tok, args: self_generics.clone() };
                }

                let mut star_tok = p_name.clone();
                if is_const {
                    star_tok.set_type(TokenType::RefConst);
                } else {
                    star_tok.set_type(TokenType::BitwiseAnd)
                }

                type_ = Expression::Unary { op: star_tok, expr: Box::new(type_), is_prefix: true };
                params.push(FunctionParam::new(Some(p_name), type_, true));
            } else {
                self.consume(TokenType::Colon, "Expecting ':' after parameter name")?;
                let type_ = self.expression()?;

                params.push(FunctionParam::new(Some(p_name), type_, is_const));
            }

            if !self.match_(&[TokenType::Comma]) {
                break;
            }
        }

        self.consume(TokenType::RightParen, "Expecting ')' after function parameters")?;

        let return_type = {
            if !(self.check(TokenType::LeftBrace) || self.check(TokenType::Semicolon)) {
                self.expression()?
            } else {
                Expression::VoidLiteral(self.previous().clone())
            }
        };

        let body = {
            if self.match_(&[TokenType::LeftBrace]) {
                if bind {
                    error!(self, self.previous(), "Cannot define function body for binding");
                }

                Some(self.block()?)
            } else {
                self.consume(TokenType::Semicolon, "Expecting '{' or ';' after function declaration")?;
                None
            }
        };

        let info = FunctionInfo { qualifiers, bind, init, link_name };
        if generics.len() == 0 {
            Some(Statement::Function { name, params, return_type, body,  generics_names: Vec::new(), info })
        } else {
            let mut generic_names = Vec::new();
            for generic in &generics {
                generic_names.push(generic.name.clone());
            }

            Some(Statement::Template {
                name: name.clone(),
                declaration: Box::new(Statement::Function {
                    name, params, return_type, body, info,
                    generics_names: generic_names.clone(),
                }),
                generics, generics_names: generic_names
            })
        }
    }

    fn struct_decl(&mut self, incoming_generics: &Vec<Generic>) -> Option<Statement> {
        let mut typedefed = false;
        let mut bind = false;

        for (name, qualifier) in self.curr_qualifiers.iter() {
            if qualifier.arg.is_some() && matches!(name.as_ref(), "typedef" | "bind") {
                error!(
                    self, qualifier.arg.as_ref().unwrap(), 
                    format!("\"{}\" qualifier doesn't accept an argument", qualifier.name.lexeme).as_str()
                );
            }

            match name.as_ref() {
                "typedef" => typedefed = true,
                "bind" => bind = true,
                _ => error!(self, qualifier.name, "Unsupported qualifier for struct definition")
            }
        }

        self.curr_qualifiers.clear();

        let name = self.consume(TokenType::Identifier, "Expecting struct name")?.clone();
        let generics = self.parse_generics(incoming_generics)?;

        let binding = {
            if bind {
                Some(name.clone())
            } else if self.match_(&[TokenType::Colon]) {
                if generics.len() != 0 {
                    error!(self, self.previous(), "Cannot use generics in a C struct binding");
                    None
                } else {
                    Some(self.consume(TokenType::Identifier, "Expecting C struct name after struct binding")?.clone())
                }
            } else {
                if typedefed {
                    error!(self, self.previous(), "Cannot use #typedef qualifier on struct that is not a binding");
                }

                None
            }
        };

        let mut fields = Vec::new();

        let has_body = {
            if self.match_(&[TokenType::LeftBrace]) {
                while !self.check(TokenType::RightBrace) {
                    let is_const = self.match_(&[TokenType::Const]);
                    let field_name = self.consume(TokenType::Identifier, "Expecting field name")?.clone();

                    let bits = {
                        if self.match_(&[TokenType::LeftSquare]) {
                            let ret = self.expression()?;
                            self.consume(TokenType::RightBrace, "Expecting ']' after struct field bit size");
                            Some(ret)
                        } else {
                            None
                        }
                    };

                    self.consume(TokenType::Colon, "Expecting ':' after field name")?;
                    let type_ = self.type_expression()?;

                    fields.push(StructField::new(field_name, type_, bits, is_const));

                    if !self.match_(&[TokenType::Comma]) {
                        break;
                    }
                }

                self.consume(TokenType::RightBrace, "Expecting '}' after struct body")?;
                true
            } else {
                self.consume(TokenType::Semicolon, "Expecting '{' or ';' after struct declaration")?;
                false
            }
        };

        if generics.len() == 0 {
            Some(Statement::Struct { name, fields, has_body, binding, generics_names: Vec::new(), bind_typedefed: typedefed })
        } else {
            let mut generic_names = Vec::new();
            for generic in &generics {
                generic_names.push(generic.name.clone());
            }

            Some(Statement::Template { name: name.clone(), declaration: Box::new(Statement::Struct { name, fields, has_body, binding, generics_names: generic_names.clone(), bind_typedefed: typedefed }), generics, generics_names: generic_names })
        }
    }

    fn no_qualifiers(&mut self, name: &str) {
        if self.curr_qualifiers.len() != 0 {
            let kw = self.previous();
            error!(self, kw, format!("Cannot use qualifiers on \"{}\" statement", name).as_str());

            for (_, qualifier) in self.curr_qualifiers.iter() {
                note(&qualifier.name, "Qualifier specified here");
            }

            self.curr_qualifiers.clear();
        }
    }

    fn impl_decl(&mut self) -> Option<Statement> {
        self.no_qualifiers("impl");

        let generics = self.parse_generics(&Vec::new())?;

        let mut struct_impl = self.type_expression()?;

        let self_generics = {
            if let Expression::Subscript { subscripted, paren: _, args: self_generics } = struct_impl {
                struct_impl = *subscripted;
                self_generics
            } else {
                Vec::new()
            }
        };

        self.consume(TokenType::LeftBrace, "Expecting '{' before impl body")?;

        let mut declarations = Vec::new();
        while (!self.check(TokenType::RightBrace)) && (!self.is_at_end()) {
            declarations.push(self.declaration(true, &generics, &self_generics)?);
        }

        self.consume(TokenType::RightBrace, "Expecting '}' after impl body")?;
        Some(Statement::Impl { object: struct_impl, declarations })
    }

    fn namespace(&mut self) -> Option<Statement> {
        self.no_qualifiers("namespace");
        let name = self.consume(TokenType::Identifier, "Expecting namespace name")?.clone();
        self.consume(TokenType::LeftBrace, "Expecting '{' before namespace body")?;
        Some(Statement::Namespace { name, body: self.block()? })
    }

    fn use_statement(&mut self) -> Option<Statement> {
        let mut typedef = false;
        let mut bind = false;

        for (name, qualifier) in self.curr_qualifiers.iter() {
            match name.as_ref() {
                "typedef" => typedef = true,
                "bind" => bind = true,
                _ => error!(self, qualifier.name, "Unsupported qualifier for use statement")
            }
        }

        self.curr_qualifiers.clear();

        let use_expr = self.expression()?;

        let as_ = {
            if let Expression::StaticGet(_, name, _) = &use_expr {
                if self.match_(&[TokenType::As]) {
                    self.consume(TokenType::Identifier, "Expecting identifier after \"as\"")?.clone()
                } else {
                    name.clone()
                }
            } else {
                self.consume(TokenType::As, "Expecting \"as\" after use expression")?;
                self.consume(TokenType::Identifier, "Expecting identifier after \"as\"")?.clone()
            }
        };

        self.consume(TokenType::Semicolon, "Expecting ';' after use statement")?;
        Some(Statement::Use { use_expr, as_name: as_, typedef, bind })
    }

    fn enum_decl(&mut self, incoming_generics: &Vec<Generic>) -> Option<Statement> {
        let mut typedefed = false;
        let mut bind = false;

        for (name, qualifier) in self.curr_qualifiers.iter() {
            if qualifier.arg.is_some() && matches!(name.as_ref(), "typedef" | "bind") {
                error!(
                    self, qualifier.arg.as_ref().unwrap(), 
                    format!("\"{}\" qualifier doesn't accept an argument", qualifier.name.lexeme).as_str()
                );
            }

            match name.as_ref() {
                "typedef" => typedefed = true,
                "bind" => bind = true,
                _ => error!(self, qualifier.name, "Unsupported qualifier for enum definition")
            }
        }

        self.curr_qualifiers.clear();

        let name = self.consume(TokenType::Identifier, "Expecting enum name")?.clone();
        let generics = self.parse_generics(incoming_generics)?;

        let binding = {
            if bind {
                Some(name.clone())
            } else if self.match_(&[TokenType::Colon]) {
                if generics.len() != 0 {
                    error!(self, self.previous(), "Cannot use generics in a C enum binding");
                    None
                } else {
                    Some(self.consume(TokenType::Identifier, "Expecting C enum name after enum binding")?.clone())
                }
            } else {
                if typedefed {
                    error!(self, self.previous(), "Cannot use #typedef qualifier on enum that is not a binding");
                }

                None
            }
        };

        let type_ = {
            if self.match_(&[TokenType::As]) {
                self.type_expression()?
            } else {
                let mut custom_tok = name.clone();
                custom_tok.set_lexeme("i32");
                Expression::Variable(custom_tok)
            }
        };

        let mut variants = Vec::new();
        let mut is_simple = true;

        let has_body = {
            if self.match_(&[TokenType::LeftBrace]) {
                while !self.check(TokenType::RightBrace) {
                    let variant_name = self.consume(TokenType::Identifier, "Expecting field name")?.clone();

                    let type_ = {
                        if self.match_(&[TokenType::LeftParen]) {
                            let res = self.expression()?;

                            if is_simple && !matches!(res, Expression::VoidLiteral(_)) {
                                is_simple = false;
                            }

                            self.consume(TokenType::RightParen, "Expecting ')' after enum variant type")?;
                            res
                        } else {
                            Expression::VoidLiteral(variant_name.clone())
                        }
                    };

                    let default = {
                        if self.match_(&[TokenType::Equal]) {
                            Some(self.expression()?)
                        } else {
                            None
                        }
                    };

                    variants.push(EnumVariant::new(variant_name, type_, default));

                    if !self.match_(&[TokenType::Comma]) {
                        break;
                    }
                }

                self.consume(TokenType::RightBrace, "Expecting '}' after enum body")?;
                true
            } else {
                self.consume(TokenType::Semicolon, "Expecting '{' or ';' after enum declaration")?;
                false
            }
        };

        if !is_simple {
            if binding.is_some() {
                error!(self, binding.as_ref().unwrap(), "Cannot use enum as sum type when creating a C binding");
            }

            if let Some(variant) = variants.iter().find(|x| x.name.lexeme.as_ref() == "kind") {
                error!(self, variant.name, "Sum type enum variant cannot be called \"kind\"");
                note(&variant.name, "This name is reserved for sum type enums");
            }
        }

        if generics.len() == 0 {
            Some(Statement::Enum { 
                name, kind_type: 
                type_, 
                variants, 
                is_simple, 
                has_body, 
                binding, 
                generics_names: Vec::new(), 
                bind_typedefed: typedefed 
            })
        } else {
            let mut generics_names = Vec::new();
            for generic in &generics {
                generics_names.push(generic.name.clone());
            }

            Some(Statement::Template { 
                name: name.clone(), 
                declaration: Box::new(Statement::Enum { 
                    name, 
                    kind_type: type_, 
                    variants, 
                    is_simple, 
                    has_body, 
                    binding, 
                    generics_names: generics_names.clone(), 
                    bind_typedefed: typedefed 
                }), 
                generics, 
                generics_names 
            })
        }
    }

    fn import_statement(&mut self) -> Option<Statement> {
        let is_include = matches!(self.previous().type_, TokenType::Include);

        if is_include {
            self.no_qualifiers("include");
        } else {
            self.no_qualifiers("import");
        }

        let import_type = {
            if self.match_(&[TokenType::Less]) {
                ImportType::Ang
            } else if self.match_(&[TokenType::ShiftLeft]) {
                ImportType::Lib
            } else {
                ImportType::Default
            }
        };

        let path = {
            if let Some(tok) = self.consume(TokenType::String, "Expecting path in string format after \"import\"") {
                tok.clone()
            } else {
                note(self.peek(), "Add quotation marks around the file");
                return None;
            }
        };

        match import_type {
            ImportType::Default => (),
            ImportType::Ang => {
                self.consume(TokenType::Greater, "Expecting '>' after import with angular brackets")?;
            }
            ImportType::Lib => {
                self.consume(TokenType::ShiftRight, "Expecting '>>' after Skye lib C import")?;
            }
        }

        let namespace = {
            if self.match_(&[TokenType::Namespace]) {
                Some(self.consume(TokenType::Identifier, "Expecting name after import namespace")?.clone())
            } else {
                None
            }
        };

        let mut flags = Vec::new();
        if self.match_(&[TokenType::Use]) {
            while !self.check(TokenType::Semicolon) {
                flags.push(self.consume(TokenType::Identifier, "Expecting flag name after use")?.clone());

                if !self.match_(&[TokenType::Comma]) {
                    break;
                }
            }
        }

        self.consume(TokenType::Semicolon, "Expecting ';' after import statement")?;

        let import = Statement::Import { path, type_: import_type, is_include, flags };
        if let Some(namespace) = namespace {
            Some(Statement::Namespace { 
                name: namespace, 
                body: vec![import] 
            })
        } else {
            Some(import)
        }
    }

    fn union_decl(&mut self) -> Option<Statement> {
        let mut typedefed = false;
        let mut bind = false;

        for (name, qualifier) in self.curr_qualifiers.iter() {
            if qualifier.arg.is_some() && matches!(name.as_ref(), "typedef" | "bind") {
                error!(
                    self, qualifier.arg.as_ref().unwrap(), 
                    format!("\"{}\" qualifier doesn't accept an argument", qualifier.name.lexeme).as_str()
                );
            }

            match name.as_ref() {
                "typedef" => typedefed = true,
                "bind" => bind = true,
                _ => error!(self, qualifier.name, "Unsupported qualifier for union definition")
            }
        }

        self.curr_qualifiers.clear();

        let name = self.consume(TokenType::Identifier, "Expecting union name")?.clone();
        if self.parse_generics(&Vec::new())?.len() != 0 {
            error!(self, self.previous(), "Generics are not allowed in unions");
        }

        let binding = {
            if bind {
                Some(name.clone())
            } else if self.match_(&[TokenType::Colon]) {
                Some(self.consume(TokenType::Identifier, "Expecting C union name after union binding")?.clone())
            } else {
                if typedefed {
                    error!(self, self.previous(), "Cannot use #typedef qualifier on union that is not a binding");
                }

                None
            }
        };

        let mut fields = Vec::new();

        let has_body = {
            if self.match_(&[TokenType::LeftBrace]) {
                while !self.check(TokenType::RightBrace) {
                    let is_const = self.match_(&[TokenType::Const]);
                    let field_name = self.consume(TokenType::Identifier, "Expecting field name")?.clone();

                    let bits = {
                        if self.match_(&[TokenType::LeftSquare]) {
                            let ret = self.expression()?;
                            self.consume(TokenType::RightBrace, "Expecting ']' after union field bit size");
                            Some(ret)
                        } else {
                            None
                        }
                    };

                    self.consume(TokenType::Colon, "Expecting ':' after field name")?;
                    let type_ = self.type_expression()?;

                    fields.push(StructField::new(field_name, type_, bits, is_const));

                    if !self.match_(&[TokenType::Comma]) {
                        break;
                    }
                }

                self.consume(TokenType::RightBrace, "Expecting '}' after union body")?;
                true
            } else {
                self.consume(TokenType::Semicolon, "Expecting '{' or ';' after union declaration")?;
                false
            }
        };

        Some(Statement::Union { name, fields, has_body, binding, bind_typedefed: typedefed })
    }

    fn macro_decl(&mut self) -> Option<Statement> {
        self.no_qualifiers("macro");

        let name = self.consume(TokenType::Identifier, "Expecting macro name")?.clone();

        let params = {
            if self.match_(&[TokenType::LeftParen]) {
                let mut params = Vec::new();
                while !self.check(TokenType::RightParen) {
                    params.push(self.consume(TokenType::Identifier, "Expecting parameter name")?.clone());

                    if !self.match_(&[TokenType::Comma]) {
                        break;
                    }
                }

                let ret = {
                    if params.len() == 1 && self.match_(&[TokenType::Star]) {
                        MacroParams::ZeroN(params[0].clone())
                    } else if params.len() == 1 && self.match_(&[TokenType::Plus]) {
                        MacroParams::OneN(params[0].clone())
                    } else {
                        MacroParams::Some(params)
                    }
                };

                self.consume(TokenType::RightParen, "Expecting ')' after macro parameters")?;
                ret
            } else {
                MacroParams::None
            }
        };

        let body = {
            if self.match_(&[TokenType::Arrow]) {
                let type_expr = self.type_expression()?;
                self.consume(TokenType::Semicolon, "Expecting ';' after macro binding")?;
                MacroBody::Binding(type_expr)
            } else if self.match_(&[TokenType::LeftBrace]) {
                MacroBody::Block(self.block()?)
            } else {
                let expr = self.expression()?;
                self.consume(TokenType::Semicolon, "Expecting ';' after macro expression")?;
                MacroBody::Expression(expr)
            }
        };

        Some(Statement::Macro { name, params, body })
    }

    fn interface(&mut self) -> Option<Statement> {
        self.no_qualifiers("interface");

        let name = self.consume(TokenType::Identifier, "Expecting interface name")?.clone();

        let declarations = {
            if self.match_(&[TokenType::LeftBrace]) {
                let mut declarations = Vec::new();
                while (!self.check(TokenType::RightBrace)) && (!self.is_at_end()) {
                    declarations.push(self.declaration(true, &Vec::new(), &Vec::new())?);
                }

                self.consume(TokenType::RightBrace, "Expecting '}' after interface body")?;

                Some(declarations)
            } else {
                None
            }
        };

        let types = {
            if self.match_(&[TokenType::For]) {
                let has_paren = self.match_(&[TokenType::LeftParen]);

                let exprs = {
                    if has_paren {
                        let exprs = self.get_nonzero_expressions(TokenType::LeftParen)?;
                        self.consume(TokenType::RightParen, "Expecting ')' after list of types enclosed in parentheses")?;
                        exprs
                    } else {
                        self.get_nonzero_expressions(TokenType::Semicolon)?
                    }
                };

                self.consume(TokenType::Semicolon, "Expecting ';' after interface types")?;
                Some(exprs)
            } else {
                None
            }
        };

        if declarations.is_none() {
            if let Some(bound_types) = &types {
                error!(self, bound_types[0], "Cannot declare interface types without interface body");
                note(&name, "Add the methods that compose the interface after the interface name, enclosed in brackets");
            } else {
                self.consume(TokenType::Semicolon, "Expecting ';' after interface declaration")?;
            }
        }

        Some(Statement::Interface { name, declarations, types })
    }

    fn extern_statement(&mut self) -> Option<Statement> {
        self.no_qualifiers("extern");
        let kw = self.previous().clone();

        let mut libraries = Vec::new();

        while !self.check(TokenType::Semicolon) {
            let library = self.consume(TokenType::Identifier, "Expecting library name")?;
            libraries.push(library.clone());

            if !self.match_(&[TokenType::Comma]) {
                break;
            }
        }

        self.consume(TokenType::Semicolon, "Expecting ';' after extern declaration")?;
        Some(Statement::Extern { kw, libraries })
    }

    fn declaration(&mut self, method: bool, incoming_generics: &Vec<Generic>, self_generics: &Vec<Expression>) -> Option<Statement> {
        if self.match_(&[TokenType::Fn]) {
            return self.function(method, incoming_generics, self_generics);
        }

        if self.match_(&[TokenType::Let, TokenType::Const]) {
            return self.var_decl();
        }

        if self.match_(&[TokenType::Struct]) {
            return self.struct_decl(incoming_generics);
        }

        if self.match_(&[TokenType::Impl]) {
            return self.impl_decl();
        }

        if self.match_(&[TokenType::Namespace]) {
            return self.namespace();
        }

        if self.match_(&[TokenType::Use]) {
            return self.use_statement();
        }

        if self.match_(&[TokenType::Enum]) {
            return self.enum_decl(incoming_generics);
        }

        if self.match_(&[TokenType::Import, TokenType::Include]) {
            return self.import_statement();
        }

        if self.match_(&[TokenType::Union]) {
            return self.union_decl();
        }

        if self.match_(&[TokenType::Macro]) {
            return self.macro_decl();
        }

        if self.match_(&[TokenType::Interface]) {
            return self.interface();
        }

        if self.match_(&[TokenType::Extern]) {
            return self.extern_statement();
        }

        if self.match_(&[TokenType::Hash]) {
            let name = {
                if self.match_(&[TokenType::Extern]) {
                    self.previous().clone()
                } else {
                    self.consume(TokenType::Identifier, "Expecting qualifier name after '#'")?.clone()
                }
            };

            let arg = {
                if self.match_(&[TokenType::LeftParen]) {
                    let arg = Some(self.consume(TokenType::Identifier, "Expecting identifier as qualifier argument")?.clone());
                    self.consume(TokenType::RightParen, "Expecting ')' after qualifier argument")?;
                    arg
                } else {
                    None
                }
            };
            
            if let Some(old_tok) = self.curr_qualifiers.get(&name.lexeme) {
                error!(self, name, "Cannot use same qualifier twice");
                note(&old_tok.name, "Previously specified here");
            } else {
                self.curr_qualifiers.insert(Rc::clone(&name.lexeme), Qualifier { name, arg });
            }

            return self.declaration(method, incoming_generics, self_generics);
        }

        self.statement()
    }

    pub fn parse(&mut self) -> Vec<Statement> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            if let Some(stmt) = self.declaration(false, &Vec::new(), &Vec::new()) {
                statements.push(stmt);
            } else {
                self.syncronize();
            }

            self.curr_qualifiers.clear();
        }

        statements
    }
}
