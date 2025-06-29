use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{ast::{AstPos, Bits, Expression}, skye_type::SkyeType, tokens::Token};

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
pub enum IrStatementData {
    Define { name: Rc<str>, value: IrValue, typedef: bool },
    Undefine { name: Rc<str> },
    VarDecl { name: Rc<str>, type_: SkyeType, initializer: Option<IrValue> }, // TODO: add qualifiers
    If { condition: IrValue, then_branch: Box<IrStatement>, else_branch: Option<Box<IrStatement>> },
    Scope { statements: Rc<RefCell<Vec<IrStatement>>> },
    Return { value: Option<IrValue> },
    Expression { value: IrValue },
    Goto { label: Rc<str> },
    Label { name: Rc<str> },
    Function { name: Rc<str>, body: Option<Vec<IrStatement>>, type_: SkyeType }, // TODO: add qualifiers
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
    CompoundLiteral { type_: SkyeType, items: HashMap<Rc<str>, IrValue> },
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