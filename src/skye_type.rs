use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{ast::{Generic, MacroBody, MacroParams, Statement}, environment::Environment, ir::{IrValue, IrValueData}, tokens::Token};

#[derive(Debug, Clone, PartialEq)]
pub struct SkyeFunctionParam {
    pub type_: SkyeType,
    pub is_const: bool
}

impl SkyeFunctionParam {
    pub fn new(type_: SkyeType, is_const: bool) -> Self {
        SkyeFunctionParam { type_, is_const }
    }
}

#[derive(Debug, Clone)]
pub enum GetResultInternal {
    Ok(IrValue, SkyeType, bool), // value holder_type is_const
    InvalidType,
    FieldNotFound
}

#[derive(Debug, Clone)]
pub enum GetResult {
    Ok(IrValue, bool), // value is_const
    InvalidType,
    FieldNotFound
}

#[derive(Debug, Clone, PartialEq)]
pub struct SkyeEnumVariant {
    pub name: Token,
    pub type_: SkyeType
}

impl SkyeEnumVariant {
    pub fn new(name: Token, type_: SkyeType) -> Self {
        SkyeEnumVariant { name, type_ }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum Operator {
    Inc, Dec, Neg,
    Not, Inv,
    Ref, ConstRef,
    Deref, ConstDeref,
    Add, Sub, Div, Mul, Mod,
    Shl, Shr,
    Or, And,
    BitOr, BitAnd, Xor,
    Gt, Ge, Lt, Le, Eq, Ne,
    SetAdd, SetSub, SetMul, SetDiv, SetMod,
    SetShl, SetShr,
    SetAnd, SetXor, SetOr,
    Subscript
}

pub enum ImplementsHow {
    Native(Vec<SkyeType>),
    ThirdParty,
    No
}

pub enum CastableHow {
    Yes,
    No,
    ConstnessLoss
}

#[derive(Clone, Copy)]
pub enum EqualsLevel {
    ConstStrict,
    Strict,
    Typewise,
    Permissive
}

#[derive(Clone)]
pub struct SkyeValue {
    pub ir_value: IrValue,
    pub is_const: bool,
    pub self_info: Option<IrValue>
}

impl SkyeValue {
    pub fn new(ir_value: IrValue, is_const: bool) -> Self {
        SkyeValue { ir_value, is_const, self_info: None }
    }

    pub fn special(type_: SkyeType) -> Self {
        SkyeValue { ir_value: IrValue::empty_with_type(type_), is_const: true, self_info: None }
    }

    pub fn with_self_info(ir_value: IrValue, is_const: bool, self_info: IrValue) -> Self {
        SkyeValue { ir_value, is_const, self_info: Some(self_info) }
    }

    pub fn follow_reference(&self, mut zero_check: Box<impl FnMut(SkyeValue) -> IrValue>) -> Self {
        self.ir_value.type_.follow_reference(self.is_const, &self.ir_value, &mut zero_check)
    }

    pub fn get_unknown() -> SkyeValue {
        SkyeValue { ir_value: IrValue::empty_with_type(SkyeType::get_unknown()), is_const: false, self_info: None }
    }

    pub fn get_unknown_type() -> SkyeValue {
        SkyeValue { ir_value: IrValue::empty_with_type(SkyeType::get_unknown_type()), is_const: false, self_info: None }
    }
}

const ALL_INTS: &[SkyeType] = &[
    SkyeType::U8, SkyeType::U16, SkyeType::U32, SkyeType::U64, SkyeType::Usz,
    SkyeType::I8, SkyeType::I16, SkyeType::I32, SkyeType::I64, SkyeType::AnyInt
];

#[derive(Debug, Clone, PartialEq)]
pub struct SkyeField {
    pub type_: SkyeType,
    pub bits: Option<u64>,
    pub is_const: bool,
}

#[derive(Clone, PartialEq)]
pub enum SkyeType {
    U8, U16, U32, U64, Usz,
    I8, I16, I32, I64, AnyInt,
    F32, F64, AnyFloat,
    Char,

    Void,
    Unknown(Rc<str>), // used for type inference

    Pointer(Box<SkyeType>, bool, bool), // type is_const is_reference
    Array(Box<SkyeType>, usize), // type size
    Type(Box<SkyeType>),
    Group(Box<SkyeType>, Box<SkyeType>), // left right
    Function(Vec<SkyeFunctionParam>, Box<SkyeType>, bool), // params return_type has_body
    Struct(Rc<str>, Option<HashMap<Rc<str>, SkyeField>>, Rc<str>), // name fields base_name
    Namespace(Rc<str>), // name
    Enum(Rc<str>, Option<HashMap<Rc<str>, SkyeType>>, Rc<str>), // name variants base_name
    Template(Rc<str>, Statement, Vec<Generic>, Vec<Token>, String, Rc<RefCell<Environment>>), // name definition generics generics_names curr_name environment
    Union(Rc<str>, Option<HashMap<Rc<str>, SkyeField>>), // name fields
    Macro(Rc<str>, MacroParams, MacroBody), // name params body
}

impl std::fmt::Debug for SkyeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::U8 => write!(f, "U8"),
            Self::U16 => write!(f, "U16"),
            Self::U32 => write!(f, "U32"),
            Self::U64 => write!(f, "U64"),
            Self::Usz => write!(f, "Usz"),
            Self::I8 => write!(f, "I8"),
            Self::I16 => write!(f, "I16"),
            Self::I32 => write!(f, "I32"),
            Self::I64 => write!(f, "I64"),
            Self::AnyInt => write!(f, "AnyInt"),
            Self::F32 => write!(f, "F32"),
            Self::F64 => write!(f, "F64"),
            Self::AnyFloat => write!(f, "AnyFloat"),
            Self::Char => write!(f, "Char"),
            Self::Void => write!(f, "Void"),
            Self::Unknown(arg0) => f.debug_tuple("Unknown").field(arg0).finish(),
            Self::Pointer(arg0, arg1, arg2) => f.debug_tuple("Pointer").field(arg0).field(arg1).field(arg2).finish(),
            Self::Array(arg0, arg1) => f.debug_tuple("Array").field(arg0).field(arg1).finish(),
            Self::Type(arg0) => f.debug_tuple("Type").field(arg0).finish(),
            Self::Group(arg0, arg1) => f.debug_tuple("Group").field(arg0).field(arg1).finish(),
            Self::Function(arg0, arg1, arg2) => f.debug_tuple("Function").field(arg0).field(arg1).field(arg2).finish(),
            Self::Struct(arg0, arg1, arg2) => f.debug_tuple("Struct").field(arg0).field(arg1).field(arg2).finish(),
            Self::Namespace(arg0) => f.debug_tuple("Namespace").field(arg0).finish(),
            Self::Enum(arg0, arg1, arg2) => f.debug_tuple("Enum").field(arg0).field(arg1).field(arg2).finish(),
            Self::Template(arg0, arg1, arg2, arg3, arg4, _) => f.debug_tuple("Template").field(arg0).field(arg1).field(arg2).field(arg3).field(arg4).finish(),
            Self::Union(arg0, arg1) => f.debug_tuple("Union").field(arg0).field(arg1).finish(),
            Self::Macro(arg0, arg1, arg2) => f.debug_tuple("Macro").field(arg0).field(arg1).field(arg2).finish(),
        }
    }
}

