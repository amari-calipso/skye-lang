use std::{cell::OnceCell, collections::{HashMap, HashSet}, rc::Rc};

use lazy_static::lazy_static;
use topo_sort::{SortResults, TopoSort};

use crate::{ast::{Bits, Expression, StringKind}, ir::{AssignOp, BinaryOp, IrStatement, IrStatementData, IrValue, IrValueData, TypeKind}, skye_type::{EqualsLevel, SkyeType}, utils::{fix_raw_string, get_real_string_length}};

const OUTPUT_INDENT_SPACES: usize = 4;

lazy_static! {
    pub static ref C_KEYWORDS: HashSet<&'static str> = HashSet::from([
        "alignas", "alignof", "auto", "constexpr", "goto", "inline", 
        "nullptr", "register", "restrict", "signed", "sizeof",
        "static", "static_assert", "thread_local", "typedef", "typeof",
        "typeof_unqual", "unsigned", "volatile", "_Alignas", "_Alignof",
        "_Atomic", "_BitInt", "_Bool", "_Complex", "_Decimal128",
        "_Decimal32", "_Decimal64", "_Generic", "_Imaginary", "_Noreturn",
        "_Static_assert", "_Thread_local"
    ]);
}

const VOID_MAIN: &str = concat!(
    "int main() {\n",
    "    _SKYE_INIT();\n",
    "    _SKYE_MAIN();\n",
    "    return 0;\n",
    "}\n\n"
);
const VOID_MAIN_PLUS_STD_ARGS: &str = concat!(
    "int main(int argc, char** argv) {\n",
    "    _SKYE_INIT();\n",
    "    _SKYE_MAIN(argc, argv);\n",
    "    return 0;\n",
    "}\n\n"
);
const VOID_MAIN_PLUS_ARGS: &str = concat!(
    "int main(int argc, char** argv) {\n",
    "    _SKYE_INIT();\n",
    "    core_DOT_Array_GENOF_core_DOT_Slice_GENOF_char_GENEND__GENAND_core_DOT_mem_DOT_HeapAllocator_GENEND_ args = _SKYE_CONVERT_ARGS(argc, argv);\n",
    "    _SKYE_MAIN(args);\n",
    "    core_DOT_Array_DOT_free_GENOF_core_DOT_Slice_GENOF_char_GENEND__GENAND_core_DOT_mem_DOT_HeapAllocator_GENEND_(&args);\n",
    "    return 0;\n",
    "}\n\n"
);
const RESULT_VOID_MAIN: &str = concat!(
    "int main() {\n",
    "    _SKYE_INIT();\n",
    "    core_DOT_Result_GENOF_void_GENAND_void_GENEND_ result = _SKYE_MAIN();\n",
    "    return result.kind != core_DOT_Result_DOT_Kind_DOT_Ok;\n",
    "}\n\n"
);
const RESULT_VOID_MAIN_PLUS_STD_ARGS: &str = concat!(
    "int main(int argc, char** argv) {\n",
    "    _SKYE_INIT();\n",
    "    core_DOT_Result_GENOF_void_GENAND_void_GENEND_ result = _SKYE_MAIN(argc, argv);\n",
    "    return result.kind != core_DOT_Result_DOT_Kind_DOT_Ok;\n",
    "}\n\n"
);
const RESULT_VOID_MAIN_PLUS_ARGS: &str = concat!(
    "int main(int argc, char** argv) {\n",
    "    _SKYE_INIT();\n",
    "    core_DOT_Array_GENOF_core_DOT_Slice_GENOF_char_GENEND__GENAND_core_DOT_mem_DOT_HeapAllocator_GENEND_ args = _SKYE_CONVERT_ARGS(argc, argv);\n",
    "    core_DOT_Result_GENOF_void_GENAND_void_GENEND_ result = _SKYE_MAIN(args);\n",
    "    core_DOT_Array_DOT_free_GENOF_core_DOT_Slice_GENOF_char_GENEND__GENAND_core_DOT_mem_DOT_HeapAllocator_GENEND_(&args);\n",
    "    return result.kind != core_DOT_Result_DOT_Kind_DOT_Ok;\n",
    "}\n\n"
);
const RESULT_I32_MAIN: &str = concat!(
    "int main() {\n",
    "    _SKYE_INIT();\n",
    "    core_DOT_Result_GENOF_void_GENAND_i32_GENEND_ result = _SKYE_MAIN();\n",
    "    if (result.kind == core_DOT_Result_DOT_Kind_DOT_Ok) return 0;\n",
    "    return result.Error;\n",
    "}\n\n"
);
const RESULT_I32_MAIN_PLUS_STD_ARGS: &str = concat!(
    "int main(int argc, char** argv) {\n",
    "    _SKYE_INIT();\n",
    "    core_DOT_Result_GENOF_void_GENAND_i32_GENEND_ result = _SKYE_MAIN(argc, argv);\n",
    "    if (result.kind == core_DOT_Result_DOT_Kind_DOT_Ok) return 0;\n",
    "    return result.Error;\n",
    "}\n\n"
);
const RESULT_I32_MAIN_PLUS_ARGS: &str = concat!(
    "int main(int argc, char** argv) {\n",
    "    _SKYE_INIT();\n",
    "    core_DOT_Array_GENOF_core_DOT_Slice_GENOF_char_GENEND__GENAND_core_DOT_mem_DOT_HeapAllocator_GENEND_ args = _SKYE_CONVERT_ARGS(argc, argv);\n",
    "    core_DOT_Result_GENOF_void_GENAND_i32_GENEND_ result = _SKYE_MAIN(args);\n",
    "    core_DOT_Array_DOT_free_GENOF_core_DOT_Slice_GENOF_char_GENEND__GENAND_core_DOT_mem_DOT_HeapAllocator_GENEND_(&args);\n",
    "    if (result.kind == core_DOT_Result_DOT_Kind_DOT_Ok) return 0;\n",
    "    return result.Error;\n",
    "}\n\n"
);
const I32_MAIN: &str = concat!(
    "int main() {\n",
    "    _SKYE_INIT();\n",
    "    return _SKYE_MAIN();\n",
    "}\n\n"
);
const I32_MAIN_PLUS_STD_ARGS: &str = concat!(
    "int main(int argc, char** argv) {\n",
    "    _SKYE_INIT();\n",
    "    return _SKYE_MAIN(argc, argv);\n",
    "}\n\n"
);
const I32_MAIN_PLUS_ARGS: &str = concat!(
    "int main(int argc, char** argv) {\n",
    "    _SKYE_INIT();\n",
    "    core_DOT_Array_GENOF_core_DOT_Slice_GENOF_char_GENEND__GENAND_core_DOT_mem_DOT_HeapAllocator_GENEND_ args = _SKYE_CONVERT_ARGS(argc, argv);\n",
    "    i32 result = _SKYE_MAIN(args);\n",
    "    core_DOT_Array_DOT_free_GENOF_core_DOT_Slice_GENOF_char_GENEND__GENAND_core_DOT_mem_DOT_HeapAllocator_GENEND_(&args);\n",
    "    return result;\n",
    "}\n\n"
);


