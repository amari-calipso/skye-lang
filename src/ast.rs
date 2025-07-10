use std::rc::Rc;

use crate::tokens::{Token, TokenType};

#[derive(Clone, PartialEq, PartialOrd)]
pub struct AstPos {
    pub source: Rc<str>,
    pub filename: Rc<str>,
    pub start: usize,
    pub end: usize,
    pub line: usize
}

impl std::fmt::Debug for AstPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AstPos").field("filename", &self.filename).field("start", &self.start).field("end", &self.end).field("line", &self.line).finish()
    }
}

impl AstPos {
    pub fn new(source: Rc<str>, filename: Rc<str>, start: usize, end: usize, line: usize) -> Self {
        AstPos { source, filename, start, end, line }
    }

    pub fn empty() -> Self {
        AstPos { source: Rc::from(""), filename: Rc::from(""), start: 0, end: 1, line: 0 }
    }
}

pub trait Ast {
    type Output;
    fn get_pos(&self) -> AstPos;
    fn replace_variable(&self, name: &Rc<str>, replace_expr: &Expression) -> Self::Output;
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Bits {
    B8, B16, B32, B64, Bsz, Any
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum StringKind {
    Slice, Raw, Char
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct FunctionParam {
    pub name: Option<Token>,
    pub type_: Expression,
    pub is_const: bool
}

impl FunctionParam {
    pub fn new(name: Option<Token>, type_: Expression, is_const: bool) -> Self {
        FunctionParam { name, type_, is_const }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct StructField {
    pub name: Token,
    pub expr: Expression,
    pub bits: Option<Expression>,
    pub is_const: bool
}

impl StructField {
    pub fn new(name: Token, expr: Expression, bits: Option<Expression>, is_const: bool) -> Self {
        StructField { name, expr, bits, is_const }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct EnumVariant {
    pub name: Token,
    pub type_: Expression,
    pub default: Option<Expression>,
}

impl EnumVariant {
    pub fn new(name: Token, type_: Expression, default: Option<Expression>) -> Self {
        EnumVariant { name, type_, default }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct SwitchCase {
    pub cases: Option<Vec<Expression>>, // is none when default
    pub code: Vec<Statement>
}

impl SwitchCase {
    pub fn new(cases: Option<Vec<Expression>>, code: Vec<Statement>) -> Self {
        SwitchCase { cases, code }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Expression {
    Binary { left: Box<Expression>, op: Token, right: Box<Expression> },
    SignedIntLiteral { value: i128, tok: Token, bits: Bits },
    UnsignedIntLiteral { value: u64, tok: Token, bits: Bits },
    FloatLiteral { value: f64, tok: Token, bits: Bits },
    StringLiteral { value: Rc<str>, tok: Token, kind: StringKind },
    VoidLiteral(Token),
    Unary { op: Token, expr: Box<Expression>, is_prefix: bool },
    Grouping(Box<Expression>),
    Variable(Token), // name
    Assign { target: Box<Expression>, op: Token, value: Box<Expression> },
    Call(Box<Expression>, Token, Vec<Expression>, bool), // callee paren arguments unpack
    FnPtr { kw: Token, return_type: Box<Expression>, params: Vec<FunctionParam> },
    Ternary { tok: Token, condition: Box<Expression>, then_expr: Box<Expression>, else_expr: Box<Expression> },
    CompoundLiteral { type_: Box<Expression>, closing_brace: Token, fields: Vec<StructField> },
    Subscript { subscripted: Box<Expression>, paren: Token, args: Vec<Expression> },
    Get(Box<Expression>, Token), // object name
    StaticGet(Option<Box<Expression>>, Token, bool), // object name gets_macro
    Slice { opening_brace: Token, items: Vec<Expression> },
    InMacro { inner: Box<Expression>, source: AstPos },
    MacroExpandedStatements { inner: Vec<Statement>, source: AstPos },
    Array { opening_brace: Token, item: Box<Expression>, size: Box<Expression> },
    ArrayLiteral { opening_brace: Token, items: Vec<Expression> },
}

impl Ast for Expression {
    type Output = Expression;

    fn get_pos(&self) -> AstPos {
        match self {
            Expression::Grouping(expr) => expr.get_pos(),
            Expression::InMacro { source, .. } | Expression::MacroExpandedStatements { source, .. } => source.clone(),
            Expression::SignedIntLiteral { tok, .. } | 
            Expression::UnsignedIntLiteral { tok, .. } | 
            Expression::FloatLiteral { tok, .. } |
            Expression::StringLiteral { tok, .. } | 
            Expression::VoidLiteral(tok) | 
            Expression::Variable(tok) => {
                AstPos::new(Rc::clone(&tok.source), Rc::clone(&tok.filename), tok.pos, tok.end, tok.line)
            }
            Expression::Binary { left, op, right } => {
                let left_pos = left.get_pos();
                let right_pos = right.get_pos();

                if left_pos.line != right_pos.line || left_pos.filename != right_pos.filename {
                    left_pos
                } else {
                    AstPos::new(Rc::clone(&op.source), Rc::clone(&op.filename), left_pos.start, right_pos.end, left_pos.line)
                }
            }
            Expression::Unary { op, expr, is_prefix } => {
                let expr_pos = expr.get_pos();

                if *is_prefix {
                    if op.line != expr_pos.line || op.filename != expr_pos.filename {
                        AstPos::new(Rc::clone(&op.source), Rc::clone(&op.filename), op.pos, op.end, op.line)
                    } else {
                        AstPos::new(Rc::clone(&op.source), Rc::clone(&op.filename), op.pos, expr.get_pos().end, op.line)
                    }
                } else {
                    if expr_pos.line != op.line || op.filename != expr_pos.filename {
                        expr_pos
                    } else {
                        AstPos::new(Rc::clone(&expr_pos.source), Rc::clone(&expr_pos.filename), expr_pos.start, op.end, expr_pos.line)
                    }
                }
            }
            Expression::Assign { target, op: _, value } => {
                let target_pos = target.get_pos();
                let value_pos = value.get_pos();

                if target_pos.line != value_pos.line || target_pos.filename != value_pos.filename {
                    target_pos
                } else {
                    AstPos::new(Rc::clone(&target_pos.source), Rc::clone(&target_pos.filename), target_pos.start, value_pos.end, target_pos.line)
                }
            }
            Expression::Call(callee, paren, ..) => {
                let callee_pos = callee.get_pos();

                if callee_pos.line != paren.line || callee_pos.filename != paren.filename {
                    callee_pos
                } else {
                    AstPos::new(Rc::clone(&callee_pos.source), Rc::clone(&callee_pos.filename), callee_pos.start, paren.end, callee_pos.line)
                }
            }
            Expression::FnPtr { kw, return_type, params: _ } => {
                let return_type_pos = return_type.get_pos();

                if kw.line != return_type_pos.line || kw.filename != return_type_pos.filename {
                    AstPos::new(Rc::clone(&kw.source), Rc::clone(&kw.filename), kw.pos, kw.end, kw.line)
                } else {
                    AstPos::new(Rc::clone(&kw.source), Rc::clone(&kw.filename), kw.pos, return_type_pos.end, kw.line)
                }
            }
            Expression::Ternary { tok: _, condition: cond, then_expr: _, else_expr: else_ } => {
                let cond_pos = cond.get_pos();
                let else_pos = else_.get_pos();

                if cond_pos.line != else_pos.line || cond_pos.filename != else_pos.filename {
                    cond_pos
                } else {
                    AstPos::new(Rc::clone(&cond_pos.source), Rc::clone(&cond_pos.filename), cond_pos.start, else_pos.end, cond_pos.line)
                }
            }
            Expression::CompoundLiteral { type_: struct_, closing_brace, fields: _ } => {
                let struct_pos = struct_.get_pos();

                if struct_pos.line != closing_brace.line || struct_pos.filename != closing_brace.filename {
                    struct_pos
                } else {
                    AstPos::new(Rc::clone(&struct_pos.source), Rc::clone(&struct_pos.filename), struct_pos.start, closing_brace.end, struct_pos.line)
                }
            }
            Expression::Subscript { subscripted, paren, args: _ } => {
                let subscripted_pos = subscripted.get_pos();

                if subscripted_pos.line != paren.line || subscripted_pos.filename != paren.filename {
                    subscripted_pos
                } else {
                    AstPos::new(Rc::clone(&subscripted_pos.source), Rc::clone(&subscripted_pos.filename), subscripted_pos.start, paren.end, subscripted_pos.line)
                }
            }
            Expression::Get(object, name) => {
                let object_pos = object.get_pos();

                if object_pos.line != name.line || object_pos.filename != name.filename {
                    AstPos::new(Rc::clone(&name.source), Rc::clone(&name.filename), name.pos, name.end, name.line)
                } else {
                    AstPos::new(Rc::clone(&object_pos.source), Rc::clone(&object_pos.filename), object_pos.start, name.end, object_pos.line)
                }
            }
            Expression::StaticGet(object, name, _) => {
                let object_pos = {
                    if let Some(object) = object {
                        object.get_pos()
                    } else {
                        name.get_pos()
                    }
                };

                if object_pos.line != name.line || object_pos.filename != name.filename {
                    AstPos::new(Rc::clone(&name.source), Rc::clone(&name.filename), name.pos, name.end, name.line)
                } else {
                    AstPos::new(Rc::clone(&object_pos.source), Rc::clone(&object_pos.filename), object_pos.start, name.end, object_pos.line)
                }
            }
            Expression::Slice { items: exprs, .. } |
            Expression::ArrayLiteral { items: exprs, .. } => {
                match exprs.len() {
                    0 => unreachable!(), // guaranteed by parser
                    1 => exprs[0].get_pos(),
                    _ => {
                        let first_pos = exprs[0].get_pos();
                        let last_pos = exprs.last().unwrap().get_pos();

                        if first_pos.line != last_pos.line || first_pos.filename != last_pos.filename {
                            first_pos
                        } else {
                            AstPos::new(Rc::clone(&first_pos.source), Rc::clone(&first_pos.filename), first_pos.start, last_pos.end, first_pos.line)
                        }
                    }
                }
            }
            Expression::Array { opening_brace, .. } => {
                AstPos::new(Rc::clone(&opening_brace.source), Rc::clone(&opening_brace.filename), opening_brace.pos, opening_brace.end, opening_brace.line)
            }
        }
    }

    fn replace_variable(&self, name: &Rc<str>, replace_expr: &Expression) -> Expression {
        match self {
            Expression::Grouping(expr) |
            Expression::InMacro { inner: expr, .. } => expr.replace_variable(name, replace_expr),
            Expression::SignedIntLiteral { .. } |
            Expression::UnsignedIntLiteral { .. } | 
            Expression::FloatLiteral { .. } |
            Expression::StringLiteral { .. } | 
            Expression::VoidLiteral(_) => self.clone(),
            Expression::Variable(tok) => {
                if tok.lexeme.as_ref() == name.as_ref() {
                    Expression::Grouping(Box::new(replace_expr.clone()))
                } else {
                    self.clone()
                }
            }
            Expression::Binary { left, op, right } => {
                Expression::Binary { left: Box::new(left.replace_variable(name, replace_expr)), op: op.clone(), right: Box::new(right.replace_variable(name, replace_expr)) }
            }
            Expression::Unary { op, expr, is_prefix } => {
                Expression::Unary { op: op.clone(), expr: Box::new(expr.replace_variable(name, replace_expr)), is_prefix: *is_prefix }
            }
            Expression::Assign { target, op, value } => {
                Expression::Assign { target: Box::new(target.replace_variable(name, replace_expr)), op: op.clone(), value: Box::new(value.replace_variable(name, replace_expr)) }
            }
            Expression::Ternary { tok: question, condition: cond, then_expr: then, else_expr: else_ } => {
                Expression::Ternary { tok: question.clone(), condition: Box::new(cond.replace_variable(name, replace_expr)), then_expr: Box::new(then.replace_variable(name, replace_expr)), else_expr: Box::new(else_.replace_variable(name, replace_expr)) }
            }
            Expression::Call(callee, paren, args, unpack) => {
                Expression::Call(
                    Box::new(callee.replace_variable(name, replace_expr)),
                    paren.clone(),
                    args.iter().map(|x| x.replace_variable(name, replace_expr)).collect(),
                    *unpack
                )
            }
            Expression::FnPtr { kw, return_type, params } => {
                Expression::FnPtr { kw: kw.clone(), return_type: Box::new(return_type.replace_variable(name, replace_expr)), params: params.iter().map(
                        |x| FunctionParam::new(
                            x.name.clone(), x.type_.replace_variable(name, replace_expr), x.is_const
                        )
                    ).collect() }
            }
            Expression::CompoundLiteral { type_: struct_, closing_brace, fields } => {
                Expression::CompoundLiteral { type_: Box::new(struct_.replace_variable(name, replace_expr)), closing_brace: closing_brace.clone(), fields: fields.iter().map(
                        |x| StructField::new(
                            x.name.clone(), x.expr.replace_variable(name, replace_expr), 
                            x.bits.clone().map(|x| x.replace_variable(name, replace_expr)), x.is_const
                        )
                    ).collect() }
            }
            Expression::Subscript { subscripted, paren, args } => {
                Expression::Subscript { subscripted: Box::new(subscripted.replace_variable(name, replace_expr)), paren: paren.clone(), args: args.iter().map(|x| x.replace_variable(name, replace_expr)).collect() }
            }
            Expression::Get(object, get_name) => {
                Expression::Get(Box::new(object.replace_variable(name, replace_expr)), get_name.clone())
            }
            Expression::StaticGet(object, get_name, gets_macro) => {
                Expression::StaticGet(object.as_ref().map(|x| Box::new(x.replace_variable(name, replace_expr))), get_name.clone(), *gets_macro)
            }
            Expression::Slice { opening_brace, items } => {
                Expression::Slice { opening_brace: opening_brace.clone(), items: items.iter().map(|x| x.replace_variable(name, replace_expr)).collect() }
            }
            Expression::ArrayLiteral { opening_brace, items } => {
                Expression::ArrayLiteral { opening_brace: opening_brace.clone(), items: items.iter().map(|x| x.replace_variable(name, replace_expr)).collect() }
            }
            Expression::MacroExpandedStatements { inner: statements, source } => {
                Expression::MacroExpandedStatements {
                    inner: statements.iter().map(|x| x.replace_variable(name, replace_expr)).collect(),
                    source: source.clone()
                }
            }
            Expression::Array { opening_brace, item, size } => {
                Expression::Array {
                    opening_brace: opening_brace.clone(),
                    item: Box::new(item.replace_variable(name, replace_expr)),
                    size: Box::new(size.replace_variable(name, replace_expr))
                }
            }
        }
    }
}

impl Expression {
    pub fn get_inner(&self) -> Expression {
        match self {
            Expression::Grouping(inner) |
            Expression::InMacro { inner, .. } => *inner.clone(),
            _ => self.clone()
        }
    }

    pub fn is_valid_assignment_target(&self) -> bool {
        match self {
            Expression::Variable(_) | Expression::Get(..) | Expression::StaticGet(..) | Expression::Subscript { .. } => true,
            Expression::Unary { op, expr: _, is_prefix } => *is_prefix && op.type_ == TokenType::Star,
            Expression::Grouping(inner) => inner.is_valid_assignment_target(),
            _ => false
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum ImportType {
    Default,
    Ang,
    Lib
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Generic {
    pub name: Token,
    pub bounds: Option<Expression>,
    pub default: Option<Expression>
}

impl Generic {
    pub fn new(name: Token, bounds: Option<Expression>, default: Option<Expression>) -> Self {
        Generic { name, bounds, default }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum MacroParams {
    None,
    Some(Vec<Token>),
    OneN(Token),
    ZeroN(Token)
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum MacroBody {
    Binding(Expression),
    Expression(Expression),
    Block(Vec<Statement>)
}

impl MacroBody {
    pub fn replace_variable(&self, name: &Rc<str>, replace_expr: &Expression) -> Self {
        match self {
            MacroBody::Binding(expression) => MacroBody::Binding(expression.replace_variable(name, replace_expr)),
            MacroBody::Expression(expression) => MacroBody::Expression(expression.replace_variable(name, replace_expr)),
            MacroBody::Block(statements) => {
                MacroBody::Block(statements.iter().map(|x| x.replace_variable(name, replace_expr)).collect())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Statement {
    Empty, // placeholder
    Expression(Expression),
    Break(Token), // kw
    Continue(Token), // kw
    Block(Token, Vec<Statement>), // kw statements
    ImportedBlock { statements: Vec<Statement>, source: AstPos },
    While { kw: Token, condition: Expression, body: Box<Statement> },
    DoWhile { kw: Token, condition: Expression, body: Box<Statement> },
    Return { kw: Token, value: Option<Expression> },
    Impl { object: Expression, declarations: Vec<Statement> },
    Namespace { name: Token, body: Vec<Statement> },
    Defer { kw: Token, statement: Box<Statement> },
    Switch { kw: Token, expr: Expression, cases: Vec<SwitchCase> },
    Import { path: Token, type_: ImportType, is_include: bool },
    Macro { name: Token, params: MacroParams, body: MacroBody },
    VarDecl {
        name: Token,
        initializer: Option<Expression>,
        type_: Option<Expression>,
        is_const: bool,
        qualifiers: Vec<Token>
    },
    If {
        kw: Token,
        condition: Expression,
        then_branch: Box<Statement>,
        else_branch: Option<Box<Statement>>
    },
    For {
        kw: Token,
        initializer: Option<Box<Statement>>,
        condition: Expression,
        increments: Vec<Expression>,
        body: Box<Statement>
    },
    Function {
        name: Token,
        params: Vec<FunctionParam>,
        return_type: Expression,
        body: Option<Vec<Statement>>,
        qualifiers: Vec<Token>,
        generics_names: Vec<Token>,
        bind: bool,
        init: bool,
    },
    Struct {
        name: Token,
        fields: Vec<StructField>,
        has_body: bool,
        binding: Option<Token>,
        generics_names: Vec<Token>,
        bind_typedefed: bool
    },
    Use {
        use_expr: Expression,
        as_name: Token,
        typedef: bool,
        bind: bool
    },
    Enum {
        name: Token,
        kind_type: Expression,
        variants: Vec<EnumVariant>,
        is_simple: bool,
        has_body: bool,
        binding: Option<Token>,
        generics_names: Vec<Token>,
        bind_typedefed: bool
    },
    Template {
        name: Token,
        declaration: Box<Statement>,
        generics: Vec<Generic>,
        generics_names: Vec<Token>
    },
    Union {
        name: Token,
        fields: Vec<StructField>,
        has_body: bool,
        binding: Option<Token>,
        bind_typedefed: bool
    },
    Foreach {
        kw: Token,
        variable_name: Token,
        iterator: Expression,
        body: Box<Statement>
    },
    Interface {
        name: Token,
        declarations: Option<Vec<Statement>>,
        types: Option<Vec<Expression>>
    },
    Extern {
        kw: Token,
        libraries: Vec<Token>
    }
}

impl Ast for Statement {
    type Output = Statement;

    fn get_pos(&self) -> AstPos {
        match self {
            Statement::ImportedBlock { source, .. } => source.clone(), 
            Statement::Empty => {
                AstPos::new(Rc::from(""), Rc::from(""), 0, 0, 0)
            }
            Statement::Expression(expr) |
            Statement::Impl { object: expr, declarations: _ } |
            Statement::Use { use_expr: expr, .. } => expr.get_pos(),

            Statement::VarDecl { name: tok, .. } |
            Statement::Block(tok, _) |
            Statement::If { kw: tok, .. } |
            Statement::While { kw: tok, .. } |
            Statement::For { kw: tok, .. } |
            Statement::DoWhile { kw: tok, .. } |
            Statement::Function { name: tok, .. } |
            Statement::Return { kw: tok, .. } |
            Statement::Struct { name: tok, .. } |
            Statement::Namespace { name: tok, .. } |
            Statement::Enum { name: tok, .. } |
            Statement::Defer { kw: tok, .. } |
            Statement::Switch { kw: tok, .. } |
            Statement::Template { name: tok, .. } |
            Statement::Break(tok) |
            Statement::Continue(tok) |
            Statement::Import { path: tok, .. } |
            Statement::Union { name: tok, .. } |
            Statement::Macro { name: tok, .. } |
            Statement::Foreach { kw: tok, .. } |
            Statement::Interface { name: tok, .. } |
            Statement::Extern { kw: tok, .. } => {
                AstPos::new(Rc::clone(&tok.source), Rc::clone(&tok.filename), tok.pos, tok.end, tok.line)
            }
        }
    }

    fn replace_variable(&self, name: &Rc<str>, replace_expr: &Expression) -> Statement {
        match self {
            Statement::Empty | 
            Statement::Break(_) | 
            Statement::Continue(_) |
            Statement::Import { .. } | 
            Statement::Extern { .. } => self.clone(),

            Statement::Expression(expression) => Statement::Expression(expression.replace_variable(name, replace_expr)),
            Statement::VarDecl { name: var_name, initializer, type_, is_const, qualifiers } => {
                Statement::VarDecl {
                    name: var_name.clone(),
                    initializer: initializer.as_ref().map(|x| x.replace_variable(name, replace_expr)),
                    type_: type_.as_ref().map(|x| x.replace_variable(name, replace_expr)),
                    is_const: *is_const,
                    qualifiers: qualifiers.clone()
                }
            }
            Statement::Block(kw, statements) => {
                Statement::Block(kw.clone(), statements.iter().map(|x| x.replace_variable(name, replace_expr)).collect())
            }
            Statement::ImportedBlock { statements, source } => {
                Statement::ImportedBlock { 
                    statements: statements.iter().map(|x| x.replace_variable(name, replace_expr)).collect(),
                    source: source.clone()
                }
            }
            Statement::If { kw, condition: cond, then_branch, else_branch } => {
                Statement::If {
                    kw: kw.clone(),
                    condition: cond.replace_variable(name, replace_expr),
                    then_branch: Box::new(then_branch.replace_variable(name, replace_expr)),
                    else_branch: else_branch.as_ref().map(|x| Box::new(x.replace_variable(name, replace_expr)))
                }
            }
            Statement::While { kw, condition: cond, body } => {
                Statement::While {
                    kw: kw.clone(),
                    condition: cond.replace_variable(name, replace_expr),
                    body: Box::new(body.replace_variable(name, replace_expr))
                }
            }
            Statement::DoWhile { kw, condition: cond, body } => {
                Statement::DoWhile {
                    kw: kw.clone(),
                    condition: cond.replace_variable(name, replace_expr),
                    body: Box::new(body.replace_variable(name, replace_expr))
                }
            }
            Statement::For { kw, initializer, condition: cond, increments: increment, body } => {
                Statement::For {
                    kw: kw.clone(),
                    initializer: initializer.as_ref().map(|x| Box::new(x.replace_variable(name, replace_expr))),
                    condition: cond.replace_variable(name, replace_expr),
                    increments: increment.iter().map(|x| x.replace_variable(name, replace_expr)).collect(),
                    body: Box::new(body.replace_variable(name, replace_expr))
                }
            }
            Statement::Function { name: kw, params, return_type, body, qualifiers, generics_names, bind, init, } => {
                Statement::Function {
                    name: kw.clone(),
                    params: params.iter().map(|x| FunctionParam::new(x.name.clone(), x.type_.replace_variable(name, replace_expr), x.is_const)).collect(),
                    return_type: return_type.replace_variable(name, replace_expr),
                    body: body.as_ref().map(|x| x.iter().map(|statement| statement.replace_variable(name, replace_expr)).collect()),
                    qualifiers: qualifiers.clone(),
                    generics_names: generics_names.clone(),
                    bind: *bind,
                    init: *init
                }
            }
            Statement::Return { kw, value: return_expr } => {
                Statement::Return { kw: kw.clone(), value: return_expr.as_ref().map(|x| x.replace_variable(name, replace_expr)) }
            }
            Statement::Struct { name: struct_name, fields, has_body, binding, generics_names, bind_typedefed } => {
                Statement::Struct {
                    name: struct_name.clone(),
                    fields: {
                        fields.iter().map(|x| 
                            StructField::new(
                                x.name.clone(), 
                                x.expr.replace_variable(name, replace_expr), 
                                x.bits.clone().map(|x| x.replace_variable(name, replace_expr)),
                                x.is_const
                            )
                        ).collect()
                    },
                    has_body: *has_body,
                    binding: binding.clone(),
                    generics_names: generics_names.clone(),
                    bind_typedefed: *bind_typedefed
                }
            }
            Statement::Impl { object: struct_, declarations } => {
                Statement::Impl { object: struct_.replace_variable(name, replace_expr), declarations: declarations.iter().map(|x| x.replace_variable(name, replace_expr)).collect() }
            }
            Statement::Namespace { name: namespace_name, body: declarations } => {
                Statement::Namespace { name: namespace_name.clone(), body: declarations.iter().map(|x| x.replace_variable(name, replace_expr)).collect() }
            }
            Statement::Use { use_expr: expression, as_name: alias, typedef, bind } => {
                Statement::Use { use_expr: expression.replace_variable(name, replace_expr), as_name: alias.clone(), typedef: *typedef, bind: *bind }
            }
            Statement::Enum { name: enum_name, kind_type, variants, is_simple, has_body, binding, generics_names, bind_typedefed } => {
                Statement::Enum {
                    name: enum_name.clone(),
                    kind_type: kind_type.replace_variable(name, replace_expr),
                    variants: variants.iter().map(|x| EnumVariant::new(x.name.clone(), x.type_.replace_variable(name, replace_expr), x.default.clone())).collect(),
                    is_simple: *is_simple,
                    has_body: *has_body,
                    binding: binding.clone(),
                    generics_names: generics_names.clone(),
                    bind_typedefed: *bind_typedefed
                }
            }
            Statement::Defer { kw, statement } => {
                Statement::Defer { kw: kw.clone(), statement: Box::new(statement.replace_variable(name, replace_expr)) }
            }
            Statement::Switch { kw, expr: cond, cases } => {
                Statement::Switch { kw: kw.clone(), expr: cond.replace_variable(name, replace_expr), cases: cases.iter().map(
                        |x| SwitchCase::new(
                            x.cases.as_ref().map(
                                |orig_cases| orig_cases.iter().map(
                                    |case| case.replace_variable(name, replace_expr)
                                ).collect()
                            ),
                            x.code.iter().map(|x| x.replace_variable(name, replace_expr)).collect()
                        )
                    ).collect() }
            }
            Statement::Template { name: template_name, declaration, generics, generics_names } => {
                Statement::Template { name: template_name.clone(), declaration: Box::new(declaration.replace_variable(name, replace_expr)), generics: generics.iter().map(
                        |x| Generic::new(
                            x.name.clone(),
                            x.bounds.as_ref().map(|bounds| bounds.replace_variable(name, replace_expr)),
                            x.default.as_ref().map(|default| default.replace_variable(name, replace_expr))
                        )
                    ).collect(), generics_names: generics_names.clone() }
            }
            Statement::Union { name: union_name, fields, has_body, binding, bind_typedefed } => {
                Statement::Union {
                    name: union_name.clone(),
                    fields: {
                        fields.iter().map(|x| 
                            StructField::new(
                                x.name.clone(), 
                                x.expr.replace_variable(name, replace_expr), 
                                x.bits.clone().map(|x| x.replace_variable(name, replace_expr)),
                                x.is_const
                            )
                        ).collect()
                    },
                    has_body: *has_body,
                    binding: binding.clone(),
                    bind_typedefed: *bind_typedefed
                }
            }
            Statement::Macro { name: macro_name, params: macro_params, body: macro_body } => {
                Statement::Macro { name: macro_name.clone(), params: macro_params.clone(), body: macro_body.replace_variable(name, replace_expr) }
            }
            Statement::Foreach { kw, variable_name: var_name, iterator, body } => {
                Statement::Foreach { kw: kw.clone(), variable_name: var_name.clone(), iterator: iterator.replace_variable(name, replace_expr), body: Box::new(body.replace_variable(name, replace_expr)) }
            }
            Statement::Interface { name: interface_name, declarations, types } => {
                Statement::Interface { name: interface_name.clone(), declarations: declarations.as_ref().map(
                        |decls| decls.iter().map(
                            |x| x.replace_variable(name, replace_expr)
                        ).collect()
                    ), types: types.as_ref().map(
                        |types_unwrapped| types_unwrapped.iter().map(
                            |x| x.replace_variable(name, replace_expr)
                        ).collect()
                    ) }
            }
        }
    }
}