impl SkyeType {
    pub fn stringify(&self) -> String {
        match self {
            SkyeType::U8  => String::from("u8"),
            SkyeType::I8  => String::from("i8"),
            SkyeType::U16 => String::from("u16"),
            SkyeType::I16 => String::from("i16"),
            SkyeType::U32 => String::from("u32"),
            SkyeType::U64 => String::from("u64"),
            SkyeType::I64 => String::from("i64"),
            SkyeType::F64 => String::from("f64"),
            SkyeType::Usz => String::from("usz"),
            SkyeType::I32 => String::from("i32"),
            SkyeType::F32 => String::from("f32"),
            SkyeType::AnyInt   => String::from("AnyInt"),
            SkyeType::AnyFloat => String::from("AnyFloat"),

            SkyeType::Char => String::from("char"),
            SkyeType::Void => String::from("void"),

            SkyeType::Group(left, right) => format!("{} | {}", left.stringify(), right.stringify()),
            SkyeType::Template(name, ..) => format!("template \"{}\"", name.replace("_DOT_", "::")),
            SkyeType::Namespace(name) => format!("namespace \"{}\"", name.replace("_DOT_", "::")),
            SkyeType::Macro(name, ..) => format!("macro \"{}\"", name),
            SkyeType::Unknown(name) => {
                if name.as_ref() == "" {
                    String::from("any")
                } else {
                    format!("any \"{}\"", name)
                }
            }

            SkyeType::Type(inner) => format!("type \"{}\"", inner.stringify()),
            SkyeType::Function(args, return_type, _) => {
                let mut buf = String::from("fn (");
                for (i, arg) in args.iter().enumerate() {
                    if arg.is_const {
                        buf.push_str("const ");
                    }

                    buf.push_str(&arg.type_.stringify());

                    if i != args.len() - 1 {
                        buf.push_str(", ");
                    }
                }

                buf.push_str(") ");
                buf.push_str(&return_type.stringify());
                buf
            }

            SkyeType::Pointer(inner, is_const, is_reference) => {
                let sym = {
                    if *is_reference {
                        '&'
                    } else {
                        '*'
                    }
                };

                if *is_const {
                    String::from(format!("{}const {}", sym, inner.stringify()))
                } else {
                    String::from(format!("{}{}", sym, inner.stringify()))
                }
            }

            SkyeType::Array(inner, size) => {
                format!("[{}; {}]", inner.stringify(), *size)
            }

            SkyeType::Struct(name, ..) |
            SkyeType::Enum(name, ..) => {
                // not ideal, but it's just error reporting ¯\_(ツ)_/¯
                name.to_string()
                    .replace("_DOT_", "::")
                    .replace("_FNPTR_", "fn (")
                    .replace("_PARAM_AND_", ", ")
                    .replace("_PARAM_END_", ") ")
                    .replace("_FNPTR_END_", "")
                    .replace("_GENOF_", "[")
                    .replace("_GENAND_", ", ")
                    .replace("_GENEND_", "]")
                    .replace("_UNKNOWN_", "{any}")
            }

            SkyeType::Union(name, _) => name.to_string().replace("_DOT_", "::"),
        }
    }