#[derive(Clone, Debug)]
pub struct CodeOutput {
    pub code: String,
    indent: usize
}

impl CodeOutput {
    pub fn new() -> Self {
        CodeOutput { code: String::new(), indent: 0 }
    }

    pub fn push_indent(&mut self) {
        for _ in 0 .. self.indent * OUTPUT_INDENT_SPACES {
            self.code.push(' ');
        }
    }

    pub fn push(&mut self, string: &str) {
        self.code.push_str(string);
    }

    pub fn inc_indent(&mut self) {
        self.indent += 1;
    }

    pub fn dec_indent(&mut self) {
        self.indent -= 1;
    }

    #[allow(dead_code)]
    pub fn set_indent(&mut self, indent: usize) {
        self.indent = indent;
    }
}

fn stringify_type(type_: &SkyeType) -> String {
    match type_ {
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

        SkyeType::Char => String::from("char"),

        SkyeType::Void       => String::from("void"),
        SkyeType::Unknown(_) => String::from("void*"),

        SkyeType::Group(..) |
        SkyeType::Template(..) |
        SkyeType::Macro(..) => String::new(),

        SkyeType::Type(inner) => stringify_type(&inner),
        SkyeType::Function(..) => type_.mangle(),

        SkyeType::Pointer(inner, ..) => {
            format!("{}*", stringify_type(&inner))
        }

        SkyeType::Array(inner, size) => {
            format!("SKYE_ARRAY_{}_{}", inner.mangle(), *size)
        }

        SkyeType::Struct(name, ..) |
        SkyeType::Namespace(name) |
        SkyeType::Enum(name, ..) |
        SkyeType::Union(name, _) => name.to_string(),
    }
}

fn get_type_dependencies_inner(type_: &SkyeType, toplevel: bool, declarations: bool) -> HashSet<Rc<str>> {
    match type_ {
        SkyeType::Type(inner) => get_type_dependencies_inner(&inner, false, declarations),
        SkyeType::Struct(name, fields, _) |
        SkyeType::Union(name, fields) => {
            let mut dependencies = HashSet::new();

            if !toplevel {
                dependencies.insert(Rc::clone(name));   
            }
            
            // if we're just declaring the struct/union, fields aren't defined yet, and hence we don't depend on them
            if !declarations {
                if let Some(fields) = fields {
                    for (_, field) in fields {
                        dependencies.extend(get_type_dependencies_inner(&field.type_, false, declarations));
                    }
                }
            }
            
            dependencies
        }
        SkyeType::Enum(name, variants, _) => {
            let mut dependencies = HashSet::new();

            if !toplevel {
                dependencies.insert(Rc::clone(name));   
            }

            // if we're just declaring the tagged union, fields aren't defined yet, and hence we don't depend on them
            if !declarations {
                if let Some(variants) = variants {
                    for (_, variant) in variants {
                        dependencies.extend(get_type_dependencies_inner(&variant, false, declarations));
                    }
                }
            }
            
            dependencies
        }
        SkyeType::Function(params, return_type, _) => {
            // if we aren't sorting declarations, functions don't matter in the order
            if !declarations {
                return HashSet::new();
            }

            let mut dependencies = get_type_dependencies_inner(&return_type, false, declarations);

            if !toplevel {
                dependencies.insert(type_.mangle().into());
            }

            for param in params {
                dependencies.extend(get_type_dependencies_inner(&param.type_, false, declarations));
            }

            dependencies
        }
        SkyeType::Pointer(inner, ..) => {
            // i didn't know this, but in declarations types hidden behind pointers still count as undeclared?
            if !declarations {
                return HashSet::new();
            }

            get_type_dependencies_inner(&inner, false, declarations)
        }
        SkyeType::Group(..) | 
        SkyeType::Namespace(_) | 
        SkyeType::Macro(..) |
        SkyeType::Template(..) => {
            println!("{:?}", type_);
            unreachable!()
        }
        _ => HashSet::new()
    }
}

fn get_type_dependencies_definitions(type_: &SkyeType) -> HashSet<Rc<str>> {
    get_type_dependencies_inner(type_, true, false)
}

fn get_type_dependencies_definitions_from_within(type_: &SkyeType) -> HashSet<Rc<str>> {
    get_type_dependencies_inner(type_, false, false)
}

fn get_type_dependencies_declarations(type_: &SkyeType) -> HashSet<Rc<str>> {
    get_type_dependencies_inner(type_, true, true)
}

fn get_type_dependencies_declarations_from_within(type_: &SkyeType) -> HashSet<Rc<str>> {
    get_type_dependencies_inner(type_, false, true)
}

fn prepare_name(name: Rc<str>) -> Rc<str> {
    if C_KEYWORDS.contains(name.as_ref()) {
        format!("__reserved_{}", name).into()
    } else {
        name
    }
}

struct TypeOutput {
    pub output: CodeOutput,
    pub dependencies: OnceCell<HashSet<Rc<str>>>
}

