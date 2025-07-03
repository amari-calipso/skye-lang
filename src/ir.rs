use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{ast::{AstPos, Bits, Expression}, type_system::SkyeType, parser::tokens::Token};

#[derive(Clone, Debug)]
pub struct IrStatement {
    pub data: IrStatementData,
    pub pos: AstPos
}

impl IrStatement {
    pub fn empty_scope(pos: AstPos) -> Self {
        IrStatement { 
            data: IrStatementData::Scope { statements: Rc::new(RefCell::new(Vec::new())) }, 
            pos 
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
pub enum IrStatementData {
    Break,
    Define { name: Rc<str>, value: IrValue, typedef: bool },
    Undefine { name: Rc<str> },
    VarDecl { name: Rc<str>, type_: SkyeType, initializer: Option<IrValue> }, // TODO: add qualifiers
    If { condition: IrValue, then_branch: Box<IrStatement>, else_branch: Option<Box<IrStatement>> },
    Scope { statements: Rc<RefCell<Vec<IrStatement>>> },
    Return { value: Option<IrValue> },
    Expression { value: IrValue },
    Goto { label: Rc<str> },
    Label { name: Rc<str> },
    Function { name: Rc<str>, params: Vec<IrFunctionParam>, body: Option<Vec<IrStatement>>, return_type: SkyeType }, // TODO: add qualifiers
    Struct { type_: SkyeType },
    Enum { name: Rc<str>, variants: Vec<IrEnumVariant>, type_: SkyeType },
    TaggedUnion { name: Rc<str>, kind_name: Rc<str>, fields: HashMap<Rc<str>, SkyeType> },
    Union { type_: SkyeType },
    Loop { body: Box<IrStatement> },
    Include { path: Rc<str>, is_ang: bool },
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

    pub fn empty() -> Self {
        IrValue { data: IrValueData::Empty, type_: SkyeType::Void }
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
            // TODO if one of the items has side effects, keep that item. 
            //      if more than one has side effects, keep whole expression.
            //      if none of the items have side effects, return empty
            IrValueData::Ternary { .. } |
            IrValueData::CompoundLiteral { .. } | 
            IrValueData::Binary { .. } |
            IrValueData::Subscript { .. } => self.clone(),
            // expressions with no side effects
            _ => {
                let mut output = self.clone();
                output.data = IrValueData::Empty;
                output
            }
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
    ShiftLeft, ShiftRight, LogicOr, LogicAnd,
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