    pub fn mangle(&self) -> String {
        match self {
            SkyeType::U8  => String::from("u8"),
            SkyeType::I8  => String::from("i8"),
            SkyeType::U16 => String::from("u16"),
            SkyeType::I16 => String::from("i16"),
            SkyeType::U32 => String::from("u32"),
            SkyeType::U64 => String::from("u64"),
            SkyeType::I64 => String::from("i64"),
            SkyeType::F64 => String::from("f64"),
            SkyeType::Usz => String::from("usz"),
            SkyeType::I32 | SkyeType::AnyInt   => String::from("i32"),
            SkyeType::F32 | SkyeType::AnyFloat => String::from("f32"),

            SkyeType::Char       => String::from("char"),
            SkyeType::Void       => String::from("void"),
            SkyeType::Unknown(_) => String::from("_UNKNOWN_"),

            SkyeType::Group(..) |
            SkyeType::Namespace(_) |
            SkyeType::Template(..) |
            SkyeType::Macro(..) => String::new(),

            SkyeType::Type(inner) => inner.mangle(),
            SkyeType::Struct(name, ..) |
            SkyeType::Enum(name, ..) |
            SkyeType::Union(name, _) => name.to_string(),

            SkyeType::Pointer(inner, ..) => {
                let inner_mangled = inner.mangle();
                if inner_mangled.len() == 0 {
                    return inner_mangled;
                }

                String::from(format!("_PTROF_{}_PTREND_", inner_mangled))
            }

            SkyeType::Array(..) => self.mangle(),

            SkyeType::Function(params, return_type, _) => {
                let ret_type_mangled = return_type.mangle();
                if ret_type_mangled.len() == 0 {
                    return ret_type_mangled;
                }

                let mut output_params = String::new();
                for i in 0 .. params.len() {
                    let param = params[i].type_.mangle();
                    if param.len() == 0 {
                        return param;
                    }

                    output_params.push_str(&param);

                    if i != params.len() - 1 {
                        output_params.push_str("_PARAM_AND_");
                    }
                }

                String::from(format!("_FNPTR_{}_PARAM_END_{}_FNPTR_END_", output_params, ret_type_mangled))
            }
        }
    }

    pub fn equals(&self, other: &SkyeType, level: EqualsLevel) -> bool {
        if matches!(other, SkyeType::Unknown(_)) {
            return true;
        }

        match self {
            SkyeType::U8  => matches!(other, SkyeType::U8  | SkyeType::AnyInt),
            SkyeType::I8  => matches!(other, SkyeType::I8  | SkyeType::AnyInt),
            SkyeType::U16 => matches!(other, SkyeType::U16 | SkyeType::AnyInt),
            SkyeType::I16 => matches!(other, SkyeType::I16 | SkyeType::AnyInt),
            SkyeType::U32 => matches!(other, SkyeType::U32 | SkyeType::AnyInt),
            SkyeType::I32 => matches!(other, SkyeType::I32 | SkyeType::AnyInt),
            SkyeType::U64 => matches!(other, SkyeType::U64 | SkyeType::AnyInt),
            SkyeType::I64 => matches!(other, SkyeType::I64 | SkyeType::AnyInt),
            SkyeType::Usz => matches!(other, SkyeType::Usz | SkyeType::AnyInt),
            SkyeType::F32 => matches!(other, SkyeType::F32 | SkyeType::AnyFloat),
            SkyeType::F64 => matches!(other, SkyeType::F64 | SkyeType::AnyFloat),
            SkyeType::AnyInt   => matches!(other, SkyeType::AnyInt)   || (!matches!(other, SkyeType::AnyFloat) && other.equals(self, level)),
            SkyeType::AnyFloat => matches!(other, SkyeType::AnyFloat) || (!matches!(other, SkyeType::AnyInt)   && other.equals(self, level)),

            SkyeType::Char => matches!(other, SkyeType::Char),
            SkyeType::Void => matches!(other, SkyeType::Void),

            SkyeType::Group(..) |
            SkyeType::Namespace(_) |
            SkyeType::Template(..) |
            SkyeType::Macro(..) => false,

            SkyeType::Unknown(_) => true,

            SkyeType::Type(self_inner) => {
                if let SkyeType::Type(other_inner) = other {
                    self_inner.equals(other_inner, level)
                } else {
                    false
                }
            }
            SkyeType::Pointer(self_inner, self_is_const, _) => {
                match level {
                    EqualsLevel::Typewise => {
                        if let SkyeType::Pointer(other_inner, ..) = other {
                            self_inner.equals(other_inner, level)
                        } else {
                            false
                        }
                    }
                    EqualsLevel::ConstStrict => {
                        if let SkyeType::Pointer(other_inner, other_is_const, _) = other {
                            !(self_is_const ^ other_is_const) && self_inner.equals(other_inner, level)
                        } else {
                            false
                        }
                    }
                    _ => {
                        if let SkyeType::Pointer(other_inner, other_is_const, _) = other {
                            if *self_is_const {
                                self_inner.equals(other_inner, level)
                            } else {
                                (!*other_is_const) && self_inner.equals(other_inner, level)
                            }
                        } else {
                            false
                        }
                    }
                }
            }
            SkyeType::Array(self_inner, self_size) => {
                if let SkyeType::Array(other_inner, other_size) = other {
                    *self_size == *other_size && self_inner.equals(&other_inner, level)
                } else {
                    false
                }
            }
            SkyeType::Function(self_params, self_return_type, _) => {
                if let SkyeType::Function(other_params, other_return_type, _) = other {
                    if self_params.len() != other_params.len() || !self_return_type.equals(other_return_type, level) {
                        false
                    } else {
                        for i in 0..self_params.len() {
                            if matches!(level, EqualsLevel::ConstStrict) && (self_params[i].is_const ^ other_params[i].is_const) {
                                return false;
                            }

                            if !self_params[i].type_.equals(&other_params[i].type_, level) {
                                return false;
                            }
                        }

                        true
                    }
                } else {
                    false
                }
            }
            SkyeType::Struct(self_name, _, self_base_name) => {
                if matches!(level, EqualsLevel::Permissive) {
                    if let SkyeType::Struct(.., other_base_name) = other {
                        self_base_name == other_base_name
                    } else {
                        false
                    }
                } else {
                    if let SkyeType::Struct(other_name, ..) = other {
                        self_name == other_name
                    } else {
                        false
                    }
                }
            }
            SkyeType::Enum(self_name, _, self_base_name) => {
                if matches!(level, EqualsLevel::Permissive) {
                    if let SkyeType::Enum(.., other_base_name) = other {
                        self_base_name == other_base_name
                    } else {
                        false
                    }
                } else {
                    if let SkyeType::Enum(other_name, ..) = other {
                        self_name == other_name
                    } else {
                        false
                    }
                }
            }
            SkyeType::Union(self_name, _) => {
                if let SkyeType::Union(other_name, _) = other {
                    self_name == other_name
                } else {
                    false
                }
            }
        }
    }