impl TypeOutput {
    pub fn new(output: CodeOutput, dependencies: HashSet<Rc<str>>) -> Self {
        TypeOutput { output, dependencies: OnceCell::from(dependencies) }
    }

    pub fn independent(output: CodeOutput) -> Self {
        TypeOutput { output, dependencies: OnceCell::from(HashSet::new()) }
    }
}

pub struct CodeGen {
    strings:      HashMap<Rc<str>, usize>,
    arrays:       HashSet<Rc<str>>,
    fnptrs:       HashSet<Rc<str>>,

    strings_code: CodeOutput,
    includes:     CodeOutput,
    declarations: HashMap<Rc<str>, TypeOutput>,
    typedefs:     HashMap<Rc<str>, TypeOutput>,
    fndefs:       Vec<CodeOutput>,

    in_function: bool,
}

impl CodeGen {
    pub fn new() -> Self {
        CodeGen {
            strings: HashMap::new(),
            arrays: HashSet::new(),
            fnptrs: HashSet::new(),
            strings_code: CodeOutput::new(),
            includes: CodeOutput::new(),
            declarations: HashMap::new(),
            typedefs: HashMap::new(),
            fndefs: Vec::new(),
            in_function: false,
        }
    }

    fn prepare_array_struct(&mut self, array_specifier: Rc<str>, type_name: &Rc<str>, type_: &SkyeType, size: usize) {
        if !self.arrays.contains(&array_specifier) {
            let mut buf = String::from("typedef struct SKYE_ARRAY_STRUCT_");
            buf.push_str(&array_specifier);

            self.arrays.insert(array_specifier);

            let mut decl_buf = CodeOutput::new();
            decl_buf.push(&buf);
            decl_buf.push(";\n");

            self.declarations.insert(Rc::clone(&type_name), TypeOutput::independent(decl_buf));

            let mut def_buf = CodeOutput::new();
            def_buf.push(&buf);
            def_buf.push(" {\n");
            def_buf.inc_indent();

            def_buf.push_indent();
            def_buf.push(&type_.stringify());
            def_buf.push(" SKYE_ARRAY[");
            def_buf.push(&size.to_string());
            def_buf.push("];\n");
            def_buf.dec_indent();

            def_buf.push("} ");
            def_buf.push(&type_name);
            def_buf.push(";\n\n");

            self.typedefs.insert(Rc::clone(&type_name), TypeOutput::independent(def_buf));
        }
    }

    fn generate_fn_signature(&mut self, inner_type: &SkyeType, return_stringified: &String, params_string: &String) {
        let mangled: Rc<str> = inner_type.mangle().into();

        if !self.fnptrs.contains(&mangled) {
            let dependencies = get_type_dependencies_declarations(inner_type);
            let mut buf = CodeOutput::new();

            buf.push("typedef ");
            buf.push(return_stringified);
            buf.push(" (*");
            buf.push(&mangled);
            buf.push(")(");
            buf.push(&params_string);
            buf.push(");\n");

            self.declarations.insert(Rc::clone(&mangled), TypeOutput::new(buf, dependencies));
            self.fnptrs.insert(mangled);
        }
    }

