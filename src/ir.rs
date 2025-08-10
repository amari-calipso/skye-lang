use std::{cell::RefCell, collections::HashMap, rc::Rc};

use alanglib::ast::SourcePos;

use crate::{ast::{Bits, Expression, StringKind}, skye_type::SkyeType, tokens::Token, utils::OrderedNamedMap};

#[derive(Clone, Debug)]
pub struct IrStatement {
    pub data: IrStatementData,
    #[allow(unused)] pub pos: SourcePos
}

impl IrStatement {
    pub fn empty_scope(pos: SourcePos) -> Self {
        IrStatement { 
            data: IrStatementData::Scope { statements: Rc::new(RefCell::new(Vec::new())) }, 
            pos 
        }
    }

    pub fn contains_unknown(&self) -> bool {
        match &self.data {
            IrStatementData::Define { value, .. } |
            IrStatementData::Expression { value } => value.contains_unknown(),
            IrStatementData::Struct { type_ } |
            IrStatementData::Union { type_ } => type_.contains_unknown(),
            IrStatementData::Loop { body } => body.contains_unknown(),
            IrStatementData::VarDecl { type_, initializer, .. } => {
                type_.contains_unknown() || initializer.as_ref().map(|x| x.contains_unknown()).unwrap_or(false)
            }
            IrStatementData::Return { value } => {
                value.as_ref().map(|x| x.contains_unknown()).unwrap_or(false)
            }
            IrStatementData::If { condition, then_branch, else_branch } => {
                condition.contains_unknown() || then_branch.contains_unknown() || 
                else_branch.as_ref().map(|x| x.contains_unknown()).unwrap_or(false)
            }
            IrStatementData::Scope { statements } => {
                for statement in statements.borrow().iter() {
                    if statement.contains_unknown() {
                        return true;
                    }
                }

                false
            }
            IrStatementData::TaggedUnion { fields, .. } => {
                for (_, field) in &fields.map {
                    if field.contains_unknown() {
                        return true;
                    }
                }

                false
            }
            IrStatementData::Function { params, body, signature, .. } => {
                if signature.contains_unknown() {
                    return true;
                }

                for param in params {
                    if param.type_.contains_unknown() {
                        return true;
                    }
                }

                if let Some(body) = body {
                    for statement in body {
                        if statement.contains_unknown() {
                            return true;
                        }
                    }
                }

                false
            }
            IrStatementData::Switch { value, branches } => {
                if value.contains_unknown() {
                    return true;
                }

                for branch in branches {
                    if branch.code.contains_unknown() {
                        return true;
                    }

                    for case in &branch.cases {
                        if case.contains_unknown() {
                            return true;
                        }
                    }
                }

                false
            }
            _ => false
        }
    }
}

#[derive(Clone, Debug)]
pub struct IrEnumVariant {
    pub name: Rc<str>,
    pub value: Option<IrValue>
}

#[derive(Clone, Debug)]
pub struct IrFunctionParam {
    pub name: Rc<str>,
    pub type_: SkyeType
}

#[derive(Clone, Debug)]
pub struct IrSwitchBranch {
    pub cases: Vec<IrValue>,
    pub code: IrStatement
}

#[derive(Clone, Debug)]
pub enum VarQualifier {
    Static, Extern, Volatile
}

impl VarQualifier {
    pub fn from_string(qualifier: &str) -> Self {
        match qualifier.to_lowercase().as_str() {
            "static"   => VarQualifier::Static,
            "extern"   => VarQualifier::Extern,
            "volatile" => VarQualifier::Volatile,
            _ => panic!("invalid qualifier")
        }
    }
}

#[derive(Clone, Debug)]
pub enum FnQualifier {
    Static, Extern, Inline
}

impl FnQualifier {
    pub fn from_string(qualifier: &str) -> Self {
        match qualifier.to_lowercase().as_str() {
            "static" => FnQualifier::Static,
            "extern" => FnQualifier::Extern,
            "inline" => FnQualifier::Inline,
            _ => panic!("invalid qualifier")
        }
    }
}