    // checks if `other` respects `self`, where `self` is a generic type bound
    pub fn is_respected_by(&self, other: &SkyeType) -> bool {
        match self {
            SkyeType::Group(left, right) => {
                left.is_respected_by(other) || right.is_respected_by(other)
            }
            _ => self.equals(other, EqualsLevel::Typewise)
        }
    }

    pub fn is_type(&self) -> bool {
        matches!(self, SkyeType::Type(_) | SkyeType::Group(..))
    }

    pub fn finalize(&self) -> SkyeType {
        match self {
            SkyeType::AnyInt   => SkyeType::I32,
            SkyeType::AnyFloat => SkyeType::F32,
            SkyeType::Type(inner) => SkyeType::Type(Box::new(inner.finalize())),
            _ => self.clone()
        }
    }

    fn get_internal(&self, from: &IrValue, name: &Token, is_source_const: bool, d: usize, zero_check: &mut Box<impl FnMut(SkyeValue) -> IrValue>) -> GetResultInternal {
        match self {
            SkyeType::Pointer(inner_type, is_pointer_const, is_reference) => {
                if !*is_reference {
                    return GetResultInternal::InvalidType;
                }

                let inner_res = inner_type.get_internal(from, name, *is_pointer_const, d + 1, zero_check);
                if let GetResultInternal::Ok(inner_val, holder_type, is_const) = inner_res {
                    let mut tmp_var_type = holder_type.clone();
                    for _ in 0 ..= d {
                        tmp_var_type = SkyeType::Pointer(Box::new(tmp_var_type), false, false);
                    }

                    let inner_final = zero_check(SkyeValue::new(IrValue::new(inner_val.data.clone(), tmp_var_type), is_const));

                    if d == 0 {
                        if let SkyeType::Pointer(..) = **inner_type {
                            GetResultInternal::Ok(
                                IrValue::new(
                                    IrValueData::DereferenceGet { 
                                        from: Box::new(IrValue { 
                                            type_: inner_final.type_.clone(), 
                                            data: IrValueData::Grouping(Box::new(inner_final)), 
                                        }), 
                                        name: Rc::clone(&name.lexeme)
                                    }, 
                                    inner_val.type_
                                ), 
                                holder_type, *is_pointer_const || is_const
                            )
                        } else {
                            GetResultInternal::Ok(
                                IrValue::new(
                                    IrValueData::DereferenceGet { from: Box::new(inner_final), name: Rc::clone(&name.lexeme) }, 
                                    inner_val.type_
                                ), 
                                holder_type, *is_pointer_const || is_const
                            )
                        }
                    } else {
                        GetResultInternal::Ok(
                            IrValue::new(IrValueData::Dereference { value: Box::new(inner_final) }, inner_val.type_), 
                            holder_type, *is_pointer_const || is_const
                        )
                    }
                } else {
                    inner_res
                }
            }
            SkyeType::Struct(_, fields, _) |
            SkyeType::Union(_, fields) => {
                if let Some(defined_fields) = fields {
                    if let Some(field) = defined_fields.get(&name.lexeme) {
                        if d == 0 {
                            GetResultInternal::Ok(
                                IrValue::new(
                                    IrValueData::Get { from: Box::new(from.clone()), name: Rc::clone(&name.lexeme) }, 
                                    field.type_.clone()
                                ), 
                                self.clone(), is_source_const || field.is_const
                            )
                        } else {
                            GetResultInternal::Ok(IrValue::new(from.data.clone(), field.type_.clone()), self.clone(), is_source_const || field.is_const)
                        }
                    } else {
                        GetResultInternal::FieldNotFound
                    }
                } else {
                    GetResultInternal::FieldNotFound
                }
            }
            SkyeType::Enum(_, fields, _) => {
                if let Some(defined_fields) = fields {
                    if let Some(field) = defined_fields.get(&name.lexeme) {
                        if d == 0 {
                            GetResultInternal::Ok(
                                IrValue::new(
                                    IrValueData::Get { from: Box::new(from.clone()), name: Rc::clone(&name.lexeme) }, 
                                    field.clone()
                                ), 
                                self.clone(), true
                            )
                        } else {
                            GetResultInternal::Ok(IrValue::new(from.data.clone(), field.clone()), self.clone(), true)
                        }
                    } else {
                        GetResultInternal::FieldNotFound
                    }
                } else {
                    GetResultInternal::InvalidType
                }
            }
            _ => GetResultInternal::InvalidType
        }
    }