    async fn generate_value(&mut self, value: IrValue, ctx: &mut reblessive::Stk) -> Rc<str> {
        if let SkyeType::Function(params, return_type, _) = &value.type_ {
            let mut params_string = String::new();

            for (i, param) in params.iter().enumerate() {
                params_string.push_str(&stringify_type(&param.type_));

                if i != params.len() - 1 {
                    params_string.push_str(", ");
                }
            }

            let return_stringified = stringify_type(&return_type);
            self.generate_fn_signature(&value.type_, &return_stringified, &params_string);
        }

        match value.data {
            IrValueData::Empty => unreachable!(),
            IrValueData::Variable { name } => prepare_name(name),
            IrValueData::Grouping(value) => {
                let generated = ctx.run(|ctx| self.generate_value(*value, ctx)).await;
                format!("({})", generated).into()
            }
            IrValueData::Increment { value } => {
                let generated = ctx.run(|ctx| self.generate_value(*value, ctx)).await;
                format!("++{}", generated).into()
            }
            IrValueData::Decrement { value } => {
                let generated = ctx.run(|ctx| self.generate_value(*value, ctx)).await;
                format!("--{}", generated).into()
            }
            IrValueData::Negative { value } => {
                let generated = ctx.run(|ctx| self.generate_value(*value, ctx)).await;
                format!("-{}", generated).into()
            }
            IrValueData::Invert { value } => {
                let generated = ctx.run(|ctx| self.generate_value(*value, ctx)).await;
                format!("~{}", generated).into()
            }
            IrValueData::Negate { value } => {
                let generated = ctx.run(|ctx| self.generate_value(*value, ctx)).await;
                format!("!{}", generated).into()
            }
            IrValueData::Reference { value } => {
                let generated = ctx.run(|ctx| self.generate_value(*value, ctx)).await;
                format!("&{}", generated).into()
            }
            IrValueData::Dereference { value } => {
                let generated = ctx.run(|ctx| self.generate_value(*value, ctx)).await;
                format!("*{}", generated).into()
            }
            IrValueData::Get { from, name } => {
                let generated = ctx.run(|ctx| self.generate_value(*from, ctx)).await;
                format!("{}.{}", generated, name).into()
            }
            IrValueData::DereferenceGet { from, name } => {
                let generated = ctx.run(|ctx| self.generate_value(*from, ctx)).await;
                format!("{}->{}", generated, name).into()
            }
            IrValueData::Ternary { condition, then_branch, else_branch } => {
                let condition   = ctx.run(|ctx| self.generate_value(*condition, ctx)).await;
                let then_branch = ctx.run(|ctx| self.generate_value(*then_branch, ctx)).await;
                let else_branch = ctx.run(|ctx| self.generate_value(*else_branch, ctx)).await;
                format!("{} ? {} : {}", condition, then_branch, else_branch).into()
            }
            IrValueData::TypeRef { kind, name } => {
                let kind_str = {
                    match kind {
                        TypeKind::Struct => "struct",
                        TypeKind::Enum   => "enum",
                        TypeKind::Union  => "union",
                    }
                };

                format!("{} {}", kind_str, name).into()
            }
            IrValueData::Cast { to, from } => {
                if matches!(from.type_, SkyeType::Array(..)) && matches!(to, SkyeType::Pointer(..)) {
                    let generated = ctx.run(|ctx| self.generate_value(*from, ctx)).await;
                    format!("({})(({}).SKYE_ARRAY)", stringify_type(&to), generated).into()
                } else {
                    let generated = ctx.run(|ctx| self.generate_value(*from, ctx)).await;
                    format!("({})({})", stringify_type(&to), generated).into()
                }
            }
            IrValueData::Subscript { subscripted, index } => {
                let generated_index = ctx.run(|ctx| self.generate_value(*index, ctx)).await;

                if matches!(subscripted.type_, SkyeType::Array(..)) {
                    let generated_subscripted = ctx.run(|ctx| self.generate_value(*subscripted, ctx)).await;
                    format!("({}).SKYE_ARRAY[{}]", generated_subscripted, generated_index).into()
                } else {
                    let generated_subscripted = ctx.run(|ctx| self.generate_value(*subscripted, ctx)).await;
                    format!("{}[{}]", generated_subscripted, generated_index).into()
                }
            }
            IrValueData::Slice { items } => {
                let mut buf = String::from("(");
                buf.push_str(&stringify_type(&value.type_));
                buf.push_str(") { .ptr = (");
                buf.push_str(&stringify_type(&items[0].type_));
                buf.push_str("[]) {");

                let size = items.len();
                for (i, item) in items.into_iter().enumerate() {
                    let generated = ctx.run(|ctx| self.generate_value(item, ctx)).await;
                    buf.push_str(&generated);

                    if i != size - 1 {
                        buf.push_str(", ");
                    }
                }

                buf.push_str("}, .length = ");
                buf.push_str(&size.to_string());
                buf.push_str(" }");
                buf.into()
            }
            IrValueData::Array { items } => {
                let size = items.len();
                let type_ = items[0].type_.clone();
                let array_specifier: Rc<str> = format!("{}_{}", type_.mangle(), size).into();
                let type_name: Rc<str> = format!("SKYE_ARRAY_{}", array_specifier).into();
                self.prepare_array_struct(array_specifier, &type_name, &type_, size);

                let mut buf = String::new();
                buf.push('(');
                buf.push_str(&type_name);
                buf.push_str(") { .SKYE_ARRAY = { ");

                for (i, item) in items.into_iter().enumerate() {
                    let generated = ctx.run(|ctx| self.generate_value(item, ctx)).await;
                    buf.push_str(&generated);

                    if i != size - 1 {
                        buf.push_str(", ");
                    }
                }

                buf.push_str(" } }");
                buf.into()
            }
            IrValueData::CompoundLiteral { items } => {
                let mut buf = String::from("(");
                buf.push_str(&stringify_type(&value.type_));
                buf.push_str(") { ");

                let size = items.len();
                for (i, (name, item)) in items.into_iter().enumerate() {
                    let generated = ctx.run(|ctx| self.generate_value(item, ctx)).await;

                    buf.push('.');
                    buf.push_str(&name);
                    buf.push_str(" = ");
                    buf.push_str(&generated);

                    if i != size - 1 {
                        buf.push_str(", ");
                    }
                }

                buf.push_str(" }");
                buf.into()
            }
            IrValueData::Call { callee, args } => {
                let is_macro = matches!(callee.type_, SkyeType::Macro(_, _, _));
                let generated_callee = ctx.run(|ctx| self.generate_value(*callee, ctx)).await;

                let mut buf = generated_callee.to_string();
                buf.push('(');

                let size = args.len();
                for (i, arg) in args.into_iter().enumerate() {
                    if is_macro {
                        let stringified: Rc<str>;

                        if arg.is_empty() {
                            stringified = stringify_type(&arg.type_).into();
                        } else {
                            stringified = ctx.run(|ctx| self.generate_value(arg, ctx)).await;
                        }

                        // https://github.com/amari-calipso/skye-lang/issues/52
                        if stringified.contains(",") {
                            buf.push('(');
                            buf.push_str(&stringified);
                            buf.push(')');
                        } else {
                            buf.push_str(&stringified);
                        }
                    } else {
                        let generated = ctx.run(|ctx| self.generate_value(arg, ctx)).await;
                        buf.push_str(&generated);
                    }

                    if i != size - 1 {
                        buf.push_str(", ");
                    }
                }

                buf.push(')');
                buf.into()
            }
            IrValueData::Assign { target, op, value } => {
                let generated_target = ctx.run(|ctx| self.generate_value(*target, ctx)).await;
                let generated_value  = ctx.run(|ctx| self.generate_value(*value, ctx)).await;

                let operator = {
                    match op {
                        AssignOp::None => "=",
                        AssignOp::Add => "+=",
                        AssignOp::Subtract => "-=",
                        AssignOp::Divide => "/=",
                        AssignOp::Multiply => "*=",
                        AssignOp::Modulo => "%=",
                        AssignOp::ShiftLeft => ">>=",
                        AssignOp::ShiftRight => "<<=",
                        AssignOp::BitwiseXor => "^=",
                        AssignOp::BitwiseOr => "|=",
                        AssignOp::BitwiseAnd => "&="
                    }
                };

                format!("{} {} {}", generated_target, operator, generated_value).into()
            }
            IrValueData::Binary { left, op, right } => {
                let generated_left  = ctx.run(|ctx| self.generate_value(*left, ctx)).await;
                let generated_right = ctx.run(|ctx| self.generate_value(*right, ctx)).await;

                let operator = {
                    match op {
                        BinaryOp::Add => "+",
                        BinaryOp::Subtract => "-",
                        BinaryOp::Divide => "/",
                        BinaryOp::Multiply => "*",
                        BinaryOp::Modulo => "%",
                        BinaryOp::ShiftLeft => ">>",
                        BinaryOp::ShiftRight => "<<",
                        BinaryOp::BitwiseXor => "^",
                        BinaryOp::BitwiseOr => "|",
                        BinaryOp::BitwiseAnd => "&",
                        BinaryOp::Greater => ">",
                        BinaryOp::GreaterEqual => ">=",
                        BinaryOp::Less => "<",
                        BinaryOp::LessEqual => "<=",
                        BinaryOp::Equal => "==",
                        BinaryOp::NotEqual => "!="
                    }
                };

                format!("{} {} {}", generated_left, operator, generated_right).into()
            }
            IrValueData::Literal { value } => {
                match value {
                    Expression::SignedIntLiteral { value, bits, .. } => {
                        match bits {
                            Bits::B8  => format!("INT8_C({})",  value).into(),
                            Bits::B16 => format!("INT16_C({})", value).into(),
                            Bits::B32 => format!("INT32_C({})", value).into(),
                            Bits::B64 => format!("INT64_C({})", value).into(),
                            Bits::Any => value.to_string().into(),
                            Bits::Bsz => unreachable!()
                        }
                    }
                    Expression::UnsignedIntLiteral { value, bits, .. } => {
                        match bits {
                            Bits::B8  => format!("UINT8_C({})",  value).into(),
                            Bits::B16 => format!("UINT16_C({})", value).into(),
                            Bits::B32 => format!("UINT32_C({})", value).into(),
                            Bits::B64 => format!("UINT64_C({})", value).into(),
                            Bits::Bsz => format!("SIZE_T_C({})", value).into(),
                            Bits::Any => unreachable!()
                        }
                    }
                    Expression::FloatLiteral { value, bits, .. } => {
                        match bits {
                            Bits::B32 => format!("(float){:?}",  value).into(),
                            Bits::B64 => format!("(double){:?}", value).into(),
                            Bits::Any => format!("{:?}",         value).into(),
                            _ => unreachable!()
                        }
                    }
                    Expression::StringLiteral { value, kind, .. } => {
                        match kind {
                            StringKind::Char => format!("'{}'", value).into(),
                            StringKind::Raw => {
                                if let Some(string_const) = self.strings.get(&value) {
                                    format!("SKYE_STRING_{}", string_const).into()
                                } else {
                                    let str_index = self.strings.len();
                                    self.strings_code.push(format!(
                                        "const char SKYE_STRING_{}[{}] = \"{}\";\n",
                                        str_index, get_real_string_length(&value), fix_raw_string(&value)
                                    ).as_ref());

                                    self.strings.insert(value, str_index);
                                    format!("SKYE_STRING_{}", str_index).into()
                                }
                            }
                            StringKind::Slice => {
                                if let Some(string_const) = self.strings.get(&value) {
                                    format!(
                                        "(core_DOT_Slice_GENOF_char_GENEND_) {{ .ptr = SKYE_STRING_{}, .length = sizeof(SKYE_STRING_{}) }}",
                                        string_const, string_const
                                    ).into()
                                } else {
                                    let str_index = self.strings.len();
                                    let string_len = get_real_string_length(&value);
                                    self.strings_code.push(format!(
                                        "const char SKYE_STRING_{}[{}] = \"{}\";\n",
                                        str_index, string_len, value
                                    ).as_ref());

                                    self.strings.insert(value, str_index);

                                    format!(
                                        "(core_DOT_Slice_GENOF_char_GENEND_) {{ .ptr = SKYE_STRING_{}, .length = {} }}",
                                        str_index, string_len
                                    ).into()
                                }
                            }
                        }
                    }
                    _ => unreachable!()
                }
            }
        }
    }