#[derive(Clone, Debug)]
pub enum IrStatementData {
    Break,
    Define { name: Rc<str>, value: IrValue, typedef: bool },
    VarDecl { name: Rc<str>, type_: SkyeType, initializer: Option<IrValue>, qualifiers: Vec<VarQualifier> },
    If { condition: IrValue, then_branch: Box<IrStatement>, else_branch: Option<Box<IrStatement>> },
    Scope { statements: Rc<RefCell<Vec<IrStatement>>> },
    Return { value: Option<IrValue> },
    Expression { value: IrValue },
    Goto { label: Rc<str> },
    Label { name: Rc<str> },
    Function { name: Rc<str>, params: Vec<IrFunctionParam>, body: Option<Vec<IrStatement>>, signature: SkyeType, qualifiers: Vec<FnQualifier> },
    Struct { type_: SkyeType },
    Enum { name: Rc<str>, variants: Vec<IrEnumVariant>, #[allow(unused)] type_: SkyeType },
    TaggedUnion { name: Rc<str>, kind_name: Rc<str>, kind_type: SkyeType, fields: OrderedNamedMap<SkyeType> },
    Union { type_: SkyeType },
    Loop { body: Box<IrStatement> },
    Include { path: Rc<str>, is_ang: bool, flags: Vec<Rc<str>> },
    Switch { value: IrValue, branches: Vec<IrSwitchBranch> }
}

#[derive(Clone, Debug)]
pub struct IrValue {
    pub data: IrValueData,
    pub type_: SkyeType,
}

impl IrValue {
    pub fn new(data: IrValueData, type_: SkyeType) -> Self {
        IrValue { data, type_ }
    }

    pub fn empty_with_type(type_: SkyeType) -> Self {
        IrValue { data: IrValueData::Empty, type_ }
    }

    pub fn any_int(value: i128) -> Self {
        IrValue { 
            data: IrValueData::Literal { value: Expression::SignedIntLiteral { value, tok: Token::empty(), bits: Bits::Any } }, 
            type_: SkyeType::AnyInt
        }
    }

    pub fn uint(value: u64, type_: SkyeType, bits: Bits) -> Self {
        IrValue { 
            data: IrValueData::Literal { value: Expression::UnsignedIntLiteral { value, tok: Token::empty(), bits } }, 
            type_
        }
    }

    pub fn is_empty(&self) -> bool {
        matches!(self.data, IrValueData::Empty)
    }

    pub fn keep_side_effects(&self) -> Self {
        match &self.data {
            IrValueData::Empty => self.clone(),
            // expressions that have side effects
            IrValueData::Call { .. } | 
            IrValueData::Increment { .. } | 
            IrValueData::Assign { .. } |
            IrValueData::Decrement { .. } => self.clone(),
            // expressions that might contain something that has side effects
            IrValueData::Grouping(value) |
            IrValueData::Cast { from: value, .. } |
            IrValueData::Negative { value } | 
            IrValueData::Invert { value } |
            IrValueData::Reference { value } |
            IrValueData::Dereference { value } |
            IrValueData::Get { from: value, .. } |
            IrValueData::DereferenceGet { from: value, .. } | 
            IrValueData::Negate { value } => value.keep_side_effects(),
            // expressions that might contain multiple things that have side effects
            IrValueData::Ternary { condition, then_branch, else_branch  } => {
                let condition = condition.keep_side_effects();
                let then_branch = then_branch.keep_side_effects();
                let else_branch = else_branch.keep_side_effects();

                if !then_branch.is_empty() || !else_branch.is_empty() {
                    // if either of the branches have side effects, then we must keep the entire expression
                    self.clone()
                } else {
                    // if neither do, then we need to keep only the condition (which might be empty due to the check above)
                    condition
                }
            }
            IrValueData::CompoundLiteral { items } => {
                let mut side_effects: Vec<IrValue> = items.iter()
                    .map(|(_, v)| v.keep_side_effects())
                    .filter(|x| !x.is_empty())
                    .collect();

                if side_effects.len() == 0 {
                    let mut output = self.clone();
                    output.data = IrValueData::Empty;
                    output
                } else if side_effects.len() == 1 {
                    side_effects.pop().unwrap()
                } else {
                    self.clone()
                }
            }
            IrValueData::Binary { left, right, .. } |
            IrValueData::Subscript { subscripted: left, index: right } => {
                let left = left.keep_side_effects();
                let right = right.keep_side_effects();

                if left.is_empty() && right.is_empty() {
                    self.clone()
                } else if left.is_empty() {
                    right
                } else {
                    left
                }
            }
            // expressions with no side effects
            _ => {
                let mut output = self.clone();
                output.data = IrValueData::Empty;
                output
            }
        }
    }