    pub fn get(&self, from: &IrValue, name: &Token, is_source_const: bool, mut zero_check: Box<impl FnMut(SkyeValue) -> IrValue>) -> GetResult {
        match self.get_internal(from, name, is_source_const, 0, &mut zero_check) {
            GetResultInternal::Ok(value, _, is_const) => GetResult::Ok(value, is_const),
            GetResultInternal::InvalidType => GetResult::InvalidType,
            GetResultInternal::FieldNotFound => GetResult::FieldNotFound,
        }
    }

    fn static_get_internal(&self, name: &Token, d: usize) -> Option<Rc<str>> {
        match self {
            SkyeType::Pointer(inner_type, ..) => inner_type.static_get_internal(name, d + 1),
            SkyeType::Type(inner_type) => {
                if d == 0 {
                    inner_type.static_get_internal(name, d + 1)
                } else {
                    None
                }
            }
            SkyeType::Namespace(namespace_name) |
            SkyeType::Struct(.., namespace_name) |
            SkyeType::Enum(.., namespace_name) |
            SkyeType::Template(namespace_name, ..) => {
                Some(format!("{}_DOT_{}", namespace_name, name.lexeme).into())
            }
            _ => None
        }
    }

    pub fn static_get(&self, name: &Token) -> Option<Rc<str>> {
        self.static_get_internal(name, 0)
    }

    pub fn get_method(&self, name: &Token, strict: bool) -> Option<Rc<str>> {
        match self {
            SkyeType::Pointer(inner_type, _, is_reference) => {
                if strict && !*is_reference {
                    None
                } else {
                    inner_type.get_method(name, strict)
                }
            }
            SkyeType::Struct(.., obj_name) |
            SkyeType::Enum(.., obj_name) |
            SkyeType::Template(obj_name, ..) => {
                Some(format!("{}_DOT_{}", obj_name, name.lexeme).into())
            }
            _ => None
        }
    }

    fn get_self_internal(&self, from: &IrValue, d: usize, zero_check: &mut Box<impl FnMut(SkyeValue) -> IrValue>) -> Option<IrValue> {
        match self {
            SkyeType::Pointer(ptr_type, is_const, _) => {
                let inner = ptr_type.get_self_internal(from, d + 1, zero_check)?;

                if d == 0 {
                    Some(IrValue::new(inner.data, SkyeType::Pointer(Box::new(inner.type_), *is_const, true)))
                } else {
                    let mut tmp_var_type = inner.type_.clone();
                    for _ in 0 ..= d {
                        tmp_var_type = SkyeType::Pointer(Box::new(tmp_var_type), false, false);
                    }

                    let inner_final = zero_check(SkyeValue::new(IrValue { type_: tmp_var_type, data: inner.data }, *is_const));
                    Some(IrValue::new(
                        IrValueData::Dereference { value: Box::new(inner_final) },
                        inner.type_
                    ))
                }
            }
            SkyeType::Struct(..) | SkyeType::Enum(..) => Some(IrValue::new(from.data.clone(), self.clone())),
            _ => None
        }
    }

    pub fn get_self(&self, from: &IrValue, is_source_const: bool, mut zero_check: Box<impl FnMut(SkyeValue) -> IrValue>) -> Option<IrValue> {
        if let SkyeType::Pointer(..) = self {
            self.get_self_internal(from, 0, &mut zero_check)
        } else {
            Some(IrValue::new(
                IrValueData::Reference { value: Box::new(from.clone()) }, 
                SkyeType::Pointer(Box::new(self.clone()), is_source_const, true)
            ))
        }
    }