    fn begin_scope(&mut self) {
        self.fndefs.last_mut().unwrap().push_indent();
        self.fndefs.last_mut().unwrap().push("{\n");
        self.fndefs.last_mut().unwrap().inc_indent();
    }

    fn end_scope(&mut self) {
        self.fndefs.last_mut().unwrap().dec_indent();
        self.fndefs.last_mut().unwrap().push_indent();
        self.fndefs.last_mut().unwrap().push("}\n");
    }

    async fn generate_statement(&mut self, statement: IrStatement, ctx: &mut reblessive::Stk) {
        match statement.data {
            IrStatementData::Break => {
                self.fndefs.last_mut().unwrap().push_indent();
                self.fndefs.last_mut().unwrap().push("break;\n");
            }
            IrStatementData::Loop { body } => {
                self.fndefs.last_mut().unwrap().push_indent();
                self.fndefs.last_mut().unwrap().push("while (1)\n");

                let not_block = !matches!(body.data, IrStatementData::Scope { .. });

                if not_block {
                    self.begin_scope();
                }

                ctx.run(|ctx| self.generate_statement(*body, ctx)).await;

                if not_block {
                    self.end_scope();
                }
            }
            IrStatementData::Label { name } => {
                self.fndefs.last_mut().unwrap().push_indent();
                self.fndefs.last_mut().unwrap().push(&prepare_name(name));
                self.fndefs.last_mut().unwrap().push(":;\n");
            }
            IrStatementData::Goto { label } => {
                self.fndefs.last_mut().unwrap().push_indent();
                self.fndefs.last_mut().unwrap().push("goto ");
                self.fndefs.last_mut().unwrap().push(&prepare_name(label));
                self.fndefs.last_mut().unwrap().push(";\n");
            }
            IrStatementData::Undefine { name } => {
                self.fndefs.last_mut().unwrap().push_indent();
                self.fndefs.last_mut().unwrap().push("#undef ");
                self.fndefs.last_mut().unwrap().push(&prepare_name(name));
                self.fndefs.last_mut().unwrap().push("\n");
            }
            IrStatementData::Expression { value } => {
                let generated = ctx.run(|ctx| self.generate_value(value, ctx)).await;
                self.fndefs.last_mut().unwrap().push_indent();
                self.fndefs.last_mut().unwrap().push(&generated);
                self.fndefs.last_mut().unwrap().push(";\n");
            }
            IrStatementData::Return { value } => {
                self.fndefs.last_mut().unwrap().push_indent();
                self.fndefs.last_mut().unwrap().push("return");

                if let Some(value) = value {
                    let generated = ctx.run(|ctx| self.generate_value(value, ctx)).await;
                    self.fndefs.last_mut().unwrap().push(" ");
                    self.fndefs.last_mut().unwrap().push(&generated);
                }

                self.fndefs.last_mut().unwrap().push(";\n");
            }
            IrStatementData::Scope { statements } => {
                let len = statements.borrow().len();

                if len == 0 {
                    self.fndefs.last_mut().unwrap().push_indent();
                    self.fndefs.last_mut().unwrap().push("{}\n");
                } else if len == 1 && matches!(statements.borrow()[0].data, IrStatementData::Scope { .. }) {
                    let statement = statements.borrow_mut().pop().unwrap();
                    ctx.run(|ctx| self.generate_statement(statement, ctx)).await;
                } else {
                    self.begin_scope();

                    for statement in statements.borrow_mut().drain(0..len) {
                        ctx.run(|ctx| self.generate_statement(statement, ctx)).await;
                    }

                    self.end_scope();
                }
            }
            IrStatementData::If { condition, then_branch, else_branch } => {
                let condition = ctx.run(|ctx| self.generate_value(condition, ctx)).await;
                
                self.fndefs.last_mut().unwrap().push_indent();
                self.fndefs.last_mut().unwrap().push("if (");
                self.fndefs.last_mut().unwrap().push(&condition);
                self.fndefs.last_mut().unwrap().push(")\n");

                let not_block = !matches!(then_branch.data, IrStatementData::Scope { .. });

                if not_block {
                    self.begin_scope();
                }

                ctx.run(|ctx| self.generate_statement(*then_branch, ctx)).await;

                if not_block {
                    self.end_scope();
                }
                
                if let Some(else_branch) = else_branch {
                    self.fndefs.last_mut().unwrap().push_indent();
                    self.fndefs.last_mut().unwrap().push("else\n");

                    let not_block = !matches!(else_branch.data, IrStatementData::Scope { .. });

                    if not_block {
                        self.begin_scope();
                    }

                    ctx.run(|ctx| self.generate_statement(*else_branch, ctx)).await;

                    if not_block {
                        self.end_scope();
                    }
                }
            }
            IrStatementData::Include { path, is_ang } => {
                self.includes.push_indent();
                self.includes.push("#include ");

                if is_ang {
                    self.includes.push("<")
                } else {
                    self.includes.push("\"");
                }

                self.includes.push(&path);

                if is_ang {
                    self.includes.push(">")
                } else {
                    self.includes.push("\"");
                }

                self.includes.push("\n");
            }
            IrStatementData::VarDecl { name, type_, initializer } => {
                let prepared_name = prepare_name(name);
                let mut buf = String::new();
                
                buf.push_str(&stringify_type(&type_));
                buf.push(' ');
                buf.push_str(&prepared_name);

                if let Some(initializer) = initializer {
                    let generated = ctx.run(|ctx| self.generate_value(initializer, ctx)).await;
                    buf.push_str(" = ");
                    buf.push_str(&generated);
                }

                buf.push_str(";\n");

                if self.in_function {
                    self.fndefs.last_mut().unwrap().push_indent();
                    self.fndefs.last_mut().unwrap().push(&buf);
                } else {
                    let mut decl_buf = CodeOutput::new();
                    decl_buf.push_indent();
                    decl_buf.push(&buf);
                    
                    let dependencies = get_type_dependencies_declarations_from_within(&type_);
                    self.declarations.insert(prepared_name, TypeOutput::new(decl_buf, dependencies));
                }
            }
            IrStatementData::Define { name, value, typedef } => {
                let prepared_name = prepare_name(name);
                let mut buf = String::new();
                
                let type_ = value.type_.clone();
                let generated = ctx.run(|ctx| self.generate_value(value, ctx)).await;

                if typedef {
                    buf.push_str("typedef ");
                    buf.push_str(&generated);
                    buf.push(' ');
                    buf.push_str(&prepared_name);
                    buf.push_str(";\n");
                } else {
                    buf.push_str("#define ");
                    buf.push_str(&prepared_name);
                    buf.push(' ');
                    buf.push_str(&generated);
                    buf.push('\n');
                }

                if self.in_function {
                    self.fndefs.last_mut().unwrap().push_indent();
                    self.fndefs.last_mut().unwrap().push(&buf);
                } else {
                    let mut decl_buf = CodeOutput::new();
                    decl_buf.push_indent();
                    decl_buf.push(&buf);

                    let dependencies = get_type_dependencies_declarations_from_within(&type_);
                    self.declarations.insert(prepared_name, TypeOutput::new(decl_buf, dependencies));
                }
            }
            IrStatementData::Struct { type_ } => {
                let dependencies = get_type_dependencies_definitions(&type_);
                if let SkyeType::Struct(name, fields, _) = type_ {
                    let prepared_name = prepare_name(name);

                    let mut buf = CodeOutput::new();
                    buf.push_indent();
                    buf.push("typedef struct SKYE_STRUCT_");
                    buf.push(&prepared_name);

                    let mut decl_buf = buf.clone();
                    decl_buf.push(" ");
                    decl_buf.push(&prepared_name);
                    decl_buf.push(";\n");
                    self.declarations.insert(Rc::clone(&prepared_name), TypeOutput::independent(decl_buf));

                    if let Some(fields) = fields {
                        buf.push(" {\n");
                        buf.inc_indent();

                        for (name, field) in fields {
                            buf.push_indent();
                            buf.push(&stringify_type(&field.type_));
                            buf.push(" ");
                            buf.push(&prepare_name(name));

                            if let Some(bits) = field.bits {
                                buf.push(": ");
                                buf.push(&bits.to_string());
                            }

                            buf.push(";\n");
                        }

                        buf.dec_indent();
                        buf.push_indent();
                        buf.push("} ");
                        buf.push(&prepared_name);
                        buf.push(";\n");

                        self.typedefs.insert(prepared_name, TypeOutput::new(buf, dependencies));
                    }
                } else {
                    unreachable!()
                }
            }
            IrStatementData::Union { type_ } => {
                let dependencies = get_type_dependencies_definitions(&type_);
                if let SkyeType::Union(name, fields) = type_ {
                    let prepared_name = prepare_name(name);

                    let mut buf = CodeOutput::new();
                    buf.push_indent();
                    buf.push("typedef union SKYE_UNION_");
                    buf.push(&prepared_name);

                    let mut decl_buf = buf.clone();
                    decl_buf.push(" ");
                    decl_buf.push(&prepared_name);
                    decl_buf.push(";\n");
                    self.declarations.insert(Rc::clone(&prepared_name), TypeOutput::independent(decl_buf));

                    if let Some(fields) = fields {
                        buf.push(" {\n");
                        buf.inc_indent();

                        for (name, field) in fields {
                            buf.push_indent();
                            buf.push(&stringify_type(&field.type_));
                            buf.push(" ");
                            buf.push(&prepare_name(name));

                            if let Some(bits) = field.bits {
                                buf.push(": ");
                                buf.push(&bits.to_string());
                            }

                            buf.push(";\n");
                        }

                        buf.dec_indent();
                        buf.push_indent();
                        buf.push("} ");

                        buf.push(&prepared_name);
                        buf.push(";\n");
                        self.typedefs.insert(prepared_name, TypeOutput::new(buf, dependencies));
                    } 
                } else {
                    unreachable!()
                }
            }
            IrStatementData::TaggedUnion { name, kind_name, kind_type, fields } => {
                let prepared_name = prepare_name(name);

                let mut buf = CodeOutput::new();
                buf.push_indent();
                buf.push("typedef struct SKYE_STRUCT_");
                buf.push(&prepared_name);

                let mut decl_buf = buf.clone();
                decl_buf.push(" ");
                decl_buf.push(&prepared_name);
                decl_buf.push(";\n");
                self.declarations.insert(Rc::clone(&prepared_name), TypeOutput::independent(decl_buf));

                buf.push(" {\n");
                buf.inc_indent();

                buf.push_indent();
                buf.push("union {\n");
                buf.inc_indent();

                let mut dependencies = HashSet::new();

                for (name, type_) in fields {
                    dependencies.extend(get_type_dependencies_definitions_from_within(&type_));

                    buf.push_indent();
                    buf.push(&stringify_type(&type_));
                    buf.push(" ");
                    buf.push(&prepare_name(name));
                    buf.push(";\n");
                }

                buf.dec_indent();
                buf.push_indent();
                buf.push("};\n\n");

                buf.push_indent();
                buf.push(&stringify_type(&kind_type));
                buf.push(" ");
                buf.push(&prepare_name(kind_name));
                buf.push(";\n");

                buf.dec_indent();
                buf.push_indent();
                buf.push("} ");
                buf.push(&prepared_name);
                buf.push(";\n");
                self.typedefs.insert(prepared_name, TypeOutput::new(buf, dependencies));
            }
            IrStatementData::Enum { name, variants, type_ } => {
                let prepared_name = prepare_name(Rc::clone(&name));

                let mut buf = CodeOutput::new();
                buf.push_indent();
                buf.push("typedef enum SKYE_ENUM_");
                buf.push(&prepared_name);
                buf.push(": ");
                buf.push(&stringify_type(&type_));
                buf.push(" {\n");
                buf.inc_indent();

                for variant in variants {
                    buf.push_indent();
                    buf.push(&name);
                    buf.push("_DOT_");
                    buf.push(&variant.name);

                    if let Some(value) = variant.value {
                        let generated = ctx.run(|ctx| self.generate_value(value, ctx)).await;

                        buf.push(" = ");
                        buf.push(&generated);
                    }

                    buf.push(",\n");
                }

                buf.dec_indent();
                buf.push_indent();
                buf.push("} ");
                buf.push(&prepared_name);
                buf.push(";\n");

                self.declarations.insert(prepared_name, TypeOutput::independent(buf));
            }
            IrStatementData::Function { name, params, body, signature } => {
                let prepared_name = prepare_name(name);

                if let SkyeType::Function(_, return_type, _) = &signature {
                    let return_stringified = stringify_type(&return_type);

                    if prepared_name.as_ref() == "_SKYE_MAIN" {
                        let returns_void        = return_stringified == "void";
                        let returns_i32         = return_stringified == "i32";
                        let returns_i32_result  = return_stringified == "core_DOT_Result_GENOF_void_GENAND_i32_GENEND_";
                        let returns_void_result = return_stringified == "core_DOT_Result_GENOF_void_GENAND_void_GENEND_";

                        let has_stdargs = {
                            params.len() == 2 &&
                            params[0].type_.equals(&SkyeType::AnyInt, EqualsLevel::Typewise) &&
                            params[1].type_.equals(&SkyeType::Pointer(
                                Box::new(SkyeType::Pointer(
                                    Box::new(SkyeType::Char),
                                    false, false
                                )),
                                false, false
                            ), EqualsLevel::Typewise)
                        };

                        let has_args = {
                            params.len() == 1 &&
                            {
                                if let SkyeType::Struct(full_name, ..) = &params[0].type_ {
                                    full_name.as_ref() == "core_DOT_Array_GENOF_core_DOT_Slice_GENOF_char_GENEND__GENAND_core_DOT_mem_DOT_HeapAllocator_GENEND_"
                                } else {
                                    false
                                }
                            }
                        };

                        let no_args = params.len() == 0;

                        if (returns_void || returns_i32 || returns_i32_result || returns_void_result) && (no_args || has_args || has_stdargs) {
                            self.fndefs.push(CodeOutput::new());

                            if returns_void {
                                if has_stdargs {
                                    self.fndefs.last_mut().unwrap().push(VOID_MAIN_PLUS_STD_ARGS);
                                } else if has_args {
                                    self.fndefs.last_mut().unwrap().push(VOID_MAIN_PLUS_ARGS);
                                } else {
                                    self.fndefs.last_mut().unwrap().push(VOID_MAIN);
                                }
                            } else if returns_i32 {
                                if has_stdargs {
                                    self.fndefs.last_mut().unwrap().push(I32_MAIN_PLUS_STD_ARGS);
                                } else if has_args {
                                    self.fndefs.last_mut().unwrap().push(I32_MAIN_PLUS_ARGS);
                                } else {
                                    self.fndefs.last_mut().unwrap().push(I32_MAIN);
                                }
                            } else if returns_i32_result {
                                if has_stdargs {
                                    self.fndefs.last_mut().unwrap().push(RESULT_I32_MAIN_PLUS_STD_ARGS);
                                } else if has_args {
                                    self.fndefs.last_mut().unwrap().push(RESULT_I32_MAIN_PLUS_ARGS);
                                } else {
                                    self.fndefs.last_mut().unwrap().push(RESULT_I32_MAIN);
                                }
                            } else if returns_void_result {
                                if has_stdargs {
                                    self.fndefs.last_mut().unwrap().push(RESULT_VOID_MAIN_PLUS_STD_ARGS);
                                } else if has_args {
                                    self.fndefs.last_mut().unwrap().push(RESULT_VOID_MAIN_PLUS_ARGS);
                                } else {
                                    self.fndefs.last_mut().unwrap().push(RESULT_VOID_MAIN);
                                }
                            }
                        }
                    }

                    let mut buf = CodeOutput::new();
                    buf.push_indent();
                    buf.push(&return_stringified);
                    buf.push(" ");
                    buf.push(&prepared_name);
                    buf.push("(");

                    let mut params_string = String::new();

                    for (i, param) in params.iter().enumerate() {
                        params_string.push_str(&stringify_type(&param.type_));
                        params_string.push(' ');
                        params_string.push_str(&param.name);

                        if i != params.len() - 1 {
                            params_string.push_str(", ");
                        }
                    }

                    self.generate_fn_signature(&signature, &return_stringified, &params_string);

                    buf.push(&params_string);
                    buf.push(")");

                    if let Some(body) = body {
                        let mut decl_buf = buf.clone();
                        decl_buf.push(";\n");
                        let dependencies = get_type_dependencies_declarations(&signature);
                        self.declarations.insert(prepared_name, TypeOutput::new(decl_buf, dependencies));

                        buf.push(" {\n");
                        buf.inc_indent();
                        self.fndefs.push(buf);
                        self.in_function = true;

                        for statement in body {
                            ctx.run(|ctx| self.generate_statement(statement, ctx)).await;
                        }

                        self.in_function = false;
                        self.end_scope();
                    } else {
                        buf.push(";\n");
                        let dependencies = get_type_dependencies_declarations(&signature);
                        self.declarations.insert(prepared_name, TypeOutput::new(buf, dependencies));
                    }
                } else {
                    unreachable!()
                }
            }
            IrStatementData::Switch { value, branches } => {
                let generated = ctx.run(|ctx| self.generate_value(value, ctx)).await;

                self.fndefs.last_mut().unwrap().push_indent();
                self.fndefs.last_mut().unwrap().push("switch (");
                self.fndefs.last_mut().unwrap().push(&generated);
                self.fndefs.last_mut().unwrap().push(") {\n");
                self.fndefs.last_mut().unwrap().inc_indent();

                for branch in branches {
                    if branch.cases.len() == 0 {
                        self.fndefs.last_mut().unwrap().push_indent();
                        self.fndefs.last_mut().unwrap().push("default:\n");
                    } else {
                        for case in branch.cases {
                            let generated = ctx.run(|ctx| self.generate_value(case, ctx)).await;

                            self.fndefs.last_mut().unwrap().push_indent();
                            self.fndefs.last_mut().unwrap().push("case ");
                            self.fndefs.last_mut().unwrap().push(&generated);
                            self.fndefs.last_mut().unwrap().push(":\n");
                        }
                    }

                    self.fndefs.last_mut().unwrap().inc_indent();
                    let not_block = !matches!(branch.code.data, IrStatementData::Scope { .. });

                    if not_block {
                        self.begin_scope();
                    }

                    ctx.run(|ctx| self.generate_statement(branch.code, ctx)).await;

                    if not_block {
                        self.end_scope();
                    }

                    self.fndefs.last_mut().unwrap().push_indent();
                    self.fndefs.last_mut().unwrap().push("break;\n");
                    self.fndefs.last_mut().unwrap().dec_indent();
                }

                self.end_scope();
            }
        }
    }

