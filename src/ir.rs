use std::{collections::HashMap, rc::Rc};

use crate::{ast::{AstPos, Expression}, skye_type::SkyeType};

#[derive(Clone, Debug)]
pub struct IrStatement {
    pub data: IrStatementData,
    pub pos: AstPos
}

#[derive(Clone, Debug)]
pub struct IrEnumVariant {
    pub name: Rc<str>,
    pub value: Option<IrValue>
}

#[derive(Clone, Debug)]
pub enum IrStatementData {
    Define { name: Rc<str>, value: IrValue, typedef: bool },
    Undefine { name: Rc<str> },
    VarDecl { name: Rc<str>, type_: SkyeType, initializer: Option<IrValue> }, // TODO: add qualifiers
    If { condition: IrValue, then_branch: Box<IrStatement>, else_branch: Option<Box<IrStatement>> },
    Scope { statements: Vec<IrStatement> },
    Return { value: Option<IrValue> },
    Expression { value: IrValue },
    Goto { label: Rc<str> },
    Label { name: Rc<str> },
    Function { name: Rc<str>, type_: SkyeType }, // TODO: add qualifiers
    Struct { name: Rc<str>, type_: SkyeType },
    Enum { name: Rc<str>, variants: Vec<IrEnumVariant> },
    TaggedUnion { name: Rc<str>, kind_name: Rc<str>, fields: HashMap<Rc<str>, SkyeType> },
    Union { name: Rc<str>, fields: HashMap<Rc<str>, SkyeType> },
    Loop { body: Box<IrStatement> }
    // TODO: switch
    // TODO: native import
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
    Slice { type_: SkyeType, items: Vec<IrValue> },
    Array { items: Vec<IrValue> },
    Ternary { condition: Box<IrValue>, then_branch: Box<IrValue>, else_branch: Box<IrValue> },
    Get { from: Box<IrValue>, name: Rc<str> },
    DereferenceGet { from: Box<IrValue>, name: Rc<str> }, // arrow operator
    // TODO: compound literal
    Grouping(Box<IrValue>), // TODO: remove this and automatically figure out grouping in backend
    Binary { left: Box<IrValue>, op: BinaryOp, right: Box<IrValue> },
    Assign { target: Box<IrValue>, op: AssignOp, value: Box<IrValue> },
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