    fn infer_type_from_similar_internal(&self, other: &SkyeType, data: Rc<RefCell<HashMap<Rc<str>, SkyeType>>>) -> Option<()> {
        if !self.equals(other, EqualsLevel::Permissive) {
            return None;
        }

        match self {
            SkyeType::U8  | SkyeType::I8  | SkyeType::U16 | SkyeType::I16 |
            SkyeType::U32 | SkyeType::I32 | SkyeType::U64 | SkyeType::I64 |
            SkyeType::Usz | SkyeType::F32 | SkyeType::F64 | SkyeType::AnyInt |
            SkyeType::AnyFloat | SkyeType::Char | SkyeType::Void |
            SkyeType::Group(..) | SkyeType::Namespace(_) | SkyeType::Template(..) |
            SkyeType::Macro(..) => (),

            SkyeType::Unknown(name) => {
                if let SkyeType::Pointer(inner_type, _, is_reference) = other {
                    data.borrow_mut().insert(Rc::clone(name), SkyeType::Pointer(inner_type.clone(), false, *is_reference));
                } else {
                    data.borrow_mut().insert(Rc::clone(name), other.clone());
                }
            }

            SkyeType::Pointer(self_inner_type, ..) |
            SkyeType::Array(self_inner_type, ..) |
            SkyeType::Type(self_inner_type) => {
                match other {
                    SkyeType::Pointer(other_inner_type, ..) |
                    SkyeType::Array(other_inner_type, ..) |
                    SkyeType::Type(other_inner_type) => {
                        self_inner_type.infer_type_from_similar_internal(other_inner_type, data)?;
                    }
                    SkyeType::Unknown(_) => (),
                    _ => unreachable!()
                }
            }

            SkyeType::Function(self_params, self_return, _) => {
                if let SkyeType::Function(other_params, other_return, _) = other {
                    for i in 0 .. self_params.len() {
                        self_params[i].type_.infer_type_from_similar_internal(&other_params[i].type_, Rc::clone(&data))?;
                    }

                    self_return.infer_type_from_similar_internal(&other_return, data)?;
                } else if !matches!(other, SkyeType::Unknown(_)) {
                    unreachable!()
                }
            }

            SkyeType::Struct(_, self_fields, _) => {
                if let SkyeType::Struct(_, other_fields, _) = other {
                    if let Some(real_self_fields) = self_fields {
                        if let Some(real_other_fields) = other_fields {
                            for (key, self_field) in real_self_fields {
                                if let Some(other_field) = real_other_fields.get(key) {
                                    self_field.type_.infer_type_from_similar_internal(&other_field.type_, Rc::clone(&data))?;
                                }
                            }
                        }
                    }
                } else if !matches!(other, SkyeType::Unknown(_)) {
                    unreachable!()
                }
            }
            SkyeType::Union(_, self_fields) => {
                match other {
                    SkyeType::Union(_, other_fields) => {
                        if let Some(real_self_fields) = self_fields {
                            if let Some(real_other_fields) = other_fields {
                                for (key, self_field) in real_self_fields {
                                    if let Some(other_field) = real_other_fields.get(key) {
                                        self_field.type_.infer_type_from_similar_internal(&other_field.type_, Rc::clone(&data))?;
                                    }
                                }
                            }
                        }
                    }
                    SkyeType::Unknown(_) => (),
                    _ => unreachable!()
                }
            }
            SkyeType::Enum(_, self_fields, _) => {
                match other {
                    SkyeType::Enum(_, other_fields, _) => {
                        if let Some(real_self_fields) = self_fields {
                            if let Some(real_other_fields) = other_fields {
                                if real_self_fields.len() >= real_other_fields.len() {
                                    for (key, value) in real_self_fields {
                                        if let Some(field) = real_other_fields.get(key) {
                                            value.infer_type_from_similar_internal(field, Rc::clone(&data))?;
                                        } else {
                                            // if variant is not there in enum and they are equal, it means that the variant type is void
                                            value.infer_type_from_similar_internal(&SkyeType::Void, Rc::clone(&data))?;
                                        }
                                    }
                                } else {
                                    for (key, value) in real_other_fields {
                                        if let Some(field) = real_self_fields.get(key) {
                                            value.infer_type_from_similar_internal(field, Rc::clone(&data))?;
                                        } else {
                                            value.infer_type_from_similar_internal(&SkyeType::Void, Rc::clone(&data))?;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    SkyeType::Unknown(_) => (),
                    _ => unreachable!()
                }
            }
        }

        Some(())
    }

    pub fn infer_type_from_similar(&self, other: &SkyeType) -> Option<HashMap<Rc<str>, SkyeType>> {
        let data = Rc::new(RefCell::new(HashMap::new()));
        self.infer_type_from_similar_internal(other, Rc::clone(&data))?;
        let result = data.borrow().clone();
        Some(result)
    }

    pub fn implements_op(&self, op: Operator) -> ImplementsHow {
        match self {
            SkyeType::Unknown(_) => ImplementsHow::Native(Vec::new()),
            SkyeType::Pointer(..) => {
                match op {
                    Operator::Add | Operator::Sub | Operator::Div | Operator::Mul | Operator::Mod |
                    Operator::Eq  | Operator::Ne => ImplementsHow::Native(ALL_INTS.into()),
                    _ => ImplementsHow::Native(Vec::new())
                }
            }

            // at this stage, the compiler can't know whether the operator is implemented or not,
            // so it assumes it is, that way it can try to find the relative function
            SkyeType::Template(..) | SkyeType::Struct(..) => ImplementsHow::ThirdParty,

            SkyeType::Enum(_, variants, _) => {
                if variants.is_none() {
                    if matches!(op, Operator::Eq | Operator::Ne) {
                        ImplementsHow::Native(ALL_INTS.into())
                    } else {
                        ImplementsHow::No
                    }
                } else {
                    ImplementsHow::ThirdParty
                }
            }

            SkyeType::Void | SkyeType::Type(_) | SkyeType::Group(..) |
            SkyeType::Namespace(_) | SkyeType::Macro(..) | SkyeType::Array(..) => {
                ImplementsHow::No
            }

            SkyeType::Union(..) | SkyeType::Function(..) => {
                if matches!(op, Operator::Ref | Operator::ConstRef) {
                    ImplementsHow::Native(Vec::new())
                } else {
                    ImplementsHow::No
                }
            }

            SkyeType::U8  | SkyeType::I8  | SkyeType::U16 | SkyeType::I16 |
            SkyeType::U32 | SkyeType::I32 | SkyeType::U64 | SkyeType::I64 |
            SkyeType::Usz | SkyeType::AnyInt => {
                match op {
                    Operator::Subscript | Operator::Deref | Operator::ConstDeref => ImplementsHow::No,
                    _ => ImplementsHow::Native(vec![SkyeType::Char])
                }
            }

            SkyeType::F32 | SkyeType::F64 | SkyeType::AnyFloat => {
                match op {
                    Operator::Subscript | Operator::Deref | Operator::ConstDeref | 
                    Operator::Inv | Operator::Not | Operator::Or | Operator::And => ImplementsHow::No,
                    Operator::Mod | Operator::SetMod => ImplementsHow::ThirdParty,
                    _ => ImplementsHow::Native(Vec::new())
                }
            }

            SkyeType::Char => {
                match op {
                    Operator::Subscript | Operator::Deref | Operator::ConstDeref => ImplementsHow::No,
                    _ => ImplementsHow::Native(vec![SkyeType::AnyInt, SkyeType::U8, SkyeType::I8])
                }
            }
        }
    }

    pub fn check_completeness(&self) -> bool {
        match self {
            SkyeType::Type(inner) => inner.check_completeness(),

            SkyeType::U8  | SkyeType::U16 | SkyeType::U32 | SkyeType::U64 | SkyeType::Usz |
            SkyeType::I8  | SkyeType::I16 | SkyeType::I32 | SkyeType::I64 | SkyeType::AnyInt |
            SkyeType::F32 | SkyeType::F64 | SkyeType::AnyFloat |
            SkyeType::Char | SkyeType::Void | SkyeType::Unknown(_) |
            SkyeType::Pointer(..) | SkyeType::Function(..) | SkyeType::Enum(..) | 
            SkyeType::Array(..) => true,

            SkyeType::Group(..) | SkyeType::Namespace(_) | SkyeType::Template(..) |
            SkyeType::Macro(..) => false,

            SkyeType::Struct(_, fields, _) => fields.is_some(),
            SkyeType::Union(_, fields) => fields.is_some()
        }
    }

    pub fn is_castable_to(&self, cast_to: &SkyeType) -> CastableHow {
        match self {
            SkyeType::Void | SkyeType::Type(_) | SkyeType::Group(..) | SkyeType::Function(..) |
            SkyeType::Struct(..) | SkyeType::Namespace(_) | SkyeType::Template(..) |
            SkyeType::Union(..) | SkyeType::Macro(..) => CastableHow::No,
            SkyeType::Unknown(_) => CastableHow::Yes,

            SkyeType::AnyFloat | SkyeType::F32 | SkyeType::F64 | SkyeType::Char => {
                if matches!(
                    cast_to,
                    SkyeType::F32 |
                    SkyeType::F64 |
                    SkyeType::AnyFloat |
                    SkyeType::Char
                ) || ALL_INTS.contains(cast_to) {
                    CastableHow::Yes
                } else {
                    CastableHow::No
                }
            }
            SkyeType::U8 | SkyeType::U16 | SkyeType::U32 | SkyeType::U64 |
            SkyeType::I8 | SkyeType::I16 | SkyeType::I32 | SkyeType::I64 |
            SkyeType::AnyInt | SkyeType::Usz => {
                if matches!(
                    cast_to,
                    SkyeType::F32 |
                    SkyeType::F64 |
                    SkyeType::AnyFloat |
                    SkyeType::Char
                ) || ALL_INTS.contains(cast_to) {
                    CastableHow::Yes
                } else if let SkyeType::Pointer(.., is_reference) = cast_to {
                    if *is_reference {
                        CastableHow::No
                    } else {
                        CastableHow::Yes
                    }
                } else {
                    CastableHow::No
                }
            }

            SkyeType::Pointer(_, self_is_const, _) => {
                if matches!(cast_to, SkyeType::Usz) {
                    CastableHow::Yes
                } else if let SkyeType::Pointer(_, cast_to_const, _) = cast_to {
                    if *cast_to_const || !*self_is_const {
                        CastableHow::Yes
                    } else {
                        CastableHow::ConstnessLoss
                    }
                } else {
                    CastableHow::No
                }
            }

            SkyeType::Array(..) => {
                if let SkyeType::Pointer(.., is_reference) = cast_to {
                    if *is_reference {
                        CastableHow::No
                    } else {
                        CastableHow::Yes
                    }
                } else {
                    CastableHow::No
                }
            }

            SkyeType::Enum(_, variants, _) => {
                if variants.is_none() && ALL_INTS.contains(cast_to) {
                    CastableHow::Yes
                } else {
                    CastableHow::No
                }
            }
        }
    }

    fn follow_reference_internal(&self, is_source_const: bool, from: &IrValue, d: usize, zero_check: &mut Box<impl FnMut(SkyeValue) -> IrValue>) -> SkyeValue {
        match self {
            SkyeType::Pointer(inner_type, is_const, is_reference) => {
                if *is_reference {
                    let value = inner_type.follow_reference_internal(*is_const, from, d + 1, zero_check);

                    let mut tmp_var_type = value.ir_value.type_.clone();
                    for _ in 0 ..= d {
                        tmp_var_type = SkyeType::Pointer(Box::new(tmp_var_type), false, false);
                    }

                    let final_value = zero_check(SkyeValue::new(IrValue { type_: tmp_var_type, data: value.ir_value.data }, value.is_const));
                    SkyeValue::new(
                        IrValue::new(
                            IrValueData::Dereference { value: Box::new(final_value) }, 
                            value.ir_value.type_
                        ), 
                        value.is_const
                    )
                } else {
                    SkyeValue::new(IrValue::new(from.data.clone(), self.clone()), is_source_const)
                }
            }
            _ => SkyeValue::new(IrValue::new(from.data.clone(), self.clone()), is_source_const)
        }
    }

    pub fn follow_reference(&self, is_source_const: bool, from: &IrValue, zero_check: &mut Box<impl FnMut(SkyeValue) -> IrValue>) -> SkyeValue {
        self.follow_reference_internal(is_source_const, from, 0, zero_check)
    }

    pub fn can_be_instantiated(&self, as_type: bool) -> bool {
        match self {
            SkyeType::Group(..) | SkyeType::Namespace(_) | SkyeType::Template(..) | SkyeType::Macro(..) => false,
            SkyeType::Void => as_type,
            SkyeType::Type(inner) => {
                if as_type {
                    inner.can_be_instantiated(as_type)
                } else {
                    false
                }
            }
            _ => true
        }
    }

    pub fn contains_unknown(&self) -> bool {
        match self {
            SkyeType::Unknown(_) => true,
            SkyeType::Pointer(inner, _, _) | SkyeType::Array(inner, _) |
            SkyeType::Type(inner) => inner.contains_unknown(),
            SkyeType::Group(a, b) => {
                a.contains_unknown() || b.contains_unknown()
            }
            SkyeType::Function(params, return_type, _) => {
                if return_type.contains_unknown() {
                    return true;
                }

                for param in params {
                    if param.type_.contains_unknown() {
                        return true;
                    }
                }

                false
            }
            SkyeType::Struct(_, fields, _) |
            SkyeType::Union(_, fields) => {
                if let Some(fields) = fields {
                    for (_, field) in fields {
                        if field.type_.contains_unknown() {
                            return true;
                        }
                    }
                }

                false
            }
            SkyeType::Enum(_, variants, _) => {
                if let Some(variants) = variants {
                    for (_, variant) in variants {
                        if variant.contains_unknown() {
                            return true;
                        }
                    }
                }

                false
            }
            _ => false
        }
    }

    fn is_recursive_inner(&self, mut main_name: Option<Rc<str>>) -> bool {
        match self {
            SkyeType::Type(inner) => inner.is_recursive_inner(main_name),
            SkyeType::Struct(name, fields, _) |
            SkyeType::Union(name, fields) => {
                if let Some(main_name) = &main_name {
                    if name.as_ref() == main_name.as_ref() {
                        return true;
                    }
                } else {
                    main_name = Some(Rc::clone(&name));
                }

                if let Some(fields) = fields {
                    for (_, field) in fields {
                        if field.type_.is_recursive_inner(main_name.clone()) {
                            return true;
                        }
                    }
                }
                
                false
            }
            SkyeType::Enum(name, variants, _) => {
                if let Some(main_name) = &main_name {
                    if name.as_ref() == main_name.as_ref() {
                        return true;
                    }
                } else {
                    main_name = Some(Rc::clone(&name));
                }

                if let Some(variants) = variants {
                    for (_, variant) in variants {
                        if variant.is_recursive_inner(main_name.clone()) {
                            return true;
                        }
                    }
                }
                
                false
            }
            _ => false
        }
    }

    pub fn is_recursive(&self) -> bool {
        self.is_recursive_inner(None)
    }

    pub fn get_unknown() -> SkyeType {
        SkyeType::Unknown(Rc::from(""))
    }

    pub fn get_unknown_type() -> SkyeType {
        SkyeType::Type(Box::new(SkyeType::Unknown(Rc::from(""))))
    }
}