    pub fn contains_unknown(&self) -> bool {
        if self.type_.contains_unknown() {
            return true;
        }

        match &self.data {
            IrValueData::Increment { value } | 
            IrValueData::Decrement { value } |
            IrValueData::Negative { value } |
            IrValueData::Invert { value } |
            IrValueData::Negate { value } |
            IrValueData::Reference { value } |
            IrValueData::Dereference { value } | 
            IrValueData::Get { from: value, .. } | 
            IrValueData::DereferenceGet { from: value, .. } |
            IrValueData::Grouping(value) => value.contains_unknown(),
            IrValueData::Cast { to, from } => {
                to.contains_unknown() || from.contains_unknown()
            }
            IrValueData::Subscript { subscripted: left, index: right } |
            IrValueData::Binary { left, right, .. } | 
            IrValueData::Assign { target: left, value: right, .. } => {
                left.contains_unknown() || right.contains_unknown()
            }
            IrValueData::Ternary { condition, then_branch, else_branch } => {
                condition.contains_unknown() || then_branch.contains_unknown() || else_branch.contains_unknown()
            }
            IrValueData::Call { callee, args } => {
                if callee.contains_unknown() {
                    return true;
                }

                for arg in args {
                    if arg.contains_unknown() {
                        return true;
                    }
                }

                false
            }
            IrValueData::Slice { items } |
            IrValueData::Array { items } => {
                for item in items {
                    if item.contains_unknown() {
                        return true;
                    }
                }

                false
            }
            IrValueData::CompoundLiteral { items } => {
                for (_, item) in items {
                    if item.contains_unknown() {
                        return true;
                    }
                }

                false
            }
            _ => false
        }
    }

    pub fn is_valid_assignment_target(&self) -> bool {
        match &self.data {
            IrValueData::Grouping(inner) => inner.is_valid_assignment_target(),
            IrValueData::Literal { value } => {
                if let Expression::StringLiteral { kind, .. } = value {
                    matches!(kind, StringKind::Slice | StringKind::C)
                } else {
                    false
                }
            }
            IrValueData::Variable { .. } |
            IrValueData::Subscript { .. } |
            IrValueData::Dereference { .. } |
            IrValueData::Get { .. } |
            IrValueData::DereferenceGet { .. } |
            IrValueData::CompoundLiteral { .. } => true,
            _ => false
        }
    }
}

#[derive(Clone, Debug)]
pub enum TypeKind {
    Struct, Enum, Union
}

#[derive(Clone, Debug)]
pub enum IrValueData {
    Empty,
    Literal { value: Expression },
    Variable { name: Rc<str> },
    Cast { to: SkyeType, from: Box<IrValue> },
    Call { callee: Box<IrValue>, args: Vec<IrValue> },
    Subscript { subscripted: Box<IrValue>, index: Box<IrValue> },
    Increment { value: Box<IrValue> }, 
    Decrement { value: Box<IrValue> },
    Negative { value: Box<IrValue> },
    Invert { value: Box<IrValue> },
    Negate { value: Box<IrValue> },
    Reference { value: Box<IrValue> },
    Dereference { value: Box<IrValue> },
    Slice { items: Vec<IrValue> },
    Array { items: Vec<IrValue> },
    Ternary { condition: Box<IrValue>, then_branch: Box<IrValue>, else_branch: Box<IrValue> },
    Get { from: Box<IrValue>, name: Rc<str> },
    DereferenceGet { from: Box<IrValue>, name: Rc<str> }, // arrow operator
    CompoundLiteral { items: HashMap<Rc<str>, IrValue> },
    Grouping(Box<IrValue>), // TODO: remove this and automatically figure out grouping in backend
    Binary { left: Box<IrValue>, op: BinaryOp, right: Box<IrValue> },
    Assign { target: Box<IrValue>, op: AssignOp, value: Box<IrValue> },
    TypeRef { kind: TypeKind, name: Rc<str> } // struct Foo, enum Foo, union Foo...
}

#[derive(Clone, Debug)]
pub enum BinaryOp {
    Add, Subtract, Divide, Multiply, Modulo, 
    ShiftLeft, ShiftRight,
    BitwiseXor, BitwiseOr, BitwiseAnd, 
    Greater, GreaterEqual, Less, LessEqual,
    Equal, NotEqual
}

#[derive(Clone, Debug)]
pub enum AssignOp {
    None, Add, Subtract, Divide, Multiply, Modulo, 
    ShiftLeft, ShiftRight, 
    BitwiseXor, BitwiseOr, BitwiseAnd, 
}