    pub fn generate(&mut self, statements: Vec<IrStatement>) -> Option<String> {
        let mut stack = reblessive::Stack::new();

        for statement in statements {
            let _ = stack.enter(|ctx| self.generate_statement(statement, ctx)).finish();
        }

        let mut topo_sort_decls = TopoSort::with_capacity(self.declarations.len());
        for (name, declaration) in &mut self.declarations {
            topo_sort_decls.insert(Rc::clone(&name), declaration.dependencies.take().unwrap());
        }

        let decls_order = {
            match topo_sort_decls.into_vec_nodes() {
                SortResults::Full(items) => items,
                SortResults::Partial(_) => panic!("dependency cycle")
            }
        };

        let mut topo_sort_typedefs = TopoSort::with_capacity(self.typedefs.len());
        for (name, definition) in &mut self.typedefs {
            topo_sort_typedefs.insert(Rc::clone(&name), definition.dependencies.take().unwrap());
        }

        let typedefs_order = {
            match topo_sort_typedefs.into_vec_nodes() {
                SortResults::Full(items) => items,
                SortResults::Partial(_) => panic!("dependency cycle")
            }
        };

        let mut output = String::from("// Hello from Skye!! ^_^\n\n");

        if self.includes.code.len() != 0 {
            output.push_str(&self.includes.code);
            output.push('\n');
        }

        if self.strings_code.code.len() != 0 {
            output.push_str(&self.strings_code.code);
            output.push('\n');
        }

        if self.declarations.len() != 0 {
            for name in decls_order {
                output.push_str(&self.declarations.get(&name).unwrap().output.code);
            }

            output.push('\n');
        }

        for name in typedefs_order {
            output.push_str(&self.typedefs.get(&name).unwrap().output.code);
            output.push('\n');
        }

        for definition in &self.fndefs {
            output.push_str(&definition.code);
            output.push('\n');
        }

        Some(output)
    }
}