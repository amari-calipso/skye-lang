use std::{cell::RefCell, collections::{HashMap, HashSet}, ffi::OsString, path::{Path, PathBuf}, rc::Rc};

use lazy_static::lazy_static;

use crate::{
    ast::{Ast, EnumVariant, Expression, FunctionParam, ImportType, LiteralKind, MacroBody, MacroParams, Statement, SwitchCase}, ast_error, ast_info, ast_note, ast_warning, astpos_note, environment::{Environment, SkyeVariable}, parser::Parser, scanner::Scanner, skye_type::{CastableHow, EqualsLevel, GetResult, ImplementsHow, Operator, SkyeEnumVariant, SkyeFunctionParam, SkyeType, SkyeValue}, token_error, token_note, token_warning, tokens::{Token, TokenType}, utils::{escape_string, fix_raw_string, get_real_string_length, note}, CompileMode
};

const OUTPUT_INDENT_SPACES: usize = 4;

lazy_static! {
    pub static ref BUILTIN_MACROS: HashSet<&'static str> = HashSet::from([
        "format", "fprint", "fprintln", "typeOf", "cast", "constCast"
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

const SKYE_INIT_DECL: &str = "void _SKYE_INIT();\n";
const SKYE_INIT_DEF:  &str = "void _SKYE_INIT() {\n";

const INIT_DEF_INDEX: usize = 0;
const ANY_DEF_START_INDEX: usize = 1;

#[derive(Clone, Debug)]
enum CurrentFn {
    None,
    Some(SkyeType, Expression)
}

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

    pub fn set_indent(&mut self, indent: usize) {
        self.indent = indent;
    }
}

pub enum ExecutionInterrupt {
    Interrupt(Rc<str>),
    Return(Rc<str>)
}

pub struct CodeGen {
    source_path: Option<Box<PathBuf>>,
    skye_path:   PathBuf,

    strings:            HashMap<Rc<str>, usize>,
    strings_code:       CodeOutput,
    includes:           CodeOutput,
    declarations:       Vec<CodeOutput>,
    struct_definitions: HashMap<Rc<str>, CodeOutput>,
    struct_defs_order:  Vec<Rc<str>>,
    definitions:        Vec<CodeOutput>,
    string_type:        Option<SkyeType>,
    tmp_var_cnt:        usize,

    globals:     Rc<RefCell<Environment>>,
    environment: Rc<RefCell<Environment>>,

    deferred:      Rc<RefCell<Vec<Vec<Statement>>>>,
    curr_function: CurrentFn,
    curr_name:     String,
    curr_loop:     Option<(Rc<str>, Rc<str>)>,

    errors:       usize,
    compile_mode: CompileMode
}

impl CodeGen {
    pub fn new(path: Option<&Path>, compile_mode: CompileMode, skye_path: PathBuf) -> Self {
        let globals = Rc::new(RefCell::new(Environment::new()));

        globals.borrow_mut().define(
            Rc::from("voidptr"),
            SkyeVariable::new(
                SkyeType::Type(
                    Box::new(SkyeType::Pointer(
                        Box::new(SkyeType::Void),
                        false, false
                    ))
                ),
                true, None
            )
        );

        let mut declarations = Vec::new();
        declarations.push(CodeOutput::new());
        declarations.last_mut().unwrap().push(SKYE_INIT_DECL);

        let mut definitions = Vec::new();
        definitions.push(CodeOutput::new());
        definitions.last_mut().unwrap().push(SKYE_INIT_DEF);
        definitions.last_mut().unwrap().inc_indent();

        CodeGen {
            struct_definitions: HashMap::new(),
            struct_defs_order: Vec::new(),
            declarations, definitions,
            strings_code: CodeOutput::new(),
            includes: CodeOutput::new(),
            strings: HashMap::new(),
            curr_name: String::new(),
            environment: Rc::clone(&globals),
            deferred: Rc::new(RefCell::new(Vec::new())),
            curr_function: CurrentFn::None,
            string_type: None, tmp_var_cnt: 0,
            curr_loop: None, errors: 0,
            compile_mode, globals, skye_path,
            source_path: path.map(|x| Box::new(PathBuf::from(x))),
        }
    }

    fn get_name(&self, name: &Rc<str>) -> Rc<str> {
        if self.curr_name == "" {
            Rc::clone(&name)
        } else {
            Rc::from(format!("{}_DOT_{}", self.curr_name, name))
        }
    }

    fn get_generics(&self, name: &Rc<str>, generics: &Vec<Token>, env: &Rc<RefCell<Environment>>) -> Rc<str> {
        if generics.len() == 0 {
            Rc::clone(name)
        } else {
            let mut buf = String::new();
            buf.push_str(name);
            buf.push_str("_GENOF_");

            for (i, generic) in generics.iter().enumerate() {
                if let Some(var) = env.borrow().get(generic) {
                    match var.type_ {
                        SkyeType::Type(inner_type) => {
                            if inner_type.can_be_instantiated(false) {
                                buf.push_str(&inner_type.mangle());
                            }
                        }
                        SkyeType::Void => buf.push_str("void"),
                        SkyeType::Unknown(_) => buf.push_str("_UNKNOWN_"),
                        _ => ()
                    }
                }

                if i != generics.len() - 1 {
                    buf.push_str("_GENAND_");
                }
            }

            buf.push_str("_GENEND_");
            Rc::from(buf)
        }
    }

    async fn get_return_type(&mut self, return_type_expr: &Expression, index: usize, allow_unknown: bool, ctx: &mut reblessive::Stk) -> SkyeType {
        let val = ctx.run(|ctx| self.evaluate(&return_type_expr, index, allow_unknown, ctx)).await;

        match val.type_ {
            SkyeType::Type(inner_type) => {
                if inner_type.check_completeness() {
                    if inner_type.can_be_instantiated(false) {
                        *inner_type
                    } else {
                        ast_error!(self, return_type_expr, format!("Cannot instantiate type {}", inner_type.stringify_native()).as_ref());
                        SkyeType::get_unknown()
                    }
                } else {
                    ast_error!(self, return_type_expr, "Cannot use incomplete type directly");
                    ast_note!(return_type_expr, "Define this type or reference it through a pointer");
                    SkyeType::get_unknown()
                }
            }
            SkyeType::Void => val.type_,
            _ => {
                ast_error!(self, return_type_expr, format!("Expecting type as return type (got {})", val.type_.stringify_native()).as_ref());
                SkyeType::get_unknown()
            }
        }
    }

    async fn get_params(&mut self, params: &Vec<FunctionParam>, existing: Option<SkyeVariable>, has_decl: bool, index: usize, allow_unknown: bool, ctx: &mut reblessive::Stk) -> (String, Vec<SkyeFunctionParam>) {
        let mut params_string = String::new();
        let mut params_output = Vec::with_capacity(params.len());
        for i in 0 .. params.len() {
            let param_type: SkyeType = {
                let inner_param_type = ctx.run(|ctx| self.evaluate(&params[i].type_, index, allow_unknown, ctx)).await.type_;
                if inner_param_type.check_completeness() {
                    if let SkyeType::Type(inner_type) = inner_param_type {
                        if inner_type.can_be_instantiated(false) {
                            if has_decl {
                                if let SkyeType::Function(existing_params, ..) = &existing.as_ref().unwrap().type_ {
                                    if !existing_params[i].type_.equals(&inner_type, EqualsLevel::Typewise) {
                                        ast_error!(
                                            self, params[i].type_,
                                            format!(
                                                "Function parameter type does not match declaration parameter type (expecting {} but got {})",
                                                inner_type.stringify_native(), existing_params[i].type_.stringify_native()
                                            ).as_ref()
                                        );
                                    }
                                }
                            }

                            *inner_type
                        } else {
                            ast_error!(
                                self, params[i].type_,
                                format!("Cannot instantiate type {}", inner_type.stringify_native()).as_ref()
                            );

                            SkyeType::get_unknown()
                        }
                    } else {
                        ast_error!(
                            self, params[i].type_,
                            format!(
                                "Expecting type as parameter type (got {})",
                                inner_param_type.stringify_native()
                            ).as_ref()
                        );

                        SkyeType::get_unknown()
                    }
                } else {
                    ast_error!(self, params[i].type_, "Cannot use incomplete type directly");
                    ast_note!(params[i].type_, "Define this type or reference it through a pointer");
                    SkyeType::get_unknown()
                }
            };

            params_output.push(SkyeFunctionParam::new(param_type.clone(), params[i].is_const));
            params_string.push_str(&param_type.stringify());

            if let Some(param) = &params[i].name {
                params_string.push(' ');
                params_string.push_str(&param.lexeme);
            }

            if i != params.len() - 1 {
                params_string.push_str(", ");
            }
        }

        (params_string, params_output)
    }

    fn generate_fn_signature(&mut self, tok: &Token, inner_type: &SkyeType, return_stringified: &String, params_string: &String) -> (String, SkyeType) {
        let mangled = inner_type.mangle();
        let type_ = SkyeType::Type(Box::new(inner_type.clone()));

        let env = self.globals.borrow();

        let existing = env.get(&Token::dummy(mangled.clone().into()));
        if let Some(fnptr) = existing {
            if !fnptr.type_.equals(&type_, EqualsLevel::Typewise) {
                if let Some(orig_tok) = fnptr.tok {
                    token_error!(self, tok, "This function pointer's mangled type resolves to a different type");
                    token_note!(orig_tok, "This definition is invalid. Change the name of this symbol");
                } else {
                    token_error!(self, tok, "This function pointer's mangled type resolves to a different type. An invalid symbol definition is present in the code");
                }
            }
        } else {
            drop(env);

            self.declarations.push(CodeOutput::new());
            self.declarations.last_mut().unwrap().push("typedef ");
            self.declarations.last_mut().unwrap().push(return_stringified);
            self.declarations.last_mut().unwrap().push(" (*");
            self.declarations.last_mut().unwrap().push(&mangled);
            self.declarations.last_mut().unwrap().push(")(");
            self.declarations.last_mut().unwrap().push(&params_string);
            self.declarations.last_mut().unwrap().push(");\n");

            let mut env = self.globals.borrow_mut();
            env.define(
                mangled.clone().into(),
                SkyeVariable::new(
                    type_.clone(), true,
                    Some(Box::new(tok.clone()))
                )
            );
        }

        (mangled, type_)
    }

    fn get_temporary_var(&mut self) -> String {
        let res = format!("SKYE_TMP_{}", self.tmp_var_cnt);
        self.tmp_var_cnt += 1;
        res
    }

    fn get_method(&mut self, object: &SkyeValue, name: &Token, strict: bool, index: usize) -> Option<SkyeValue> {
        match object.type_.get_method(name, strict) {
            GetResult::Ok(value, ..) => {
                let search_tok = Token::dummy(Rc::clone(&value));

                let result = self.globals.borrow().get(&search_tok);

                if let Some(var) = result {
                    return Some(SkyeValue::with_self_info(
                        value, var.type_, true,
                        object.type_.get_self(
                            &object.value,
                            object.is_const,
                            self.external_zero_check(name, index)
                        ).expect("get_self failed")
                    ))
                } else {
                    None
                }
            }
            _ => None
        }
    }

    pub fn split_interpolated_string(&mut self, str: &Rc<str>, ref_token: &Token) -> Vec<(String, bool)> {
        let mut result: Vec<(String, bool)> = Vec::new();

        let mut interpolated = 0usize;
        let mut escaped = false;
        for (i, ch) in str.chars().enumerate() {
            if ch == '{' {
                if escaped {
                    if result.len() == 0 {
                        result.push((String::new(), false));
                    }

                    escaped = false;
                    result.last_mut().unwrap().0.push(ch);
                    continue;
                } else {
                    if i == str.len() - 1 {
                        token_error!(self, ref_token, "Invalid interpolated string");
                        note(
                            &ref_token.source, "Brackets must be matched '{{'",
                            &ref_token.filename, ref_token.pos + i, 1,
                            ref_token.line
                        );
                    } else if str.chars().nth(i + 1).unwrap() == '{' {
                        escaped = true;
                    } else {
                        if interpolated == 0 {
                            result.push((String::new(), true));
                        }

                        interpolated += 1;
                    }
                }
            } else if ch == '}' {
                if escaped {
                    if result.len() == 0 {
                        result.push((String::new(), false));
                    }

                    escaped = false;
                    result.last_mut().unwrap().0.push(ch);
                    continue;
                } else {
                    if interpolated == 0 && i != str.len() - 1 && str.chars().nth(i + 1).unwrap() == '}' {
                        escaped = true;
                    } else {
                        if interpolated == 0 {
                            token_error!(self, ref_token, "Invalid interpolated string");
                            note(
                                &ref_token.source, "Unbalanced brackets",
                                &ref_token.filename, ref_token.pos + i, 1,
                                ref_token.line
                            );
                        }

                        interpolated -= 1;
                        if interpolated == 0 {
                            result.push((String::new(), false));
                        }
                    }
                }
            } else {
                if result.len() == 0 {
                    result.push((String::new(), false));
                }

                result.last_mut().unwrap().0.push(ch);
            }
        }

        result
    }

    async fn handle_builtin_macros(&mut self, macro_name: &Rc<str>, arguments: &Vec<Expression>, index: usize, allow_unknown: bool, _callee_expr: &Expression, ctx: &mut reblessive::Stk) -> Option<SkyeValue> {
        match macro_name.as_ref() {
            "format" | "fprint" | "fprintln" => {
                let is_format   = macro_name.as_ref() == "format";
                let is_fprintln = macro_name.as_ref() == "fprintln";

                let first = ctx.run(|ctx| self.evaluate(&arguments[0], index, allow_unknown, ctx)).await;

                let (real_fmt_string, tok) = {
                    if let Expression::Literal { value, tok: string_tok, kind } = arguments[1].get_inner() {
                        if matches!(kind, LiteralKind::String) {
                            (value, string_tok)
                        } else {
                            ast_error!(self, arguments[1], "Format string must be a string");
                            return Some(SkyeValue::special(SkyeType::Void));
                        }
                    } else {
                        ast_error!(self, arguments[1], "Format string must be a literal");
                        ast_note!(arguments[1], "The format string must be known at compile time for the compiler to generate the necessary code");
                        return Some(SkyeValue::special(SkyeType::Void));
                    }
                };

                let mut splitted = self.split_interpolated_string(&real_fmt_string, &tok);

                if is_fprintln {
                    splitted.push((String::from("\\n"), false));
                }

                let mut statements = Vec::new();
                for (portion, mut interpolated) in &splitted {
                    if portion == "" {
                        continue;
                    }

                    let portion_expr = {
                        if interpolated {
                            let mut scanner = Scanner::new(&portion, Rc::clone(&tok.filename));
                            scanner.scan_tokens();

                            if scanner.had_error {
                                self.errors += 1;
                                token_note!(tok, "This error occurred while lexing this interpolated string");
                                continue;
                            }

                            let mut parser = Parser::new(scanner.tokens);
                            if let Some(result) = parser.expression() {
                                if let Expression::Literal { kind, .. } = result.get_inner() {
                                    if matches!(kind, LiteralKind::String) {
                                        interpolated = false;
                                    }
                                }

                                result
                            } else {
                                self.errors += 1;
                                token_note!(tok, "This error occurred while parsing this interpolated string");
                                continue;
                            }
                        } else {
                            Expression::Literal { value: Rc::from(portion.as_ref()), tok: tok.clone(), kind: LiteralKind::String }
                        }
                    };

                    // this evaluation will be performed again later, so generate the code in a scratch buffer
                    let tmp_index = self.definitions.len();
                    self.definitions.push(CodeOutput::new());
                    let evaluated = ctx.run(|ctx| self.evaluate(&portion_expr, tmp_index, allow_unknown, ctx)).await;
                    self.definitions.swap_remove(tmp_index);

                    let mut do_write = true;
                    let interpolated_expr = 'interpolated_expr_blk: {
                        if interpolated {
                            if SkyeType::AnyInt.is_respected_by(&evaluated.type_) {
                                do_write = false;

                                if is_format {
                                    break 'interpolated_expr_blk Expression::Call(
                                        Box::new(Expression::Variable(Token::dummy(Rc::from("core_DOT_fmt_DOT_intToBuf")))),
                                        tok.clone(),
                                        vec![arguments[0].clone(), portion_expr]
                                    );
                                } else {
                                    break 'interpolated_expr_blk Expression::Call(
                                        Box::new(Expression::Variable(Token::dummy(Rc::from("core_DOT_fmt_DOT___intToFile")))),
                                        tok.clone(),
                                        vec![arguments[0].clone(), portion_expr]
                                    );
                                }
                            }

                            if SkyeType::AnyFloat.is_respected_by(&evaluated.type_) {
                                do_write = false;

                                if is_format {
                                    break 'interpolated_expr_blk Expression::Call(
                                        Box::new(Expression::Variable(Token::dummy(Rc::from("core_DOT_fmt_DOT_floatToBuf")))),
                                        tok.clone(),
                                        vec![arguments[0].clone(), portion_expr]
                                    );
                                } else {
                                    break 'interpolated_expr_blk Expression::Call(
                                        Box::new(Expression::Variable(Token::dummy(Rc::from("core_DOT_fmt_DOT___floatToFile")))),
                                        tok.clone(),
                                        vec![arguments[0].clone(), portion_expr]
                                    );
                                }
                            }

                            if let SkyeType::Struct(full_name, ..) = &evaluated.type_ {
                                if full_name.as_ref() == "core_DOT_Slice_GENOF_char_GENEND_" {
                                    break 'interpolated_expr_blk portion_expr;
                                }
                            }

                            if matches!(evaluated.type_, SkyeType::Char) {
                                break 'interpolated_expr_blk Expression::Slice { opening_brace: tok.clone(), items: vec![portion_expr] };
                            }

                            let mut search_tok = Token::dummy(Rc::from("asString"));
                            if self.get_method(&evaluated, &search_tok, false, index).is_some() {
                                Expression::Call(
                                    Box::new(Expression::Get(
                                        Box::new(Expression::Grouping(
                                            Box::new(portion_expr.clone())
                                        )),
                                        search_tok
                                    )),
                                    tok.clone(),
                                    Vec::new()
                                )
                            } else {
                                search_tok = Token::dummy(Rc::from("toString"));
                                if self.get_method(&evaluated, &search_tok, false, index).is_some() {
                                    Expression::Call(
                                        Box::new(Expression::Get(
                                            Box::new(Expression::Grouping(
                                                Box::new(portion_expr.clone())
                                            )),
                                            search_tok
                                        )),
                                        tok.clone(),
                                        Vec::new()
                                    )
                                } else {
                                    ast_error!(
                                        self, portion_expr,
                                        format!(
                                            "Type {} is not printable",
                                            evaluated.type_.stringify_native()
                                        ).as_ref()
                                    );

                                    ast_note!(portion_expr, "Implement a \"asString\" or \"toString\" method to be able to print this type");
                                    token_note!(tok, "This error occurred while evaluating this interpolated string");
                                    Expression::Literal { value: Rc::from(""), tok: tok.clone(), kind: LiteralKind::Void }
                                }
                            }
                        } else {
                            portion_expr
                        }
                    };

                    if is_format {
                        let search_tok = Token::dummy(Rc::from("pushString"));
                        if self.get_method(&first, &search_tok, false, index).is_some() {
                            if do_write {
                                statements.push(Statement::Expression(
                                    Expression::Call(
                                        Box::new(Expression::Get(
                                            Box::new(Expression::Grouping(
                                                Box::new(arguments[0].clone())
                                            )),
                                            search_tok
                                        )),
                                        tok.clone(),
                                        vec![interpolated_expr]
                                    )
                                ));
                            } else {
                                statements.push(Statement::Expression(interpolated_expr));
                            }
                        } else {
                            ast_error!(
                                self, arguments[0],
                                format!(
                                    "Type {} is not a valid string buffer",
                                    evaluated.type_.stringify_native()
                                ).as_ref()
                            );

                            ast_note!(arguments[0], "This type does not implement a \"pushString\" method");
                        }
                    } else {
                        let search_tok = Token::dummy(Rc::from("write"));
                        if self.get_method(&first, &search_tok, false, index).is_some() {
                            if do_write {
                                statements.push(Statement::Expression(
                                    Expression::Call(
                                        Box::new(Expression::Get(
                                            Box::new(Expression::Call(
                                                Box::new(Expression::Get(
                                                    Box::new(Expression::Grouping(
                                                        Box::new(arguments[0].clone())
                                                    )),
                                                    search_tok
                                                )),
                                                tok.clone(),
                                                vec![interpolated_expr]
                                            )),
                                            Token::dummy(Rc::from("expect"))
                                        )),
                                        tok.clone(),
                                        vec![Expression::Literal { value: Rc::from("String interpolation failed writing to file"), tok: tok.clone(), kind: LiteralKind::String }]
                                    )
                                ));
                            } else {
                                statements.push(Statement::Expression(interpolated_expr));
                            }
                        } else {
                            ast_error!(
                                self, arguments[0],
                                format!(
                                    "Type {} is not a valid writable object",
                                    first.type_.stringify_native()
                                ).as_ref()
                            );

                            ast_note!(arguments[0], "This type does not implement a \"write\" method");
                        }
                    }
                }

                let stmts = Statement::Block(tok.clone(), statements);
                let _ = ctx.run(|ctx| self.execute(&stmts, index, ctx)).await;
                Some(SkyeValue::special(SkyeType::Void))
            }
            "typeOf" => {
                let inner = ctx.run(|ctx| self.evaluate(&arguments[0], index, allow_unknown, ctx)).await;

                match inner.type_ {
                    SkyeType::Void         => ast_error!(self, arguments[0], "Cannot get type of void"),
                    SkyeType::Type(_)      => ast_error!(self, arguments[0], "Cannot get type of type"),
                    SkyeType::Group(..)    => ast_error!(self, arguments[0], "Cannot get type of type group"),
                    SkyeType::Namespace(_) => ast_error!(self, arguments[0], "Cannot get type of namespace"),
                    SkyeType::Template(..) => ast_error!(self, arguments[0], "Cannot get type of template"),
                    SkyeType::Macro(..)    => ast_error!(self, arguments[0], "Cannot get type of macro"),
                    _ => return Some(SkyeValue::special(SkyeType::Type(Box::new(inner.type_))))
                }

                Some(SkyeValue::special(inner.type_))
            }
            "cast" => {
                let cast_to = ctx.run(|ctx| self.evaluate(&arguments[0], index, allow_unknown, ctx)).await;

                if let SkyeType::Type(inner_type) = cast_to.type_ {
                    let to_cast = ctx.run(|ctx| self.evaluate(&arguments[1], index, allow_unknown, ctx)).await;

                    let castable_how = to_cast.type_.is_castable_to(&inner_type);
                    if matches!(castable_how, CastableHow::Yes | CastableHow::ConstnessLoss) {
                        if matches!(castable_how, CastableHow::ConstnessLoss) {
                            ast_warning!(arguments[1], "This cast discards the constness from casted type"); // +W-constness-loss
                            ast_note!(arguments[0], "Cast to a const variant of this type");

                            if matches!(to_cast.type_, SkyeType::Pointer(..)) {
                                ast_note!(arguments[1], "Since this is a pointer, you can also use the @constCast macro to discard its constness");
                            }
                        }

                        if inner_type.equals(&to_cast.type_, EqualsLevel::ConstStrict) {
                            Some(to_cast)
                        } else {
                            Some(SkyeValue::new(Rc::from(format!("({})({})", inner_type.stringify(), to_cast.value)), *inner_type, true))
                        }
                    } else {
                        // cast from specific type to interface
                        if let SkyeType::Enum(full_name, variants, _) = &*inner_type {
                            if let Some(real_variants) = variants {
                                let mangled = to_cast.type_.mangle();
                                if let Some(result) = real_variants.get(&Rc::from(mangled.as_ref())) {
                                    if result.equals(&to_cast.type_, EqualsLevel::Typewise) {
                                        return Some(SkyeValue::new(Rc::from(format!("{}_DOT_{}({})", full_name, mangled, to_cast.value).as_ref()), *inner_type, true));
                                    }
                                }
                            }
                        }

                        // cast from interface to specific type
                        if let SkyeType::Enum(_, variants, base_name) = &to_cast.type_ {
                            if let Some(real_variants) = variants {
                                let mangled = inner_type.mangle();
                                if let Some(result) = real_variants.get(&Rc::from(mangled.as_ref())) {
                                    if result.equals(&inner_type, EqualsLevel::Typewise) {
                                        let mut question = Token::dummy(Rc::from(""));
                                        let mut custom_tok = question.clone();
                                        question.set_type(TokenType::Question);
                                        custom_tok.set_lexeme(&mangled);

                                        let option_expr = Expression::Unary { op: question, expr: Box::new(Expression::Variable(custom_tok)), is_prefix: true };
                                        let option_type = ctx.run(|ctx| self.evaluate(&option_expr, index, allow_unknown, ctx)).await;

                                        if let SkyeType::Type(inner_option_type) = option_type.type_ {
                                            let mangled_option_type = inner_option_type.mangle();

                                            let tmp_var = self.get_temporary_var();

                                            self.definitions[index].push_indent();
                                            self.definitions[index].push(&to_cast.type_.stringify());
                                            self.definitions[index].push(" ");
                                            self.definitions[index].push(&tmp_var);
                                            self.definitions[index].push(" = ");
                                            self.definitions[index].push(&to_cast.value);
                                            self.definitions[index].push(";\n");

                                            return Some(SkyeValue::new(
                                                Rc::from(format!(
                                                    "({}.kind == {}_DOT_Kind_DOT_{} ? {}_DOT_Some({}.{}) : {}_DOT_None)",
                                                    tmp_var, base_name, mangled, mangled_option_type, tmp_var, mangled, mangled_option_type
                                                ).as_ref()), *inner_option_type, true
                                            ));
                                        } else {
                                            panic!("option type generation resulted in not a type");
                                        }
                                    }
                                }
                            }
                        }

                        ast_error!(
                            self, arguments[1],
                            format!(
                                "Type {} cannot be casted to type {}",
                                to_cast.type_.stringify_native(),
                                inner_type.stringify_native()
                            ).as_ref()
                        );

                        Some(SkyeValue::special(*inner_type))
                    }
                } else {
                    ast_error!(
                        self, arguments[0],
                        format!(
                            "Expecting type as cast type (got {})",
                            cast_to.type_.stringify_native()
                        ).as_ref()
                    );

                    Some(SkyeValue::special(cast_to.type_))
                }
            }
            "constCast" => {
                let to_cast = ctx.run(|ctx| self.evaluate(&arguments[0], index, allow_unknown, ctx)).await;

                if let SkyeType::Pointer(inner_type, is_const, is_reference) = &to_cast.type_ {
                    if *is_const {
                        Some(SkyeValue::new(to_cast.value, SkyeType::Pointer(inner_type.clone(), false, *is_reference), true))
                    } else {
                        Some(to_cast)
                    }
                } else {
                    ast_error!(
                        self, arguments[0],
                        format!(
                            "Expecting pointer as @constCast argument (got {})",
                            to_cast.type_.stringify_native()
                        ).as_ref()
                    );

                    Some(to_cast)
                }
            }
            _ => None
        }
    }

    fn output_call(&mut self, return_type: &SkyeType, callee_value: &str, args: &str, index: usize) -> String {
        self.definitions[index].push_indent();

        let tmp_var_name = {
            if matches!(return_type, SkyeType::Void) {
                String::new()
            } else {
                let tmp_var = self.get_temporary_var();

                self.definitions[index].push(&return_type.stringify());
                self.definitions[index].push(" ");
                self.definitions[index].push(&tmp_var);
                self.definitions[index].push(" = ");

                tmp_var
            }
        };

        self.definitions[index].push(callee_value);
        self.definitions[index].push("(");
        self.definitions[index].push(args);
        self.definitions[index].push(");\n");

        tmp_var_name
    }

    async fn call(&mut self, callee: &SkyeValue, expr: &Expression, callee_expr: &Expression, arguments: &Vec<Expression>, index: usize, allow_unknown: bool, ctx: &mut reblessive::Stk) -> SkyeValue {
        let (arguments_len, arguments_mod) = {
            if callee.self_info.is_some() {
                (arguments.len() + 1, 1 as usize)
            } else {
                (arguments.len(), 0 as usize)
            }
        };

        match &callee.type_ {
            SkyeType::Function(params, return_type, _) => {
                if params.len() != arguments_len {
                    ast_error!(
                        self, expr,
                        format!(
                            "Expecting {} arguments for function call but got {}",
                            params.len(), arguments_len
                        ).as_str()
                    );

                    return SkyeValue::special(*return_type.clone());
                }

                let mut args = String::new();
                for i in 0 .. arguments_len {
                    let arg = 'argblock: {
                        if i == 0 {
                            if let Some((info_val, info_type)) = &callee.self_info {
                                if let SkyeType::Pointer(_, is_const, _) = info_type {
                                    break 'argblock SkyeValue::new(
                                        Rc::clone(info_val),
                                        info_type.clone(),
                                        *is_const
                                    );
                                } else {
                                    unreachable!()
                                }
                            }
                        }

                        ctx.run(|ctx| self.evaluate(&arguments[i - arguments_mod], index, allow_unknown, ctx)).await
                    };

                    let is_self = i == 0 && arguments_mod == 1;

                    let new_arg = {
                        if is_self {
                            arg
                        } else if let SkyeType::Pointer(.., is_reference) = &params[i].type_ {
                            if *is_reference && !matches!(arg.type_, SkyeType::Pointer(..)) {
                                // automatically create reference for pass-by-reference params
                                let arg_pos = arguments[i - arguments_mod].get_pos();
                                let custom_tok = Token::new(
                                    arg_pos.source, arg_pos.filename,
                                    TokenType::BitwiseAnd, Rc::from(""),
                                    arg_pos.start, arg_pos.end, arg_pos.line
                                );

                                let ref_expr = Expression::Unary { op: custom_tok, expr: Box::new(Expression::Grouping(
                                        Box::new(arguments[i - arguments_mod].clone())
                                    )), is_prefix: true };

                                ctx.run(|ctx| self.evaluate(&ref_expr, index, allow_unknown, ctx)).await
                            } else {
                                arg
                            }
                        } else {
                            arg
                        }
                    };

                    if params[i].type_.equals(&new_arg.type_, EqualsLevel::Permissive) {
                        if !params[i].type_.equals(&new_arg.type_, EqualsLevel::Strict) {
                            if is_self {
                                if let Some((_, self_type)) = &callee.self_info {
                                    ast_error!(
                                        self, callee_expr,
                                        format!(
                                            "This method cannot be called from type {}",
                                            self_type.stringify_native()
                                        ).as_ref()
                                    );
                                } else {
                                    unreachable!()
                                }
                            } else {
                                ast_error!(
                                    self, arguments[i - arguments_mod],
                                    format!(
                                        "Argument type does not match parameter type (expecting {} but got {})",
                                        params[i].type_.stringify_native(), new_arg.type_.stringify_native()
                                    ).as_ref()
                                );
                            }
                        }
                    } else {
                        if is_self {
                            ast_error!(self, callee_expr, "This method cannot be called from a const source");
                        } else {
                            ast_error!(
                                self, arguments[i - arguments_mod],
                                format!(
                                    "Argument type does not match parameter type (expecting {} but got {})",
                                    params[i].type_.stringify_native(), new_arg.type_.stringify_native()
                                ).as_ref()
                            );
                        }
                    }

                    let search_tok = Token::dummy(Rc::from("__copy__"));
                    if let Some(value) = self.get_method(&new_arg, &search_tok, true, index) {
                        let v = Vec::new();
                        let copy_constructor = ctx.run(|ctx| self.call(&value, expr, &arguments[i - arguments_mod], &v, index, allow_unknown, ctx)).await;
                        args.push_str(&copy_constructor.value);

                        ast_info!(arguments[i - arguments_mod], "Skye inserted a copy constructor call for this expression"); // +I-copies
                    } else {
                        args.push_str(&new_arg.value);
                    }

                    if i != arguments_len - 1 {
                        args.push_str(", ");
                    }
                }

                let call_output = self.output_call(return_type, &callee.value, &args, index);
                SkyeValue::new(Rc::from(call_output.as_ref()), *return_type.clone(), false)
            }
            SkyeType::Template(name, definition, generics, generics_names, curr_name, read_env) => {
                if let Statement::Function { params, return_type: return_type_expr, .. } = definition {
                    if params.len() != arguments_len {
                        ast_error!(
                            self, expr,
                            format!(
                                "Expecting {} arguments for function call but got {}",
                                params.len(), arguments_len
                            ).as_str()
                        );

                        return SkyeValue::get_unknown();
                    }

                    let mut generics_to_find = HashMap::new();
                    for generic in generics {
                        generics_to_find.insert(Rc::clone(&generic.name.lexeme), None);
                    }

                    let tmp_env = Rc::new(RefCell::new(
                        Environment::with_enclosing(Rc::clone(&read_env))
                    ));

                    let mut generics_found_at = HashMap::new();
                    let mut args = String::new();
                    for i in 0 .. arguments_len {
                        let call_evaluated = 'argblock: {
                            if i == 0 {
                                if let Some((info_val, info_type)) = &callee.self_info {
                                    if let SkyeType::Pointer(_, is_const, _) = info_type {
                                        break 'argblock SkyeValue::new(
                                            Rc::clone(info_val),
                                            info_type.clone(),
                                            *is_const
                                        );
                                    } else {
                                        unreachable!()
                                    }
                                }
                            }

                            ctx.run(|ctx| self.evaluate(&arguments[i - arguments_mod], index, false, ctx)).await
                        };

                        // definition type evaluation has to be performed in definition environment
                        let previous = Rc::clone(&self.environment);
                        self.environment = Rc::clone(&tmp_env);

                        let previous_name = self.curr_name.clone();
                        self.curr_name = curr_name.clone();

                        let def_evaluated = ctx.run(|ctx| self.evaluate(&params[i].type_, index, true, ctx)).await;

                        self.curr_name   = previous_name;
                        self.environment = previous;

                        let def_type = {
                            if matches!(def_evaluated.type_, SkyeType::Unknown(_)) {
                                SkyeType::Type(Box::new(def_evaluated.type_))
                            } else {
                                def_evaluated.type_
                            }
                        };

                        let is_self = i == 0 && arguments_mod == 1;

                        let new_call_evaluated = {
                            if is_self {
                                call_evaluated
                            } else if let SkyeType::Type(inner_type) = &def_type {
                                if let SkyeType::Pointer(.., is_reference) = **inner_type {
                                    if is_reference && !matches!(call_evaluated.type_, SkyeType::Pointer(..)) {
                                        // automatically create reference for pass-by-reference params
                                        let arg_pos = arguments[i - arguments_mod].get_pos();
                                        let custom_tok = Token::new(
                                            arg_pos.source, arg_pos.filename,
                                            TokenType::BitwiseAnd, Rc::from(""),
                                            arg_pos.start, arg_pos.end, arg_pos.line
                                        );

                                        let ref_expr = Expression::Unary { op: custom_tok, expr: Box::new(Expression::Grouping(
                                                Box::new(arguments[i - arguments_mod].clone())
                                            )), is_prefix: true };

                                        ctx.run(|ctx| self.evaluate(&ref_expr, index, allow_unknown, ctx)).await
                                    } else {
                                        call_evaluated
                                    }
                                } else {
                                    call_evaluated
                                }
                            } else {
                                call_evaluated
                            }
                        };

                        if !def_type.check_completeness() {
                            ast_error!(self, params[i].type_, "Cannot use incomplete type directly");
                            ast_note!(params[i].type_, "Define this type or reference it through a pointer");
                            ast_note!(expr, "This error is a result of template generation originating from this call");
                        }

                        if let SkyeType::Type(inner_type) = &def_type {
                            if inner_type.equals(&new_call_evaluated.type_, EqualsLevel::Permissive) {
                                if let Some(inferred) = inner_type.infer_type_from_similar(&new_call_evaluated.type_) {
                                    for (generic_name, generic_type) in inferred {
                                        if let Some(generic_to_find) = generics_to_find.get(&generic_name) {
                                            if generic_to_find.is_none() {
                                                if matches!(generic_type, SkyeType::Void) {
                                                    generics_to_find.insert(Rc::clone(&generic_name), Some(generic_type));
                                                } else {
                                                    generics_to_find.insert(Rc::clone(&generic_name), Some(SkyeType::Type(Box::new(generic_type))));
                                                }

                                                generics_found_at.insert(generic_name, i);
                                            }
                                        }
                                    }
                                } else {
                                    if i == 0 && arguments_mod == 1 {
                                        // the only way self info makes inference error is if method is not available for type
                                        if let Some((_, self_type)) = &callee.self_info {
                                            ast_error!(
                                                self, callee_expr,
                                                format!(
                                                    "This method cannot be called from type {}",
                                                    self_type.stringify_native()
                                                ).as_ref()
                                            );
                                        } else {
                                            unreachable!()
                                        }
                                    } else {
                                        ast_error!(
                                            self, arguments[i - arguments_mod],
                                            format!(
                                                "Argument type does not match parameter type (expecting {} but got {})",
                                                inner_type.stringify_native(), new_call_evaluated.type_.stringify_native()
                                            ).as_ref()
                                        );

                                        ast_note!(params[i].type_, "Parameter type defined here");
                                    }
                                }
                            } else {
                                if i == 0 && arguments_mod == 1 {
                                    // the only way self info is not equal to parameter type is if constness is not respected
                                    ast_error!(self, callee_expr, "This method cannot be called from a const source");
                                } else {
                                    ast_error!(
                                        self, arguments[i - arguments_mod],
                                        format!(
                                            "Argument type does not match parameter type (expecting {} but got {})",
                                            inner_type.stringify_native(), new_call_evaluated.type_.stringify_native()
                                        ).as_ref()
                                    );

                                    ast_note!(params[i].type_, "Parameter type defined here");
                                }
                            }
                        } else {
                            ast_error!(
                                self, params[i].type_,
                                format!(
                                    "Expecting type as parameter type (got {})",
                                    def_type.stringify_native()
                                ).as_ref()
                            );

                            ast_note!(expr, "This error is a result of template generation originating from this call");
                        }

                        let search_tok = Token::dummy(Rc::from("__copy__"));
                        if let Some(value) = self.get_method(&new_call_evaluated, &search_tok, true, index) {
                            let loc_callee_expr = {
                                if i != 0 || arguments_mod != 1 {
                                    &arguments[i - arguments_mod]
                                } else {
                                    callee_expr
                                }
                            };

                            let v = Vec::new();
                            let copy_constructor = ctx.run(|ctx| self.call(&value, expr, loc_callee_expr, &v, index, allow_unknown, ctx)).await;
                            args.push_str(&copy_constructor.value);

                            ast_info!(loc_callee_expr, "Skye inserted a copy constructor call for this expression"); // +I-copies
                        } else {
                            args.push_str(&new_call_evaluated.value);
                        }

                        if i != arguments_len - 1 {
                            args.push_str(", ");
                        }
                    }

                    for expr_generic in generics {
                        let generic_type = generics_to_find.get(&expr_generic.name.lexeme).unwrap();

                        let type_ = {
                            if let Some(t) = generic_type {
                                Some(t.clone())
                            } else if let Some(default) = &expr_generic.default {
                                let previous = Rc::clone(&self.environment);
                                self.environment = Rc::clone(&tmp_env);

                                let evaluated = ctx.run(|ctx| self.evaluate(&default, index, false, ctx)).await;

                                self.environment = previous;

                                if matches!(evaluated.type_, SkyeType::Type(_) | SkyeType::Void) {
                                    if evaluated.type_.check_completeness() {
                                        if evaluated.type_.can_be_instantiated(true) {
                                            Some(evaluated.type_)
                                        } else {
                                            ast_error!(self, default, format!("Cannot instantiate type {}", evaluated.type_.stringify_native()).as_ref());
                                            None
                                        }
                                    } else {
                                        ast_error!(self, default, "Cannot use incomplete type directly");
                                        ast_note!(default, "Define this type or reference it through a pointer");
                                        None
                                    }
                                } else {
                                    ast_error!(
                                        self, default,
                                        format!(
                                            "Expecting type as default generic (got {})",
                                            evaluated.type_.stringify_native()
                                        ).as_ref()
                                    );

                                    None
                                }
                            } else {
                                None
                            }
                        };

                        if let Some(inner_type) = type_ {
                            if let Some(bounds) = &expr_generic.bounds {
                                let previous = Rc::clone(&self.environment);
                                self.environment = Rc::clone(&tmp_env);

                                let evaluated = ctx.run(|ctx| self.evaluate(&bounds, index, false, ctx)).await;

                                self.environment = previous;

                                if evaluated.type_.is_type() || matches!(evaluated.type_, SkyeType::Void) {
                                    if evaluated.type_.is_respected_by(&inner_type) {
                                        let mut env = tmp_env.borrow_mut();
                                        env.define(
                                            Rc::clone(&expr_generic.name.lexeme),
                                            SkyeVariable::new(
                                                inner_type, true,
                                                Some(Box::new(expr_generic.name.clone()))
                                            )
                                        );
                                    } else {
                                        let at = *generics_found_at.get(&expr_generic.name.lexeme).unwrap();

                                        if at != 0 || arguments_mod != 1 {
                                            ast_error!(
                                                self, arguments[at - arguments_mod],
                                                format!(
                                                    "Generic bound is not respected by this type (expecting {} but got {})",
                                                    evaluated.type_.stringify_native(), inner_type.stringify_native()
                                                ).as_ref()
                                            );

                                            token_note!(expr_generic.name, "Generic defined here");
                                        }
                                    }
                                } else {
                                    ast_error!(
                                        self, bounds,
                                        format!(
                                            "Expecting type or group as generic bound (got {})",
                                            evaluated.type_.stringify_native()
                                        ).as_ref()
                                    );
                                }
                            } else {
                                let mut env = tmp_env.borrow_mut();
                                env.define(
                                    Rc::clone(&expr_generic.name.lexeme),
                                    SkyeVariable::new(
                                        inner_type, true,
                                        Some(Box::new(expr_generic.name.clone()))
                                    )
                                );
                            }
                        } else {
                            if self.errors == 0 { // avoids having inference errors caused by other errors
                                ast_error!(self, callee_expr, "Skye cannot infer the generic types for this function");
                                ast_note!(callee_expr, "This expression is a template and requires generic typing");
                                ast_note!(callee_expr, "Manually specify the generic types");
                            }

                            return SkyeValue::get_unknown();
                        }
                    }

                    let previous = Rc::clone(&self.environment);
                    self.environment = Rc::clone(&tmp_env);

                    let previous_name = self.curr_name.clone();
                    self.curr_name = curr_name.clone();

                    let return_evaluated = {
                        let ret_type = ctx.run(|ctx| self.evaluate(&return_type_expr, index, false, ctx)).await.type_;

                        match ret_type {
                            SkyeType::Type(inner_type) => {
                                if !inner_type.check_completeness() {
                                    ast_error!(self, return_type_expr, "Cannot use incomplete type directly");
                                    ast_note!(return_type_expr, "Define this type or reference it through a pointer");
                                    ast_note!(expr, "This error is a result of template generation originating from this call");
                                }

                                if !inner_type.can_be_instantiated(false) {
                                    ast_error!(self, return_type_expr, format!("Cannot instantiate type {}", inner_type.stringify_native()).as_ref());
                                }

                                *inner_type.clone()
                            }
                            SkyeType::Void => ret_type,
                            _ => {
                                ast_error!(
                                    self, return_type_expr,
                                    format!(
                                        "Expecting type as return type (got {})",
                                        ret_type.stringify_native()
                                    ).as_ref()
                                );

                                ast_note!(expr, "This error is a result of template generation originating from this call");
                                SkyeType::get_unknown()
                            }
                        }
                    };

                    let final_name = self.get_generics(&name, &generics_names, &self.environment);

                    let search_tok = Token::dummy(Rc::clone(&final_name));

                    let mut env = self.globals.borrow_mut();
                    if let Some(existing) = env.get(&search_tok) {
                        if let SkyeType::Function(.., has_body) = existing.type_ {
                            if has_body {
                                env = self.environment.borrow_mut();
                                for generic in generics {
                                    env.undef(Rc::clone(&generic.name.lexeme));
                                }

                                drop(env);
                                self.curr_name   = previous_name;
                                self.environment = previous;

                                let call_output = self.output_call(&return_evaluated, &final_name, &args, index);
                                return SkyeValue::new(Rc::from(call_output.as_ref()), return_evaluated, true);
                            }
                        } else {
                            if let Some(tok) = existing.tok {
                                ast_error!(self, callee_expr, "Template generation for this call resulted in an invalid type");
                                token_note!(tok, "This definition is invalid. Change the name of this symbol");
                            } else {
                                ast_error!(self, callee_expr, "Template generation for this call resulted in an invalid type. An invalid symbol definition is present in the code");
                            }
                        }
                    }

                    drop(env);

                    let old_errors = self.errors;

                    let type_ = {
                        match ctx.run(|ctx| self.execute(&definition, 0, ctx)).await {
                            Ok(item) => item.unwrap_or_else(|| {
                                ast_error!(self, expr, "Could not process template generation for this expression");
                                SkyeType::get_unknown()
                            }),
                            Err(_) => unreachable!("execution interrupt happened out of context")
                        }
                    };

                    if self.errors != old_errors {
                        ast_note!(expr, "This error is a result of template generation originating from this call");
                    }

                    self.curr_name   = previous_name;
                    self.environment = previous;

                    env = tmp_env.borrow_mut();
                    for generic in generics {
                        env.undef(Rc::clone(&generic.name.lexeme));
                    }

                    env.define(
                        Rc::clone(&final_name),
                        SkyeVariable::new(
                            type_.clone(), true,
                            None
                        )
                    );

                    let call_output = self.output_call(&return_evaluated, &final_name, &args, index);
                    SkyeValue::new(Rc::from(call_output.as_ref()), return_evaluated, false)
                } else {
                    ast_error!(self, callee_expr, "Cannot call this expression");
                    ast_note!(
                        callee_expr,
                        format!(
                            "This expression has type {}",
                            callee.type_.stringify_native()
                        ).as_ref()
                    );

                    SkyeValue::get_unknown()
                }
            }
            SkyeType::Macro(macro_name, params_opt, body) => {
                assert!(!matches!(params_opt, MacroParams::None)); // covered by unary '@' evaluation

                match params_opt {
                    MacroParams::Some(params) => {
                        if params.len() != arguments_len {
                            ast_error!(
                                self, expr,
                                format!(
                                    "Expecting {} arguments for macro call but got {}",
                                    params.len(), arguments_len
                                ).as_str()
                            );

                            return SkyeValue::get_unknown();
                        }
                    }
                    MacroParams::OneN(_) => {
                        if arguments_len == 0 {
                            ast_error!(self, expr, "Expecting at least one argument for macro call but got none");
                            return SkyeValue::get_unknown();
                        }
                    }
                    _ => ()
                }

                if let MacroBody::Binding(return_type) = body {
                    let tmp_env = Rc::new(RefCell::new(Environment::with_enclosing(Rc::clone(&self.environment))));
                    let mut env = tmp_env.borrow_mut();

                    let mut args = String::new();
                    for i in 0 .. arguments_len {
                        let arg = ctx.run(|ctx| self.evaluate(&arguments[i], index, allow_unknown, ctx)).await;

                        if let SkyeType::Type(inner_type) = &arg.type_ {
                            args.push_str(&inner_type.stringify());
                        } else {
                            args.push_str(&arg.value);
                        }

                        if let MacroParams::Some(params) = params_opt {
                            env.define(
                                Rc::clone(&params[i].lexeme),
                                SkyeVariable::new(
                                    arg.type_, true,
                                    Some(Box::new(params[i].clone()))
                                )
                            );
                        }

                        if i != arguments_len - 1 {
                            args.push_str(", ");
                        }
                    }

                    drop(env);
                    let previous = Rc::clone(&self.environment);
                    self.environment = tmp_env;

                    let call_return_type = ctx.run(|ctx| self.evaluate(&return_type, index, allow_unknown, ctx)).await;

                    self.environment = previous;

                    if let SkyeType::Type(inner_type) = call_return_type.type_ {
                        SkyeValue::new(Rc::from(format!("{}({})", callee.value, args)), *inner_type, false)
                    } else {
                        ast_error!(
                            self, return_type,
                            format!(
                                "Expecting type as return type (got {})",
                                call_return_type.type_.stringify_native()
                            ).as_ref()
                        );
                        ast_note!(expr, "This error is a result of this macro expansion");
                        SkyeValue::get_unknown()
                    }
                } else if let Some(result) = ctx.run(|ctx| self.handle_builtin_macros(macro_name, arguments, index, allow_unknown, callee_expr, ctx)).await {
                    return result;
                } else if let MacroBody::Expression(return_expr) = body {
                    if macro_name.as_ref() == "panic" {
                        // macros should be handled at macro expansion time,
                        // but the panic macro sometimes gets generated by the codegen step itself.
                        // so handle it here too, to cover those cases

                        if let MacroParams::Some(params) = params_opt {
                            let mut curr_expr = return_expr.clone();

                            for i in 0 .. arguments_len {
                                curr_expr = curr_expr.replace_variable(&params[i].lexeme, &arguments[i]);
                            }

                            if matches!(self.compile_mode, CompileMode::Debug) {
                                let panic_pos = callee_expr.get_pos();

                                curr_expr = curr_expr.replace_variable(
                                    &Rc::from("PANIC_POS"),
                                    &Expression::Literal { value: Rc::from(format!(
                                        "{}: line {}, pos {}",
                                        escape_string(&panic_pos.filename), panic_pos.line + 1, panic_pos.start
                                    )), tok: Token::dummy(Rc::from("")), kind: LiteralKind::String }
                                );
                            } else {
                                curr_expr = curr_expr.replace_variable(
                                    &Rc::from("PANIC_POS"),
                                    &Expression::Literal { value: Rc::from(""), tok: Token::dummy(Rc::from("")), kind: LiteralKind::String }
                                );
                            }

                            let old_errors = self.errors;

                            let res = ctx.run(|ctx| self.evaluate(&curr_expr, index, allow_unknown, ctx)).await;

                            if self.errors != old_errors {
                                ast_note!(expr, "This error is a result of this macro expansion");
                            }

                            res
                        } else {
                            unreachable!("@panic call in codegen did not have MacroParams::Some");
                        }
                    } else {
                        ast_error!(self, expr, "Macro call is not allowed here");
                        SkyeValue::get_unknown()
                    }
                } else {
                    ast_error!(self, expr, "Macro call is not allowed here");
                    SkyeValue::get_unknown()
                }
            }
            _ => {
                ast_error!(self, callee_expr, "Cannot call this expression");
                ast_note!(
                    callee_expr,
                    format!(
                        "This expression has type {}",
                        callee.type_.stringify_native()
                    ).as_ref()
                );

                SkyeValue::get_unknown()
            }
        }
    }

    async fn pre_eval_unary_operator(
        &mut self, inner: SkyeValue, inner_expr: &Expression,
        expr: &Expression, op_stringified: &str, op_method: &str,
        op_type: Operator, op: &Token, index: usize, allow_unknown: bool,
        ctx: &mut reblessive::Stk
    ) -> SkyeValue {
        match inner.type_.implements_op(op_type) {
            ImplementsHow::Native(_) => {
                let tmp_var = self.get_temporary_var();

                self.definitions[index].push_indent();
                self.definitions[index].push(&inner.type_.stringify());
                self.definitions[index].push(" ");
                self.definitions[index].push(&tmp_var);
                self.definitions[index].push(" = ");
                self.definitions[index].push(op_stringified);
                self.definitions[index].push(&inner.value);
                self.definitions[index].push(";\n");

                SkyeValue::new(Rc::from(tmp_var), inner.type_, false)
            }
            ImplementsHow::ThirdParty => {
                let search_tok = Token::dummy(Rc::from(op_method));
                if let Some(value) = self.get_method(&inner, &search_tok, true, index) {
                    let v = Vec::new();
                    let _ = ctx.run(|ctx| self.call(&value, expr, inner_expr, &v, index, allow_unknown, ctx)).await;
                    inner
                } else {
                    token_error!(
                        self, op,
                        format!(
                            "Prefix unary '{}' operator is not implemented for type {}",
                            op_stringified, inner.type_.stringify_native()
                        ).as_ref()
                    );

                    SkyeValue::get_unknown()
                }
            }
            ImplementsHow::No => {
                token_error!(
                    self, op,
                    format!(
                        "Type {} cannot use prefix unary '{}' operator",
                        inner.type_.stringify_native(), op_stringified
                    ).as_ref()
                );

                SkyeValue::get_unknown()
            }
        }
    }

    async fn post_eval_unary_operator(
        &mut self, inner: SkyeValue, inner_expr: &Expression,
        expr: &Expression, op_stringified: &str, op_method: &str,
        op_type: Operator, op: &Token, index: usize, allow_unknown: bool,
        ctx: &mut reblessive::Stk
    ) -> SkyeValue {
        let tmp_var = self.get_temporary_var();

        self.definitions[index].push_indent();
        self.definitions[index].push(&inner.type_.stringify());
        self.definitions[index].push(" ");
        self.definitions[index].push(&tmp_var);
        self.definitions[index].push(" = ");
        self.definitions[index].push(&inner.value);
        self.definitions[index].push(";\n");

        match inner.type_.implements_op(op_type) {
            ImplementsHow::Native(_) => {
                self.definitions[index].push_indent();
                self.definitions[index].push(&inner.value);
                self.definitions[index].push(op_stringified);
                self.definitions[index].push(";\n");

                SkyeValue::new(Rc::from(tmp_var), inner.type_, false)
            }
            ImplementsHow::ThirdParty => {
                let search_tok = Token::dummy(Rc::from(op_method));
                if let Some(value) = self.get_method(&inner, &search_tok, true, index) {
                    let v = Vec::new();
                    let _ = ctx.run(|ctx| self.call(&value, expr, inner_expr, &v, index, allow_unknown, ctx)).await;
                    SkyeValue::new(Rc::from(tmp_var), inner.type_, false)
                } else {
                    token_error!(
                        self, op,
                        format!(
                            "postfix unary '{}' operator is not implemented for type {}",
                            op_stringified, inner.type_.stringify_native()
                        ).as_ref()
                    );

                    SkyeValue::get_unknown()
                }
            }
            ImplementsHow::No => {
                token_error!(
                    self, op,
                    format!(
                        "Type {} cannot use postfix unary '{}' operator",
                        inner.type_.stringify_native(), op_stringified
                    ).as_ref()
                );

                SkyeValue::get_unknown()
            }
        }
    }

    async fn unary_operator(
        &mut self, inner: SkyeValue, inner_expr: &Expression,
        expr: &Expression, op_stringified: &str, op_method: &str,
        op_type: Operator, op: &Token, index: usize, allow_unknown: bool,
        ctx: &mut reblessive::Stk
    ) -> SkyeValue {
        let new_inner = inner.follow_reference(self.external_zero_check(op, index));

        match new_inner.type_.implements_op(op_type) {
            ImplementsHow::Native(_) => SkyeValue::new(Rc::from(format!("{}{}", op_stringified, new_inner.value)), new_inner.type_, false),
            ImplementsHow::ThirdParty => {
                let search_tok = Token::dummy(Rc::from(op_method));
                if let Some(value) = self.get_method(&new_inner, &search_tok, true, index) {
                    let v = Vec::new();
                    ctx.run(|ctx| self.call(&value, expr, inner_expr, &v, index, allow_unknown, ctx)).await
                } else {
                    token_error!(
                        self, op,
                        format!(
                            "Prefix unary '{}' operator is not implemented for type {}",
                            op_stringified, new_inner.type_.stringify_native()
                        ).as_ref()
                    );

                    SkyeValue::get_unknown()
                }
            }
            ImplementsHow::No => {
                token_error!(
                    self, op,
                    format!(
                        "Type {} cannot use prefix unary '{}' operator",
                        new_inner.type_.stringify_native(), op_stringified
                    ).as_ref()
                );

                SkyeValue::get_unknown()
            }
        }
    }

    async fn binary_operator(
        &mut self, left: SkyeValue, forced_return_type: Option<SkyeType>,
        left_expr: &Expression, right_expr: &Expression, expr: &Expression,
        op_stringified: &str, op: &Token, op_method: &str, op_type: Operator,
        index: usize, allow_unknown: bool, ctx: &mut reblessive::Stk
    ) -> SkyeValue {
        let new_left = left.follow_reference(self.external_zero_check(op, index));

        match new_left.type_.implements_op(op_type) {
            ImplementsHow::Native(compatible_types) => {
                let right = ctx.run(|ctx| self.evaluate(&right_expr, index, allow_unknown, ctx)).await.follow_reference(self.external_zero_check(op, index));

                if matches!(new_left.type_, SkyeType::Unknown(_)) ||
                    new_left.type_.equals(&right.type_, EqualsLevel::Typewise) ||
                    compatible_types.contains(&right.type_)
                {
                    if let Some(type_) = forced_return_type {
                        SkyeValue::new(Rc::from(format!("{} {} {}", new_left.value, op_stringified, right.value)), type_, false)
                    } else {
                        SkyeValue::new(Rc::from(format!("{} {} {}", new_left.value, op_stringified, right.value)), new_left.type_, false)
                    }
                } else {
                    ast_error!(
                        self, right_expr,
                        format!(
                            "Left operand type ({}) does not match right operand type ({})",
                            new_left.type_.stringify_native(), right.type_.stringify_native()
                        ).as_ref()
                    );

                    SkyeValue::get_unknown()
                }
            }
            ImplementsHow::ThirdParty => {
                let search_tok = Token::dummy(Rc::from(op_method));
                if let Some(value) = self.get_method(&new_left, &search_tok, true, index) {
                    let args = vec![right_expr.clone()];
                    ctx.run(|ctx| self.call(&value, expr, left_expr, &args, index, allow_unknown, ctx)).await
                } else {
                    ast_error!(
                        self, left_expr,
                        format!(
                            "Binary '{}' operator is not implemented for type {}",
                            op_stringified, new_left.type_.stringify_native()
                        ).as_ref()
                    );

                    SkyeValue::get_unknown()
                }
            }
            ImplementsHow::No => {
                ast_error!(
                    self, left_expr,
                    format!(
                        "Type {} cannot use binary '{}' operator",
                        new_left.type_.stringify_native(), op_stringified
                    ).as_ref()
                );

                SkyeValue::get_unknown()
            }
        }
    }

    async fn binary_operator_int_on_right(
        &mut self, left: SkyeValue, left_expr: &Expression,
        right_expr: &Expression, expr: &Expression, op_stringified: &str,
        op: &Token, op_method: &str, op_type: Operator,
        index: usize, allow_unknown: bool, ctx: &mut reblessive::Stk
    ) -> SkyeValue {
        let new_left = left.follow_reference(self.external_zero_check(op, index));

        match new_left.type_.implements_op(op_type) {
            ImplementsHow::Native(_) => {
                let right = ctx.run(|ctx| self.evaluate(&right_expr, index, allow_unknown, ctx)).await
                    .follow_reference(self.external_zero_check(op, index));

                if right.type_.equals(&SkyeType::AnyInt, EqualsLevel::Typewise) {
                    SkyeValue::new(Rc::from(format!("{} {} {}", new_left.value, op_stringified, right.value)), new_left.type_, false)
                } else {
                    ast_error!(
                        self, right_expr,
                        format!(
                            "Expecting right operand type to be integer but got {}",
                            right.type_.stringify_native()
                        ).as_ref()
                    );

                    SkyeValue::get_unknown()
                }
            }
            ImplementsHow::ThirdParty => {
                let search_tok = Token::dummy(Rc::from(op_method));
                if let Some(value) = self.get_method(&new_left, &search_tok, true, index) {
                    let args = vec![right_expr.clone()];
                    ctx.run(|ctx| self.call(&value, expr, left_expr, &args, index, allow_unknown, ctx)).await
                } else {
                    ast_error!(
                        self, left_expr,
                        format!(
                            "Binary '{}' operator is not implemented for type {}",
                            op_stringified, new_left.type_.stringify_native()
                        ).as_ref()
                    );

                    SkyeValue::get_unknown()
                }
            }
            ImplementsHow::No => {
                ast_error!(
                    self, left_expr,
                    format!(
                        "Type {} cannot use binary '{}' operator",
                        new_left.type_.stringify_native(), op_stringified
                    ).as_ref()
                );

                SkyeValue::get_unknown()
            }
        }
    }

    async fn zero_check(&mut self, value: &SkyeValue, tok: &Token, msg: &str, index: usize, ctx: &mut reblessive::Stk) -> Rc<str> {
        if matches!(self.compile_mode, CompileMode::Debug) {
            let tmp_var = self.get_temporary_var();

            self.definitions[index].push_indent();
            self.definitions[index].push(&value.type_.stringify());
            self.definitions[index].push(" ");
            self.definitions[index].push(&tmp_var);
            self.definitions[index].push(" = ");
            self.definitions[index].push(&value.value);
            self.definitions[index].push(";\n");

            self.definitions[index].push_indent();
            self.definitions[index].push("if (");
            self.definitions[index].push(&tmp_var);
            self.definitions[index].push(" == 0) {\n");
            self.definitions[index].inc_indent();

            let mut at_tok = tok.clone();
            at_tok.set_type(TokenType::At);

            let mut panic_tok = tok.clone();
            panic_tok.set_lexeme("panic");

            let panic_stmt = Statement::Expression(Expression::Call(
                Box::new(Expression::Unary { op: at_tok, expr: Box::new(Expression::Variable(panic_tok)), is_prefix: true }),
                tok.clone(),
                vec![Expression::Literal { value: Rc::from(msg), tok: tok.clone(), kind: LiteralKind::String }]
            ));

            let _ = ctx.run(|ctx| self.execute(&panic_stmt, index, ctx)).await;

            self.definitions[index].dec_indent();
            self.definitions[index].push_indent();
            self.definitions[index].push("}\n");

            Rc::from(tmp_var.as_ref())
        } else {
            value.value.clone()
        }
    }

    fn external_zero_check<'a>(&'a mut self, tok: &'a Token, index: usize) -> Box<impl FnMut(SkyeValue) -> Rc<str> + 'a> {
        Box::new(move |value| {
            let mut stack = reblessive::Stack::new();
            stack.enter(|ctx| self.zero_check(&value, tok, "Null pointer dereference", index, ctx)).finish()
        })
    }

    async fn binary_operator_with_zero_check(
        &mut self, left: SkyeValue, forced_return_type: Option<SkyeType>,
        left_expr: &Expression, right_expr: &Expression, expr: &Expression,
        op_stringified: &str, op: &Token, op_method: &str, op_type: Operator,
        index: usize, allow_unknown: bool, ctx: &mut reblessive::Stk
    ) -> SkyeValue {
        let new_left = left.follow_reference(self.external_zero_check(op, index));

        match new_left.type_.implements_op(op_type) {
            ImplementsHow::Native(compatible_types) => {
                let right = ctx.run(|ctx| self.evaluate(&right_expr, index, allow_unknown, ctx)).await
                    .follow_reference(self.external_zero_check(op, index));

                if matches!(new_left.type_, SkyeType::Unknown(_)) ||
                    new_left.type_.equals(&right.type_, EqualsLevel::Typewise) ||
                    compatible_types.contains(&right.type_)
                {
                    let right_value = ctx.run(|ctx| self.zero_check(&right, op, "Division by zero", index, ctx)).await;

                    if let Some(type_) = forced_return_type {
                        SkyeValue::new(Rc::from(format!("{} {} {}", new_left.value, op_stringified, right_value)), type_, false)
                    } else {
                        SkyeValue::new(Rc::from(format!("{} {} {}", new_left.value, op_stringified, right_value)), new_left.type_, false)
                    }
                } else {
                    ast_error!(
                        self, right_expr,
                        format!(
                            "Left operand type ({}) does not match right operand type ({})",
                            new_left.type_.stringify_native(), right.type_.stringify_native()
                        ).as_ref()
                    );

                    SkyeValue::get_unknown()
                }
            }
            ImplementsHow::ThirdParty => {
                let search_tok = Token::dummy(Rc::from(op_method));
                if let Some(value) = self.get_method(&new_left, &search_tok, true, index) {
                    let args = vec![right_expr.clone()];
                    ctx.run(|ctx| self.call(&value, expr, left_expr, &args, index, allow_unknown, ctx)).await
                } else {
                    ast_error!(
                        self, left_expr,
                        format!(
                            "Binary '{}' operator is not implemented for type {}",
                            op_stringified, new_left.type_.stringify_native()
                        ).as_ref()
                    );

                    SkyeValue::get_unknown()
                }
            }
            ImplementsHow::No => {
                ast_error!(
                    self, left_expr,
                    format!(
                        "Type {} cannot use binary '{}' operator",
                        new_left.type_.stringify_native(), op_stringified
                    ).as_ref()
                );

                SkyeValue::get_unknown()
            }
        }
    }

    async fn get_type_equality(&mut self, inner_left: &SkyeType, right_expr: &Expression, index: usize, allow_unknown: bool, reversed: bool, ctx: &mut reblessive::Stk) -> SkyeValue {
        let right = ctx.run(|ctx| self.evaluate(&right_expr, index, allow_unknown, ctx)).await;

        if let SkyeType::Type(inner_right) = right.type_ {
            if reversed ^ inner_left.equals(&inner_right, EqualsLevel::Typewise) {
                SkyeValue::new(Rc::from("UINT8_C(1)"), SkyeType::U8, true)
            } else {
                SkyeValue::new(Rc::from("UINT8_C(0)"), SkyeType::U8, true)
            }
        } else {
            ast_error!(
                self, right_expr,
                format!(
                    "Left operand type does not match right operand type (expecting type on right operand but got {})",
                    right.type_.stringify_native()
                ).as_ref()
            );

            SkyeValue::get_unknown()
        }
    }

    async fn evaluate(&mut self, expr: &Expression, index: usize, allow_unknown: bool, ctx: &mut reblessive::Stk) -> SkyeValue {
        match expr {
            Expression::Grouping(inner_expr) => {
                let inner = ctx.run(|ctx| self.evaluate(&inner_expr, index, allow_unknown, ctx)).await;
                SkyeValue::new(Rc::from(format!("({})", inner.value)), inner.type_, inner.is_const)
            }
            Expression::InMacro { inner: inner_expr, source } => {
                let old_errors = self.errors;
                let inner = ctx.run(|ctx| self.evaluate(&inner_expr, index, allow_unknown, ctx)).await;

                if self.errors != old_errors {
                    astpos_note!(source, "This error is a result of this macro expansion");
                }

                inner
            }
            Expression::MacroExpandedStatements { inner, source } => {
                let old_errors = self.errors;

                for statement in inner {
                    let _ = ctx.run(|ctx| self.execute(statement, index, ctx)).await;
                }

                if self.errors != old_errors {
                    astpos_note!(source, "This error is a result of this macro expansion");
                }

                SkyeValue::special(SkyeType::Void)
            }
            Expression::Slice { opening_brace, items } => {
                let first_item = ctx.run(|ctx| self.evaluate(&items[0], index, allow_unknown, ctx)).await;
                let mut items_stringified = String::from("{");
                items_stringified.push_str(&first_item.value);

                if items.len() != 1 {
                    items_stringified.push_str(", ");
                }

                for i in 1 .. items.len() {
                    let evaluated = ctx.run(|ctx| self.evaluate(&items[i], index, allow_unknown, ctx)).await;

                    if !evaluated.type_.equals(&first_item.type_, EqualsLevel::Typewise) {
                        ast_error!(
                            self, items[i],
                            format!(
                                "Items inside array do not have matching types (expecting {} but got {})",
                                first_item.type_.stringify_native(), evaluated.type_.stringify_native()
                            ).as_ref()
                        );
                        ast_note!(items[0], "First item defined here");
                    }

                    items_stringified.push_str(&evaluated.value);

                    if i != items.len() - 1 {
                        items_stringified.push_str(", ");
                    }
                }

                items_stringified.push('}');

                let mut slice_tok = opening_brace.clone();
                slice_tok.set_lexeme("core_DOT_Slice");

                let tmp_var = Rc::from(self.get_temporary_var());

                let mut type_tok = opening_brace.clone();
                type_tok.set_lexeme(&tmp_var);

                let mut env = self.environment.borrow_mut();
                env.define(
                    Rc::clone(&tmp_var),
                    SkyeVariable::new(
                        SkyeType::Type(Box::new(first_item.type_.clone())),
                        true,
                        Some(Box::new(type_tok.clone()))
                    )
                );

                drop(env);

                let subscript_expr = Expression::Subscript { subscripted: Box::new(Expression::Variable(slice_tok)), paren: opening_brace.clone(), args: vec![Expression::Variable(type_tok)] };

                let return_type = ctx.run(|ctx| self.evaluate(&subscript_expr, index, allow_unknown, ctx)).await;

                let mut env = self.environment.borrow_mut();
                env.undef(tmp_var);

                if let SkyeType::Type(inner_type) = return_type.type_ {
                    SkyeValue::new(
                        Rc::from(format!(
                            "({}) {{ .ptr = ({}[]) {}, .length = {} }}",
                            return_type.value,
                            first_item.type_.stringify(),
                            items_stringified,
                            items.len()
                        )),
                        *inner_type,
                        true
                    )
                } else {
                    panic!("struct template generation resulted in not a type");
                }
            }
            Expression::Literal { value, kind, .. } => {
                match kind {
                    LiteralKind::Void => SkyeValue::special(SkyeType::Void),

                    LiteralKind::U8  => SkyeValue::new(Rc::from(format!("UINT8_C({})",  value)), SkyeType::U8,  true),
                    LiteralKind::I8  => SkyeValue::new(Rc::from(format!("INT8_C({})",   value)), SkyeType::I8,  true),
                    LiteralKind::U16 => SkyeValue::new(Rc::from(format!("UINT16_C({})", value)), SkyeType::U16, true),
                    LiteralKind::I16 => SkyeValue::new(Rc::from(format!("INT16_C({})",  value)), SkyeType::I16, true),
                    LiteralKind::U32 => SkyeValue::new(Rc::from(format!("UINT32_C({})", value)), SkyeType::U32, true),
                    LiteralKind::I32 => SkyeValue::new(Rc::from(format!("INT32_C({})",  value)), SkyeType::I32, true),
                    LiteralKind::U64 => SkyeValue::new(Rc::from(format!("UINT64_C({})", value)), SkyeType::U64, true),
                    LiteralKind::I64 => SkyeValue::new(Rc::from(format!("INT64_C({})",  value)), SkyeType::I64, true),
                    LiteralKind::Usz => SkyeValue::new(Rc::from(format!("SIZE_T_C({})", value)), SkyeType::Usz, true),
                    LiteralKind::F32 => SkyeValue::new(Rc::from(format!("(float){}",    value)), SkyeType::F32, true),
                    LiteralKind::F64 => SkyeValue::new(Rc::from(format!("(double){}",   value)), SkyeType::F64, true),

                    LiteralKind::AnyInt   => SkyeValue::new(Rc::clone(value), SkyeType::AnyInt, true),
                    LiteralKind::AnyFloat => SkyeValue::new(Rc::clone(value), SkyeType::AnyFloat, true),

                    LiteralKind::Char => SkyeValue::new(Rc::from(format!("'{}'", value)), SkyeType::Char, true),
                    LiteralKind::RawString => {
                        if let Some(string_const) = self.strings.get(value) {
                            SkyeValue::new(Rc::from(format!("SKYE_STRING_{}", string_const)), SkyeType::Pointer(Box::new(SkyeType::Char), true, false), true)
                        } else {
                            let str_index = self.strings.len();
                            self.strings_code.push(format!(
                                "const char SKYE_STRING_{}[{}] = \"{}\";\n",
                                str_index, get_real_string_length(value), fix_raw_string(value)
                            ).as_ref());

                            self.strings.insert(Rc::clone(value), str_index);
                            SkyeValue::new(Rc::from(format!("SKYE_STRING_{}", str_index)), SkyeType::Pointer(Box::new(SkyeType::Char), true, false), true)
                        }
                    }
                    LiteralKind::String => {
                        if self.string_type.is_none() {
                            if let SkyeType::Type(inner_type) = &self.globals.borrow().get(
                                &Token::dummy(Rc::from("String"))
                            ).as_ref().expect("No String type is defined yet").type_
                            {
                                self.string_type = Some(*inner_type.clone());
                            } else {
                                panic!("The default String type was overwritten with an invalid type");
                            }
                        }

                        if let Some(string_const) = self.strings.get(value) {
                            SkyeValue::new(
                                Rc::from(format!(
                                    "(String) {{ .ptr = SKYE_STRING_{}, .length = sizeof(SKYE_STRING_{}) }}",
                                    string_const, string_const
                                )),
                                self.string_type.as_ref().unwrap().clone(),
                                true
                            )
                        } else {
                            let str_index = self.strings.len();
                            let string_len = get_real_string_length(value);
                            self.strings_code.push(format!(
                                "const char SKYE_STRING_{}[{}] = \"{}\";\n",
                                str_index, string_len, value
                            ).as_ref());

                            self.strings.insert(Rc::clone(value), str_index);

                            SkyeValue::new(
                                Rc::from(format!(
                                    "(String) {{ .ptr = SKYE_STRING_{}, .length = {} }}",
                                    str_index, string_len
                                )),
                                self.string_type.as_ref().unwrap().clone(),
                                true
                            )
                        }
                    }
                }
            }
            Expression::Unary { op, expr: inner_expr, is_prefix } => {
                let inner = ctx.run(|ctx| self.evaluate(&inner_expr, index, allow_unknown, ctx)).await;

                if *is_prefix {
                    match op.type_ {
                        TokenType::PlusPlus => {
                            let new_inner = inner.follow_reference(self.external_zero_check(op, index));

                            if new_inner.is_const {
                                ast_error!(self, inner_expr, "Cannot apply '++' operator on const value");
                                new_inner
                            } else if !inner_expr.is_valid_assignment_target() {
                                ast_error!(self, inner_expr, "Can only apply '++' operator on valid assignment targets");
                                new_inner
                            } else {
                                ctx.run(|ctx| self.pre_eval_unary_operator(
                                    new_inner, inner_expr, expr, "++",
                                    "__inc__", Operator::Inc, op, index, allow_unknown, ctx
                                )).await
                            }
                        }
                        TokenType::MinusMinus => {
                            let new_inner = inner.follow_reference(self.external_zero_check(op, index));

                            if new_inner.is_const {
                                ast_error!(self, inner_expr, "Cannot apply '--' operator on const value");
                                new_inner
                            } else if !inner_expr.is_valid_assignment_target() {
                                ast_error!(self, inner_expr, "Can only apply '--' operator on valid assignment targets");
                                new_inner
                            } else {
                                ctx.run(|ctx| self.pre_eval_unary_operator(
                                    new_inner, inner_expr, expr, "--",
                                    "__dec__", Operator::Dec, op, index, allow_unknown, ctx
                                )).await
                            }
                        }
                        TokenType::Plus => {
                            ctx.run(|ctx| self.unary_operator(
                                inner, inner_expr, expr, "+", "__pos__", Operator::Pos, op, index, allow_unknown, ctx
                            )).await
                        }
                        TokenType::Minus => {
                            ctx.run(|ctx| self.unary_operator(
                                inner, inner_expr, expr, "-", "__neg__", Operator::Neg, op, index, allow_unknown, ctx
                            )).await
                        }
                        TokenType::Bang => {
                            if matches!(inner.type_, SkyeType::Type(_) | SkyeType::Void | SkyeType::Unknown(_)) {
                                // !type syntax for void!type (result operator)

                                if !inner.type_.check_completeness() {
                                    ast_error!(self, inner_expr, "Cannot use incomplete type directly");
                                    ast_note!(inner_expr, "Define this type or reference it through a pointer");
                                }

                                if !inner.type_.can_be_instantiated(true) {
                                    ast_error!(self, inner_expr, format!("Cannot instantiate type {}", inner.type_.stringify_native()).as_ref());
                                }

                                let mut custom_token = op.clone();
                                custom_token.set_lexeme("core_DOT_Result");

                                let subscript_expr = Expression::Subscript { subscripted: Box::new(Expression::Variable(custom_token)), paren: op.clone(), args: vec![
                                        Expression::Literal { value: Rc::from(""), tok: op.clone(), kind: LiteralKind::Void },
                                        *inner_expr.clone()
                                    ] };

                                ctx.run(|ctx| self.evaluate(&subscript_expr, index, allow_unknown, ctx)).await
                            } else {
                                ctx.run(|ctx| self.unary_operator(
                                    inner, inner_expr, expr, "!", "__not__", Operator::Not, op, index, allow_unknown, ctx
                                )).await
                            }
                        }
                        TokenType::Question => {
                            if matches!(inner.type_, SkyeType::Type(_) | SkyeType::Void | SkyeType::Unknown(_)) {
                                // option operator

                                if !inner.type_.check_completeness() {
                                    ast_error!(self, inner_expr, "Cannot use incomplete type directly");
                                    ast_note!(inner_expr, "Define this type or reference it through a pointer");
                                }

                                if !inner.type_.can_be_instantiated(true) {
                                    ast_error!(self, inner_expr, format!("Cannot instantiate type {}", inner.type_.stringify_native()).as_ref());
                                }

                                let mut custom_token = op.clone();
                                custom_token.set_lexeme("core_DOT_Option");

                                let subscript_expr = Expression::Subscript { subscripted: Box::new(Expression::Variable(custom_token)), paren: op.clone(), args: vec![*inner_expr.clone()] };

                                ctx.run(|ctx| self.evaluate(&subscript_expr, index, allow_unknown, ctx)).await
                            } else {
                                ast_error!(
                                    self, inner_expr,
                                    format!(
                                        "Invalid operand for option operator (expecting type but got {})",
                                        inner.type_.stringify_native()
                                    ).as_ref()
                                );

                                SkyeValue::get_unknown()
                            }
                        }
                        TokenType::Tilde => {
                            ctx.run(|ctx| self.unary_operator(
                                inner, inner_expr, expr, "~", "__inv__", Operator::Inv, op, index, allow_unknown, ctx
                            )).await
                        }
                        TokenType::BitwiseAnd => {
                            match inner.type_ {
                                SkyeType::Type(type_type) => {
                                    SkyeValue::new(
                                        Rc::from(format!("{}*", inner.value)),
                                        SkyeType::Type(Box::new(SkyeType::Pointer(type_type, false, true))),
                                        true
                                    )
                                }
                                SkyeType::Unknown(_) => {
                                    SkyeValue::special(SkyeType::Type(Box::new(SkyeType::Pointer(Box::new(inner.type_), false, true))))
                                }
                                _ => {
                                    let new_inner = inner.follow_reference(self.external_zero_check(op, index));

                                    match new_inner.type_.implements_op(Operator::Ref) {
                                        ImplementsHow::Native(_) | ImplementsHow::ThirdParty => {
                                            let value = {
                                                if inner_expr.is_valid_assignment_target() {
                                                    new_inner.value
                                                } else {
                                                    let tmp_var = self.get_temporary_var();

                                                    self.definitions[index].push_indent();
                                                    self.definitions[index].push(&new_inner.type_.stringify());
                                                    self.definitions[index].push(" ");
                                                    self.definitions[index].push(&tmp_var);
                                                    self.definitions[index].push(" = ");
                                                    self.definitions[index].push(&new_inner.value);
                                                    self.definitions[index].push(";\n");

                                                    Rc::from(tmp_var)
                                                }
                                            };

                                            SkyeValue::new(Rc::from(format!("&{}", value)), SkyeType::Pointer(Box::new(new_inner.type_), new_inner.is_const, true), true)
                                        }
                                        ImplementsHow::No => {
                                            token_error!(
                                                self, op,
                                                format!(
                                                    "Type {} cannot use '&' operator",
                                                    inner.type_.stringify_native()
                                                ).as_ref()
                                            );

                                            SkyeValue::get_unknown()
                                        }
                                    }
                                }
                            }
                        }
                        TokenType::RefConst => {
                            match inner.type_ {
                                SkyeType::Type(type_type) => {
                                    SkyeValue::new(
                                        Rc::from(format!("{}*", inner.value)),
                                        SkyeType::Type(Box::new(SkyeType::Pointer(type_type, true, true))),
                                        true
                                    )
                                }
                                SkyeType::Unknown(_) => {
                                    SkyeValue::special(SkyeType::Type(Box::new(SkyeType::Pointer(Box::new(inner.type_), true, true))))
                                }
                                _ => {
                                    let new_inner = inner.follow_reference(self.external_zero_check(op, index));

                                    match new_inner.type_.implements_op(Operator::ConstRef) {
                                        ImplementsHow::Native(_) | ImplementsHow::ThirdParty => {
                                            let value = {
                                                if inner_expr.is_valid_assignment_target() {
                                                    new_inner.value
                                                } else {
                                                    let tmp_var = self.get_temporary_var();

                                                    self.definitions[index].push_indent();
                                                    self.definitions[index].push(&new_inner.type_.stringify());
                                                    self.definitions[index].push(" ");
                                                    self.definitions[index].push(&tmp_var);
                                                    self.definitions[index].push(" = ");
                                                    self.definitions[index].push(&new_inner.value);
                                                    self.definitions[index].push(";\n");

                                                    Rc::from(tmp_var)
                                                }
                                            };

                                            SkyeValue::new(Rc::from(format!("&{}", value)), SkyeType::Pointer(Box::new(new_inner.type_), true, false), true)
                                        }
                                        ImplementsHow::No => {
                                            token_error!(
                                                self, op,
                                                format!(
                                                    "Type {} cannot use '&const' operator",
                                                    new_inner.type_.stringify_native()
                                                ).as_ref()
                                            );

                                            SkyeValue::get_unknown()
                                        }
                                    }
                                }
                            }
                        }
                        TokenType::Star => {
                            match inner.type_ {
                                SkyeType::Pointer(ref ptr_type, is_const, _) => {
                                    if matches!(**ptr_type, SkyeType::Void) {
                                        ast_error!(self, inner_expr, "Cannot dereference a voidptr");
                                        SkyeValue::get_unknown()
                                    } else {
                                        let inner_value = ctx.run(|ctx| self.zero_check(&inner, op, "Null pointer dereference", index, ctx)).await;
                                        SkyeValue::new(Rc::from(format!("*{}", inner_value)), *ptr_type.clone(), is_const)
                                    }
                                }
                                SkyeType::Type(type_type) => {
                                    if !type_type.can_be_instantiated(false) {
                                        ast_error!(self, inner_expr, format!("Cannot instantiate type {}", type_type.stringify_native()).as_ref());
                                    }

                                    SkyeValue::new(
                                        Rc::from(format!("{}*", inner.value)),
                                        SkyeType::Type(Box::new(SkyeType::Pointer(type_type, false, false))),
                                        true
                                    )
                                }
                                SkyeType::Unknown(_) => {
                                    SkyeValue::special(SkyeType::Type(Box::new(SkyeType::Pointer(Box::new(inner.type_), false, false))))
                                }
                                _ => {
                                    match inner.type_.implements_op(Operator::Deref) {
                                        ImplementsHow::Native(_) => {
                                            // never happens as far as i know, but i'll keep it here in case i decide to make it do something else
                                            return SkyeValue::new(Rc::from(format!("*{}", inner.value)), inner.type_, false);
                                        }
                                        ImplementsHow::ThirdParty => {
                                            let search_tok = Token::dummy(Rc::from("__deref__"));
                                            if let Some(value) = self.get_method(&inner, &search_tok, true, index) {
                                                let v = Vec::new();
                                                return ctx.run(|ctx| self.call(&value, expr, inner_expr, &v, index, allow_unknown, ctx)).await;
                                            }
                                        }
                                        ImplementsHow::No => (),
                                    }

                                    match inner.type_.implements_op(Operator::AsPtr) {
                                        ImplementsHow::Native(_) => unreachable!(),
                                        ImplementsHow::ThirdParty => {
                                            let search_tok = Token::dummy(Rc::from("__asptr__"));
                                            if let Some(value) = self.get_method(&inner, &search_tok, true, index) {
                                                let v = Vec::new();
                                                let value = ctx.run(|ctx| self.call(&value, expr, inner_expr, &v, index, allow_unknown, ctx)).await;

                                                let (inner_type, is_const) = {
                                                    if let SkyeType::Pointer(inner, ptr_is_const, _) = &value.type_ {
                                                        (*inner.clone(), *ptr_is_const)
                                                    } else {
                                                        token_error!(
                                                            self, op,
                                                            format!(
                                                                "Expecting pointer as return type of __asptr__ (got {})",
                                                                value.type_.stringify_native()
                                                            ).as_ref()
                                                        );

                                                        return SkyeValue::get_unknown();
                                                    }
                                                };

                                                let value_value = ctx.run(|ctx| self.zero_check(&value, op, "Null pointer dereference", index, ctx)).await;
                                                SkyeValue::new(Rc::from(format!("*{}", value_value)), inner_type, is_const)
                                            } else {
                                                token_error!(
                                                    self, op,
                                                    format!(
                                                        "Prefix unary '*' operator is not implemented for type {}",
                                                        inner.type_.stringify_native()
                                                    ).as_ref()
                                                );

                                                SkyeValue::get_unknown()
                                            }
                                        }
                                        ImplementsHow::No => {
                                            token_error!(
                                                self, op,
                                                format!(
                                                    "Type {} cannot use prefix unary '*' operator",
                                                    inner.type_.stringify_native()
                                                ).as_ref()
                                            );

                                            SkyeValue::get_unknown()
                                        }
                                    }
                                }
                            }
                        }
                        TokenType::StarConst => {
                            match inner.type_ {
                                SkyeType::Pointer(ref ptr_type, ..) => { // readonly dereference
                                    if matches!(**ptr_type, SkyeType::Void) {
                                        ast_error!(self, inner_expr, "Cannot dereference a voidptr");
                                        SkyeValue::get_unknown()
                                    } else {
                                        let inner_value = ctx.run(|ctx| self.zero_check(&inner, op, "Null pointer dereference", index, ctx)).await;
                                        SkyeValue::new(Rc::from(format!("*{}", inner_value)), *ptr_type.clone(), true)
                                    }
                                }
                                SkyeType::Type(type_type) => {
                                    if !type_type.can_be_instantiated(false) {
                                        ast_error!(self, inner_expr, format!("Cannot instantiate type {}", type_type.stringify_native()).as_ref());
                                    }

                                    SkyeValue::new(
                                        Rc::from(format!("{}*", inner.value)),
                                        SkyeType::Type(Box::new(SkyeType::Pointer(type_type, true, false))),
                                        true
                                    )
                                }
                                SkyeType::Unknown(_) => {
                                    SkyeValue::special(SkyeType::Type(Box::new(SkyeType::Pointer(Box::new(inner.type_), true, false))))
                                }
                                _ => {
                                    ctx.run(|ctx| self.unary_operator(
                                        inner, inner_expr, expr, "*", "__constderef__", Operator::ConstDeref, op, index, allow_unknown, ctx
                                    )).await
                                }
                            }
                        }
                        TokenType::Try => {
                            if matches!(self.curr_function, CurrentFn::None) {
                                token_error!(self, op, "Can only use \"try\" operator inside functions");
                                return SkyeValue::get_unknown();
                            }

                            if let SkyeType::Enum(full_name, variants, name) = &inner.type_ {
                                let (return_type, return_expr) = {
                                    if let CurrentFn::Some(return_type, return_type_expr) = &self.curr_function {
                                        if let SkyeType::Enum(_, return_variants, return_type_name) = return_type {
                                            if return_variants.is_some() && name.as_ref() == return_type_name.as_ref() {
                                                (return_type.clone(), return_type_expr.clone())
                                            } else {
                                                token_error!(
                                                    self, op,
                                                    format!(
                                                        "Can only use \"try\" operator inside functions returning core::Result or core::Option (got {})",
                                                        return_type.stringify_native()
                                                    ).as_ref()
                                                );

                                                ast_note!(return_type_expr, "Return type defined here");
                                                return SkyeValue::get_unknown();
                                            }
                                        } else {
                                            token_error!(
                                                self, op,
                                                format!(
                                                    "Can only use \"try\" operator inside functions returning core::Result or core::Option (got {})",
                                                    return_type.stringify_native()
                                                ).as_ref()
                                            );

                                            ast_note!(return_type_expr, "Return type defined here");
                                            return SkyeValue::get_unknown();
                                        }
                                    } else {
                                        unreachable!();
                                    }
                                };

                                let tmp_var_name = self.get_temporary_var();

                                self.definitions[index].push_indent();
                                self.definitions[index].push(&full_name);
                                self.definitions[index].push(" ");
                                self.definitions[index].push(&tmp_var_name);
                                self.definitions[index].push(" = ");
                                self.definitions[index].push(&inner.value);
                                self.definitions[index].push(";\n");

                                self.definitions[index].push_indent();
                                self.definitions[index].push("if (");
                                self.definitions[index].push(&tmp_var_name);
                                self.definitions[index].push(".kind == ");

                                match name.as_ref() {
                                    "core_DOT_Option" => {
                                        self.definitions[index].push("core_DOT_Option_DOT_Kind_DOT_None) {\n");
                                        self.definitions[index].inc_indent();

                                        ctx.run(|ctx| self.handle_all_deferred(index, false, expr, "in the propagation branch of this expression", ctx)).await;

                                        self.definitions[index].push_indent();
                                        self.definitions[index].push("return ");

                                        if return_type.equals(&inner.type_, EqualsLevel::Typewise) {
                                            self.definitions[index].push(&tmp_var_name);
                                            self.definitions[index].push(";\n");
                                        } else if let SkyeType::Enum(full_name, ..) = &return_type {
                                            self.definitions[index].push(&full_name);
                                            self.definitions[index].push("_DOT_None;\n");
                                        } else {
                                            unreachable!();
                                        }

                                        self.definitions[index].dec_indent();
                                        self.definitions[index].push_indent();
                                        self.definitions[index].push("}\n");

                                        if let Some(variant) = variants.as_ref().unwrap().get("Some") {
                                            SkyeValue::new(
                                                Rc::from(format!("{}.Some", tmp_var_name)),
                                                variant.clone(),
                                                true
                                            )
                                        } else {
                                            // when variant is void
                                            SkyeValue::special(SkyeType::Void)
                                        }
                                    }
                                    "core_DOT_Result" => {
                                        self.definitions[index].push("core_DOT_Result_DOT_Kind_DOT_Error) {\n");
                                        self.definitions[index].inc_indent();

                                        ctx.run(|ctx| self.handle_all_deferred(index, false, expr, "in the propagation branch of this expression", ctx)).await;

                                        self.definitions[index].push_indent();
                                        self.definitions[index].push("return ");

                                        if return_type.equals(&inner.type_, EqualsLevel::Typewise) {
                                            self.definitions[index].push(&tmp_var_name);
                                            self.definitions[index].push(";\n");
                                        } else if let SkyeType::Enum(full_name, return_variants, _) = &return_type {
                                            if let Some(return_variant) = return_variants.as_ref().unwrap().get("Error") {
                                                if let Some(variant) = variants.as_ref().unwrap().get("Error") {
                                                    if variant.equals(return_variant, EqualsLevel::Typewise) {
                                                        self.definitions[index].push(&full_name);
                                                        self.definitions[index].push("_DOT_Error(");
                                                        self.definitions[index].push(&tmp_var_name);
                                                        self.definitions[index].push(".Error);\n");
                                                    } else {
                                                        ast_error!(
                                                            self, expr,
                                                            format!(
                                                                "core::Result \"Error\" variant type ({}) does not match with return type's \"Error\" variant type ({})",
                                                                variant.stringify_native(), return_variant.stringify_native(),
                                                            ).as_ref()
                                                        );

                                                        ast_note!(return_expr, "Return type defined here");
                                                    }
                                                } else {
                                                    ast_error!(
                                                        self, expr,
                                                        format!(
                                                            "core::Result \"Error\" variant type (void) does not match with return type's \"Error\" variant type ({})",
                                                            return_variant.stringify_native(),
                                                        ).as_ref()
                                                    );

                                                    ast_note!(return_expr, "Return type defined here");
                                                }
                                            } else if let Some(variant) = variants.as_ref().unwrap().get("Error") {
                                                ast_error!(
                                                    self, expr,
                                                    format!(
                                                        "core::Result \"Error\" variant type ({}) does not match with return type's \"Error\" variant type (void)",
                                                        variant.stringify_native(),
                                                    ).as_ref()
                                                );

                                                ast_note!(return_expr, "Return type defined here");
                                            } else {
                                                self.definitions[index].push(&full_name);
                                                self.definitions[index].push("_DOT_Error;\n");
                                            }
                                        } else {
                                            unreachable!();
                                        }

                                        self.definitions[index].dec_indent();
                                        self.definitions[index].push_indent();
                                        self.definitions[index].push("}\n");

                                        if let Some(variant) = variants.as_ref().unwrap().get("Ok") {
                                            SkyeValue::new(
                                                Rc::from(format!("{}.Ok", tmp_var_name)),
                                                variant.clone(),
                                                true
                                            )
                                        } else {
                                            // when variant is void
                                            SkyeValue::special(SkyeType::Void)
                                        }
                                    }
                                    _ => {
                                        ast_error!(
                                            self, inner_expr,
                                            format!(
                                                "Can only use \"try\" operator on expressions returning core::Result or core::Option (got {})",
                                                inner.type_.stringify_native()
                                            ).as_ref()
                                        );

                                        SkyeValue::get_unknown()
                                    }
                                }
                            } else {
                                ast_error!(
                                    self, inner_expr,
                                    format!(
                                        "Can only use \"try\" operator on expressions returning core::Result or core::Option (got {})",
                                        inner.type_.stringify_native()
                                    ).as_ref()
                                );

                                SkyeValue::get_unknown()
                            }
                        }
                        TokenType::At => {
                            if let SkyeType::Type(inner_type) = inner.type_ {
                                if let SkyeType::Macro(name, params, body) = &*inner_type {
                                    if matches!(params, MacroParams::None) {
                                        if let MacroBody::Binding(return_type) = body {
                                            let ret_type = ctx.run(|ctx| self.evaluate(return_type, index, allow_unknown, ctx)).await;

                                            if let SkyeType::Type(inner_type) = ret_type.type_ {
                                                if !inner_type.check_completeness() {
                                                    ast_error!(self, return_type, "Cannot use incomplete type directly");
                                                    ast_note!(return_type, "Define this type or reference it through a pointer");
                                                }

                                                SkyeValue::new(Rc::clone(name), *inner_type, true)
                                            } else {
                                                ast_error!(
                                                    self, return_type,
                                                    format!(
                                                        "Expecting type as return type (got {})",
                                                        ret_type.type_.stringify_native()
                                                    ).as_ref()
                                                );

                                                ast_note!(expr, "This error is a result of this macro expansion");
                                                SkyeValue::get_unknown()
                                            }
                                        } else {
                                            ast_error!(self, expr, "Macro is not allowed here");
                                            SkyeValue::get_unknown()
                                        }
                                    } else {
                                        SkyeValue::new(Rc::clone(name), *inner_type, true)
                                    }
                                } else {
                                    if !matches!(&*inner_type, SkyeType::Unknown(_)) {
                                        token_error!(
                                            self, op,
                                            format!(
                                                "'@' can only be used on macros (got {})",
                                                inner_type.stringify_native()
                                            ).as_ref()
                                        );
                                    }

                                    SkyeValue::get_unknown()
                                }
                            } else {
                                if !matches!(inner.type_, SkyeType::Unknown(_)) {
                                    token_error!(
                                        self, op,
                                        format!(
                                            "'@' can only be used on macros (got {})",
                                            inner.type_.stringify_native()
                                        ).as_ref()
                                    );
                                }

                                SkyeValue::get_unknown()
                            }
                        }
                        _ => unreachable!()
                    }
                } else {
                    match op.type_ {
                        TokenType::PlusPlus => {
                            let new_inner = inner.follow_reference(self.external_zero_check(op, index));

                            if new_inner.is_const {
                                ast_error!(self, inner_expr, "Cannot apply '++' operator on const value");
                                new_inner
                            } else if !inner_expr.is_valid_assignment_target() {
                                ast_error!(self, inner_expr, "Can only apply '++' operator on valid assignment targets");
                                new_inner
                            } else {
                                ctx.run(|ctx| self.post_eval_unary_operator(
                                    new_inner, inner_expr, expr, "++",
                                    "__inc__", Operator::Inc, op, index, allow_unknown, ctx
                                )).await
                            }
                        }
                        TokenType::MinusMinus => {
                            let new_inner = inner.follow_reference(self.external_zero_check(op, index));

                            if new_inner.is_const {
                                ast_error!(self, inner_expr, "Cannot apply '--' operator on const value");
                                new_inner
                            } else if !inner_expr.is_valid_assignment_target() {
                                ast_error!(self, inner_expr, "Can only apply '--' operator on valid assignment targets");
                                new_inner
                            } else {
                                ctx.run(|ctx| self.post_eval_unary_operator(
                                    new_inner, inner_expr, expr, "--",
                                    "__dec__", Operator::Dec, op, index, allow_unknown, ctx
                                )).await
                            }
                        }
                        _ => unreachable!()
                    }
                }
            }
            Expression::Binary { left: left_expr, op, right: right_expr } => {
                let left = ctx.run(|ctx| self.evaluate(&left_expr, index, allow_unknown, ctx)).await;

                match op.type_ {
                    TokenType::Plus => {
                        ctx.run(|ctx| self.binary_operator(
                            left, None, &left_expr, &right_expr,
                            expr, "+", op, "__add__", Operator::Add,
                            index, allow_unknown, ctx
                        )).await
                    }
                    TokenType::Minus => {
                        ctx.run(|ctx| self.binary_operator(
                            left, None, &left_expr, &right_expr,
                            expr, "-", op, "__sub__", Operator::Sub,
                            index, allow_unknown, ctx
                        )).await
                    }
                    TokenType::Slash => {
                        ctx.run(|ctx| self.binary_operator_with_zero_check(
                            left, None, &left_expr, &right_expr,
                            expr, "/", op, "__div__", Operator::Div,
                            index, allow_unknown, ctx
                        )).await
                    }
                    TokenType::Star => {
                        ctx.run(|ctx| self.binary_operator(
                            left, None, &left_expr, &right_expr,
                            expr, "*", op, "__mul__", Operator::Mul,
                            index, allow_unknown, ctx
                        )).await
                    }
                    TokenType::Mod => {
                        ctx.run(|ctx| self.binary_operator_with_zero_check(
                            left, None, &left_expr, &right_expr,
                            expr, "%", op, "__mod__", Operator::Mod,
                            index, allow_unknown, ctx
                        )).await
                    }
                    TokenType::ShiftLeft => {
                        ctx.run(|ctx| self.binary_operator_int_on_right(
                            left, &left_expr, &right_expr,
                            expr, "<<", op, "__shl__", Operator::Shl,
                            index, allow_unknown, ctx
                        )).await
                    }
                    TokenType::ShiftRight => {
                        ctx.run(|ctx| self.binary_operator_int_on_right(
                            left, &left_expr, &right_expr,
                            expr, ">>", op, "__shr__", Operator::Shr,
                            index, allow_unknown, ctx
                        )).await
                    }
                    TokenType::LogicOr => {
                        let new_left = left.follow_reference(self.external_zero_check(op, index));

                        match new_left.type_.implements_op(Operator::Or) {
                            ImplementsHow::Native(compatible_types) => {
                                // needed so short circuiting can work
                                let tmp_var = self.get_temporary_var();

                                self.definitions[index].push_indent();
                                self.definitions[index].push("u8 ");
                                self.definitions[index].push(&tmp_var);
                                self.definitions[index].push(";\n");

                                self.definitions[index].push_indent();
                                self.definitions[index].push("if (");
                                self.definitions[index].push(&new_left.value);
                                self.definitions[index].push(") {\n");
                                self.definitions[index].inc_indent();

                                self.definitions[index].push_indent();
                                self.definitions[index].push(&tmp_var);
                                self.definitions[index].push(" = 1;\n");
                                self.definitions[index].dec_indent();

                                self.definitions[index].push_indent();
                                self.definitions[index].push("} else {\n");
                                self.definitions[index].inc_indent();

                                let right = ctx.run(|ctx| self.evaluate(&right_expr, index, allow_unknown, ctx)).await
                                    .follow_reference(self.external_zero_check(op, index));

                                if !(
                                    matches!(new_left.type_, SkyeType::Unknown(_)) ||
                                    new_left.type_.equals(&right.type_, EqualsLevel::Typewise) ||
                                    compatible_types.contains(&right.type_)
                                ) {
                                    ast_error!(
                                        self, right_expr,
                                        format!(
                                            "Left operand type ({}) does not match right operand type ({})",
                                            new_left.type_.stringify_native(), right.type_.stringify_native()
                                        ).as_ref()
                                    );
                                }

                                self.definitions[index].push_indent();
                                self.definitions[index].push(&tmp_var);
                                self.definitions[index].push(" = ");
                                self.definitions[index].push(&right.value);
                                self.definitions[index].push(";\n");
                                self.definitions[index].dec_indent();

                                self.definitions[index].push_indent();
                                self.definitions[index].push("}\n");

                                SkyeValue::new(Rc::from(tmp_var), SkyeType::U8, false)
                            }
                            ImplementsHow::ThirdParty => {
                                let search_tok = Token::dummy(Rc::from("__or__"));
                                if let Some(value) = self.get_method(&new_left, &search_tok, true, index) {
                                    let args = vec![*right_expr.clone()];
                                    ctx.run(|ctx| self.call(&value, expr, left_expr, &args, index, allow_unknown, ctx)).await
                                } else {
                                    ast_error!(
                                        self, left_expr,
                                        format!(
                                            "Binary '||' operator is not implemented for type {}",
                                            new_left.type_.stringify_native()
                                        ).as_ref()
                                    );

                                    SkyeValue::get_unknown()
                                }
                            }
                            ImplementsHow::No => {
                                ast_error!(
                                    self, left_expr,
                                    format!(
                                        "Type {} cannot use binary '||' operator",
                                        new_left.type_.stringify_native()
                                    ).as_ref()
                                );

                                SkyeValue::get_unknown()
                            }
                        }
                    }
                    TokenType::LogicAnd => {
                        let new_left = left.follow_reference(self.external_zero_check(op, index));

                        match new_left.type_.implements_op(Operator::And) {
                            ImplementsHow::Native(compatible_types) => {
                                // needed so short circuiting can work
                                let tmp_var = self.get_temporary_var();

                                self.definitions[index].push_indent();
                                self.definitions[index].push("u8 ");
                                self.definitions[index].push(&tmp_var);
                                self.definitions[index].push(";\n");

                                self.definitions[index].push_indent();
                                self.definitions[index].push("if (");
                                self.definitions[index].push(&new_left.value);
                                self.definitions[index].push(") {\n");
                                self.definitions[index].inc_indent();

                                let right = ctx.run(|ctx| self.evaluate(&right_expr, index, allow_unknown, ctx)).await
                                    .follow_reference(self.external_zero_check(op, index));

                                if !(
                                    matches!(new_left.type_, SkyeType::Unknown(_)) ||
                                    new_left.type_.equals(&right.type_, EqualsLevel::Typewise) ||
                                    compatible_types.contains(&right.type_)
                                ) {
                                    ast_error!(
                                        self, right_expr,
                                        format!(
                                            "Left operand type ({}) does not match right operand type ({})",
                                            new_left.type_.stringify_native(), right.type_.stringify_native()
                                        ).as_ref()
                                    );
                                }

                                self.definitions[index].push_indent();
                                self.definitions[index].push(&tmp_var);
                                self.definitions[index].push(" = ");
                                self.definitions[index].push(&right.value);
                                self.definitions[index].push(";\n");
                                self.definitions[index].dec_indent();

                                self.definitions[index].push_indent();
                                self.definitions[index].push("} else {\n");
                                self.definitions[index].inc_indent();

                                self.definitions[index].push_indent();
                                self.definitions[index].push(&tmp_var);
                                self.definitions[index].push(" = 0;\n");
                                self.definitions[index].dec_indent();

                                self.definitions[index].push_indent();
                                self.definitions[index].push("}\n");

                                SkyeValue::new(Rc::from(tmp_var), SkyeType::U8, false)
                            }
                            ImplementsHow::ThirdParty => {
                                let search_tok = Token::dummy(Rc::from("__and__"));
                                if let Some(value) = self.get_method(&new_left, &search_tok, true, index) {
                                    let args = vec![*right_expr.clone()];
                                    ctx.run(|ctx| self.call(&value, expr, left_expr, &args, index, allow_unknown, ctx)).await
                                } else {
                                    ast_error!(
                                        self, left_expr,
                                        format!(
                                            "Binary '&&' operator is not implemented for type {}",
                                            new_left.type_.stringify_native()
                                        ).as_ref()
                                    );

                                    SkyeValue::get_unknown()
                                }
                            }
                            ImplementsHow::No => {
                                ast_error!(
                                    self, left_expr,
                                    format!(
                                        "Type {} cannot use binary '&&' operator",
                                        new_left.type_.stringify_native()
                                    ).as_ref()
                                );

                                SkyeValue::get_unknown()
                            }
                        }
                    }
                    TokenType::BitwiseXor => {
                        ctx.run(|ctx| self.binary_operator(
                            left, None, &left_expr, &right_expr,
                            expr, "^", op, "__xor__", Operator::Xor,
                            index, allow_unknown, ctx
                        )).await
                    }
                    TokenType::BitwiseOr => {
                        if left.type_.is_type() || matches!(left.type_, SkyeType::Void) {
                            let right = ctx.run(|ctx| self.evaluate(&right_expr, index, allow_unknown, ctx)).await;

                            if right.type_.is_type() || matches!(right.type_, SkyeType::Void) {
                                SkyeValue::special(SkyeType::Group(Box::new(left.type_), Box::new(right.type_)))
                            } else {
                                ast_error!(
                                    self, right_expr,
                                    format!(
                                        "Left operand type ({}) does not match right operand type ({})",
                                        left.type_.stringify_native(), right.type_.stringify_native()
                                    ).as_ref()
                                );

                                SkyeValue::get_unknown()
                            }
                        } else {
                            ctx.run(|ctx| self.binary_operator(
                                left, None, &left_expr, &right_expr,
                                expr, "|", op, "__bitor__", Operator::BitOr,
                                index, allow_unknown, ctx
                            )).await
                        }
                    }
                    TokenType::BitwiseAnd => {
                        ctx.run(|ctx| self.binary_operator(
                            left, None, &left_expr, &right_expr,
                            expr, "&", op, "__bitand__", Operator::BitAnd,
                            index, allow_unknown, ctx
                        )).await
                    }
                    TokenType::Greater => {
                        ctx.run(|ctx| self.binary_operator(
                            left, Some(SkyeType::U8), &left_expr, &right_expr,
                            expr, ">", op, "__gt__", Operator::Gt,
                            index, allow_unknown, ctx
                        )).await
                    }
                    TokenType::GreaterEqual => {
                        ctx.run(|ctx| self.binary_operator(
                            left, Some(SkyeType::U8), &left_expr, &right_expr,
                            expr, ">=", op, "__ge__", Operator::Ge,
                            index, allow_unknown, ctx
                        )).await
                    }
                    TokenType::Less => {
                        ctx.run(|ctx| self.binary_operator(
                            left, Some(SkyeType::U8), &left_expr, &right_expr,
                            expr, "<", op, "__lt__", Operator::Lt,
                            index, allow_unknown, ctx
                        )).await
                    }
                    TokenType::LessEqual => {
                        ctx.run(|ctx| self.binary_operator(
                            left, Some(SkyeType::U8), &left_expr, &right_expr,
                            expr, "<=", op, "__le__", Operator::Le,
                            index, allow_unknown, ctx
                        )).await
                    }
                    TokenType::EqualEqual => {
                        if let SkyeType::Type(inner_left) = left.type_ {
                            ctx.run(|ctx| self.get_type_equality(
                                &*inner_left, right_expr, index, allow_unknown, false, ctx
                            )).await
                        } else {
                            ctx.run(|ctx| self.binary_operator(
                                left, Some(SkyeType::U8), &left_expr, &right_expr,
                                expr, "==", op, "__eq__", Operator::Eq,
                                index, allow_unknown, ctx
                            )).await
                        }
                    }
                    TokenType::BangEqual => {
                        if let SkyeType::Type(inner_left) = left.type_ {
                            ctx.run(|ctx| self.get_type_equality(
                                &*inner_left, right_expr, index, allow_unknown, true, ctx
                            )).await
                        } else {
                            ctx.run(|ctx| self.binary_operator(
                                left, Some(SkyeType::U8), &left_expr, &right_expr,
                                expr, "!=", op, "__ne__", Operator::Ne,
                                index, allow_unknown, ctx
                            )).await
                        }
                    }
                    TokenType::Bang => {
                        let left_ok = matches!(left.type_, SkyeType::Type(_) | SkyeType::Void | SkyeType::Unknown(_));
                        if left_ok {
                            if !left.type_.check_completeness() {
                                ast_error!(self, left_expr, "Cannot use incomplete type directly");
                                ast_note!(left_expr, "Define this type or reference it through a pointer");
                            }

                            if !left.type_.can_be_instantiated(true) {
                                ast_error!(self, left_expr, format!("Cannot instantiate type {}", left.type_.stringify_native()).as_ref());
                            }

                            let right = ctx.run(|ctx| self.evaluate(&right_expr, index, allow_unknown, ctx)).await;

                            if matches!(right.type_, SkyeType::Type(_) | SkyeType::Void | SkyeType::Unknown(_)) {
                                // result operator

                                if !right.type_.check_completeness() {
                                    ast_error!(self, right_expr, "Cannot use incomplete type directly");
                                    ast_note!(left_expr, "Define this type or reference it through a pointer");
                                }

                                if !right.type_.can_be_instantiated(true) {
                                    ast_error!(self, left_expr, format!("Cannot instantiate type {}", right.type_.stringify_native()).as_ref());
                                }

                                let mut custom_token = op.clone();
                                custom_token.set_lexeme("core_DOT_Result");

                                let subscript_expr = Expression::Subscript { subscripted: Box::new(Expression::Variable(custom_token)), paren: op.clone(), args: vec![
                                        *left_expr.clone(),
                                        *right_expr.clone(),
                                    ] };

                                ctx.run(|ctx| self.evaluate(&subscript_expr, index, allow_unknown, ctx)).await
                            } else {
                                ast_error!(
                                    self, right_expr,
                                    format!(
                                        "Invalid operand for result operator (expecting type but got {})",
                                        right.type_.stringify_native()
                                    ).as_ref()
                                );

                                SkyeValue::get_unknown()
                            }
                        } else {
                            ast_error!(
                                self, left_expr,
                                format!(
                                    "Invalid operand for result operator (expecting type but got {})",
                                    left.type_.stringify_native()
                                ).as_ref()
                            );

                            SkyeValue::get_unknown()
                        }
                    }
                    _ => unreachable!()
                }
            }
            Expression::Variable(name) => {
                if let Some(var_info) = self.environment.borrow().get(&name) {
                    SkyeValue::new(name.lexeme.clone(), var_info.type_, var_info.is_const)
                } else if allow_unknown {
                    SkyeValue::special(SkyeType::Unknown(Rc::clone(&name.lexeme)))
                } else {
                    token_error!(
                        self, name,
                        format!(
                            "Cannot reference undefined symbol \"{}\"",
                            name.lexeme
                        ).as_ref()
                    );

                    SkyeValue::get_unknown()
                }
            }
            Expression::Assign { target: target_expr, op, value: value_expr } => {
                let target = ctx.run(|ctx| self.evaluate(&target_expr, index, allow_unknown, ctx)).await;
                let target_type = target.type_.clone();

                if matches!(op.type_, TokenType::Equal) {
                    if target.is_const {
                        ast_error!(self, target_expr, "Assignment target is const");
                    }
                } else {
                    if target.follow_reference(self.external_zero_check(op, index)).is_const {
                        ast_error!(self, target_expr, "Assignment target is const");
                    }
                }

                match op.type_ {
                    TokenType::Equal => {
                        let value = ctx.run(|ctx| self.evaluate(&value_expr, index, allow_unknown, ctx)).await;

                        if target_type.equals(&value.type_, EqualsLevel::Strict) {
                            let search_tok = Token::dummy(Rc::from("__copy__"));
                            let output_value = {
                                if let Some(value) = self.get_method(&value, &search_tok, true, index) {
                                    let v = Vec::new();
                                    let copy_constructor = ctx.run(|ctx| self.call(&value, expr, &value_expr, &v, index, allow_unknown, ctx)).await;
                                    ast_info!(value_expr, "Skye inserted a copy constructor call for this expression"); // +I-copies
                                    copy_constructor
                                } else {
                                    value
                                }
                            };

                            SkyeValue::new(Rc::from(format!("{} = {}", target.value, output_value.value)), output_value.type_, true)
                        } else {
                            ast_error!(
                                self, value_expr,
                                format!(
                                    "Value type ({}) does not match target type ({})",
                                    value.type_.stringify_native(), target_type.stringify_native()
                                ).as_ref()
                            );

                            SkyeValue::get_unknown()
                        }
                    }
                    TokenType::PlusEquals => {
                        ctx.run(|ctx| self.binary_operator(
                            target, None, &target_expr, &value_expr,
                            expr, "+=", op, "__setadd__", Operator::SetAdd,
                            index, allow_unknown, ctx
                        )).await
                    }
                    TokenType::MinusEquals => {
                        ctx.run(|ctx| self.binary_operator(
                            target, None, &target_expr, &value_expr,
                            expr, "-=", op, "__setsub__", Operator::SetSub,
                            index, allow_unknown, ctx
                        )).await
                    }
                    TokenType::StarEquals => {
                        ctx.run(|ctx| self.binary_operator(
                            target, None, &target_expr, &value_expr,
                            expr, "*=", op, "__setmul__", Operator::SetMul,
                            index, allow_unknown, ctx
                        )).await
                    }
                    TokenType::SlashEquals => {
                        ctx.run(|ctx| self.binary_operator_with_zero_check(
                            target, None, &target_expr, &value_expr,
                            expr, "/=", op, "__setdiv__", Operator::SetDiv,
                            index, allow_unknown, ctx
                        )).await
                    }
                    TokenType::ModEquals => {
                        ctx.run(|ctx| self.binary_operator_with_zero_check(
                            target, None, &target_expr, &value_expr,
                            expr, "%=", op, "__setmod__", Operator::SetMod,
                            index, allow_unknown, ctx
                        )).await
                    }
                    TokenType::ShiftLeftEquals => {
                        ctx.run(|ctx| self.binary_operator(
                            target, None, &target_expr, &value_expr,
                            expr, "<<=", op, "__setshl__", Operator::SetShl,
                            index, allow_unknown, ctx
                        )).await
                    }
                    TokenType::ShiftRightEquals => {
                        ctx.run(|ctx| self.binary_operator(
                            target, None, &target_expr, &value_expr,
                            expr, ">>=", op, "__setshr__", Operator::SetShr,
                            index, allow_unknown, ctx
                        )).await
                    }
                    TokenType::AndEquals => {
                        ctx.run(|ctx| self.binary_operator(
                            target, None, &target_expr, &value_expr,
                            expr, "&=", op, "__setand__", Operator::SetAnd,
                            index, allow_unknown, ctx
                        )).await
                    }
                    TokenType::XorEquals => {
                        ctx.run(|ctx| self.binary_operator(
                            target, None, &target_expr, &value_expr,
                            expr, "^=", op, "__setxor__", Operator::SetXor,
                            index, allow_unknown, ctx
                        )).await
                    }
                    TokenType::OrEquals => {
                        ctx.run(|ctx| self.binary_operator(
                            target, None, &target_expr, &value_expr,
                            expr, "|=", op, "__setor__", Operator::SetOr,
                            index, allow_unknown, ctx
                        )).await
                    }
                    _ => unreachable!()
                }
            }
            Expression::Call(callee_expr, _, arguments) => {
                let callee = ctx.run(|ctx| self.evaluate(&callee_expr, index, allow_unknown, ctx)).await;
                ctx.run(|ctx| self.call(&callee, expr, callee_expr, arguments, index, allow_unknown, ctx)).await
            }
            Expression::FnPtr { kw, return_type: return_type_expr, params } => {
                let return_type = ctx.run(|ctx| self.get_return_type(return_type_expr, index, allow_unknown, ctx)).await;
                let (params_string, params_output) = ctx.run(|ctx| self.get_params(params, None, false, index, allow_unknown, ctx)).await;

                let return_stringified = return_type.stringify();
                let inner_type = SkyeType::Function(params_output, Box::new(return_type), false);

                let (mangled, type_) = self.generate_fn_signature(kw, &inner_type, &return_stringified, &params_string);

                SkyeValue::new(mangled.into(), type_, true)
            }
            Expression::Ternary { condition: cond_expr, then_expr: then_branch_expr, else_expr: else_branch_expr, .. } => {
                let cond = ctx.run(|ctx| self.evaluate(&cond_expr, index, allow_unknown, ctx)).await;

                match cond.type_ {
                    SkyeType::U8  | SkyeType::I8  | SkyeType::U16 | SkyeType::I16 |
                    SkyeType::U32 | SkyeType::I32 | SkyeType::U64 | SkyeType::I64 |
                    SkyeType::Usz | SkyeType::AnyInt | SkyeType::Unknown(_) => (),
                    _ => {
                        ast_error!(
                            self, cond_expr,
                            format!(
                                "Expecting expression of primitive arithmetic type for ternary operator condition (got {})",
                                cond.type_.stringify_native()
                            ).as_ref()
                        );
                    }
                }

                let tmp_var = self.get_temporary_var();

                let tmp_index = self.definitions.len();
                self.definitions.push(CodeOutput::new());

                let indent = self.definitions[index].indent;
                self.definitions[tmp_index].set_indent(indent);
                self.definitions[tmp_index].push_indent();
                self.definitions[tmp_index].push("if ");

                let not_grouping = !matches!(**cond_expr, Expression::Grouping(_));

                if not_grouping {
                    self.definitions[tmp_index].push("(");
                }

                self.definitions[tmp_index].push(&cond.value);

                if not_grouping {
                    self.definitions[tmp_index].push(")");
                }

                self.definitions[tmp_index].push(" {\n");
                self.definitions[tmp_index].inc_indent();

                let then_branch = ctx.run(|ctx| self.evaluate(&then_branch_expr, tmp_index, allow_unknown, ctx)).await;

                let is_not_void = !matches!(then_branch.type_, SkyeType::Void);
                let value_is_not_empty = then_branch.value.as_ref() != "";

                if is_not_void || value_is_not_empty {
                    self.definitions[tmp_index].push_indent();
                }

                if is_not_void {
                    self.definitions[tmp_index].push(&tmp_var);
                    self.definitions[tmp_index].push(" = ");
                }

                if value_is_not_empty {
                    self.definitions[tmp_index].push(&then_branch.value);
                    self.definitions[tmp_index].push(";\n");
                }

                self.definitions[tmp_index].dec_indent();

                self.definitions[tmp_index].push_indent();
                self.definitions[tmp_index].push("} else {\n");
                self.definitions[tmp_index].inc_indent();

                let else_branch = ctx.run(|ctx| self.evaluate(&else_branch_expr, tmp_index, allow_unknown, ctx)).await;

                let is_not_void = !matches!(then_branch.type_, SkyeType::Void);
                let value_is_not_empty = else_branch.value.as_ref() != "";

                if is_not_void || value_is_not_empty {
                    self.definitions[tmp_index].push_indent();
                }

                if is_not_void {
                    self.definitions[tmp_index].push(&tmp_var);
                    self.definitions[tmp_index].push(" = ");
                }

                if value_is_not_empty {
                    self.definitions[tmp_index].push(&else_branch.value);
                    self.definitions[tmp_index].push(";\n");
                }

                self.definitions[tmp_index].dec_indent();

                self.definitions[tmp_index].push_indent();
                self.definitions[tmp_index].push("}\n");

                if !then_branch.type_.equals(&else_branch.type_, EqualsLevel::Typewise) {
                    ast_error!(
                        self, else_branch_expr,
                        format!(
                            "Ternary operator then branch type ({}) does not match else branch type ({})",
                            then_branch.type_.stringify_native(), else_branch.type_.stringify_native()
                        ).as_ref()
                    );
                }

                if is_not_void {
                    self.definitions[index].push_indent();
                    self.definitions[index].push(&then_branch.type_.stringify());
                    self.definitions[index].push(" ");
                    self.definitions[index].push(&tmp_var);
                    self.definitions[index].push(";\n");
                }

                let tmp_code = self.definitions.swap_remove(tmp_index);
                self.definitions[index].push(&tmp_code.code);

                SkyeValue::new(Rc::from(tmp_var), then_branch.type_, true)
            }
            Expression::CompoundLiteral { type_: identifier_expr, fields, .. } => {
                let identifier_type = ctx.run(|ctx| self.evaluate(&identifier_expr, index, allow_unknown, ctx)).await;

                match &identifier_type.type_ {
                    SkyeType::Type(inner_type) => {
                        if !inner_type.check_completeness() {
                            ast_error!(self, identifier_expr, "Cannot use incomplete type directly");
                            ast_note!(identifier_expr, "Define this type or reference it through a pointer");
                        }

                        match &**inner_type {
                            SkyeType::Struct(name, def_fields, _) => {
                                if let Some(defined_fields) = def_fields {
                                    if fields.len() != defined_fields.len() {
                                        ast_error!(self, expr, format!(
                                            "Expecting {} fields but got {}",
                                            defined_fields.len(), fields.len()
                                        ).as_str());
                                        return SkyeValue::special(*inner_type.clone());
                                    }

                                    let mut fields_output = String::new();
                                    for (i, field) in fields.iter().enumerate() {
                                        if let Some((field_type, _)) = defined_fields.get(&field.name.lexeme) {
                                            let field_evaluated = ctx.run(|ctx| self.evaluate(&field.expr, index, allow_unknown, ctx)).await;

                                            if !field_type.equals(&field_evaluated.type_, EqualsLevel::Strict) {
                                                ast_error!(
                                                    self, field.expr,
                                                    format!(
                                                        "Invalid type for this field (expecting {} but got {})",
                                                        field_type.stringify_native(), field_evaluated.type_.stringify_native()
                                                    ).as_ref()
                                                );
                                            }

                                            fields_output.push('.');
                                            fields_output.push_str(&field.name.lexeme);
                                            fields_output.push_str(" = ");

                                            let search_tok = Token::dummy(Rc::from("__copy__"));
                                            if let Some(value) = self.get_method(&field_evaluated, &search_tok, true, index) {
                                                let v = Vec::new();
                                                let copy_constructor = ctx.run(|ctx| self.call(&value, expr, &field.expr, &v, index, allow_unknown, ctx)).await;
                                                fields_output.push_str(&copy_constructor.value);

                                                ast_info!(field.expr, "Skye inserted a copy constructor call for this expression"); // +I-copies
                                            } else {
                                                fields_output.push_str(&field_evaluated.value);
                                            }

                                            if i != fields.len() - 1 {
                                                fields_output.push_str(", ");
                                            }
                                        } else {
                                            token_error!(self, field.name, "Unknown struct field");
                                        }
                                    }

                                    SkyeValue::new(Rc::from(format!("({}) {{ {} }}", name, fields_output)), *inner_type.clone(), true)
                                } else {
                                    ast_error!(self, identifier_expr, "Cannot initialize struct that is declared but has no definition");
                                    SkyeValue::get_unknown()
                                }
                            }
                            SkyeType::Bitfield(name, def_fields) => {
                                if let Some(defined_fields) = def_fields {
                                    if fields.len() != defined_fields.len() {
                                        ast_error!(self, expr, format!(
                                            "Expecting {} fields but got {}",
                                            defined_fields.len(), fields.len()
                                        ).as_str());
                                        return SkyeValue::special(*inner_type.clone());
                                    }

                                    let mut fields_output = String::new();
                                    for (i, field) in fields.iter().enumerate() {
                                        if let Some(field_type) = defined_fields.get(&field.name.lexeme) {
                                            let field_evaluated = ctx.run(|ctx| self.evaluate(&field.expr, index, allow_unknown, ctx)).await;

                                            if !field_type.equals(&field_evaluated.type_, EqualsLevel::Strict) {
                                                ast_error!(
                                                    self, field.expr,
                                                    format!(
                                                        "Invalid type for this field (expecting {} but got {})",
                                                        field_type.stringify_native(), field_evaluated.type_.stringify_native()
                                                    ).as_ref()
                                                );
                                            }

                                            // copy costructor here is not needed because bitfields always have numeric types

                                            fields_output.push('.');
                                            fields_output.push_str(&field.name.lexeme);
                                            fields_output.push_str(" = ");
                                            fields_output.push_str(&field_evaluated.value);

                                            if i != fields.len() - 1 {
                                                fields_output.push_str(", ");
                                            }
                                        } else {
                                            token_error!(self, field.name, "Unknown bitfield field");
                                        }
                                    }

                                    SkyeValue::new(Rc::from(format!("({}) {{ {} }}", name, fields_output)), *inner_type.clone(), true)
                                } else {
                                    ast_error!(self, identifier_expr, "Cannot initialize bitfield that is declared but has no definition");
                                    SkyeValue::get_unknown()
                                }
                            }
                            SkyeType::Union(name, def_fields) => {
                                if let Some(defined_fields) = def_fields {
                                    if fields.len() != 1 {
                                        ast_error!(self, expr, "Can only assign one field of a union");
                                        return SkyeValue::special(*inner_type.clone());
                                    }

                                    let mut buf = String::new();
                                    if let Some(field_type) = defined_fields.get(&fields[0].name.lexeme) {
                                        let field_evaluated = ctx.run(|ctx| self.evaluate(&fields[0].expr, index, allow_unknown, ctx)).await;

                                        if !field_type.equals(&field_evaluated.type_, EqualsLevel::Strict) {
                                            ast_error!(
                                                self, fields[0].expr,
                                                format!(
                                                    "Invalid type for this field (expecting {} but got {})",
                                                    field_type.stringify_native(), field_evaluated.type_.stringify_native()
                                                ).as_ref()
                                            );
                                        }

                                        buf.push('.');
                                        buf.push_str(&fields[0].name.lexeme);
                                        buf.push_str(" = ");

                                        let search_tok = Token::dummy(Rc::from("__copy__"));
                                        if let Some(value) = self.get_method(&field_evaluated, &search_tok, true, index) {
                                            let v = Vec::new();
                                            let copy_constructor = ctx.run(|ctx| self.call(&value, expr, &fields[0].expr, &v, index, allow_unknown, ctx)).await;
                                            buf.push_str(&copy_constructor.value);

                                            ast_info!(fields[0].expr, "Skye inserted a copy constructor call for this expression"); // +I-copies
                                        } else {
                                            buf.push_str(&field_evaluated.value);
                                        }
                                    } else {
                                        token_error!(self, fields[0].name, "Unknown union field");
                                    }

                                    SkyeValue::new(Rc::from(format!("({}) {{ {} }}", name, buf)), *inner_type.clone(), true)
                                } else {
                                    ast_error!(self, identifier_expr, "Cannot initialize union that is declared but has no definition");
                                    SkyeValue::get_unknown()
                                }
                            }
                            _ => {
                                ast_error!(
                                    self, identifier_expr,
                                    format!(
                                        "Expecting struct, struct template, union, or bitfield type as compound literal identifier (got {})",
                                        inner_type.stringify_native()
                                    ).as_ref()
                                );

                                SkyeValue::get_unknown()
                            }
                        }
                    }
                    SkyeType::Template(name, definition, generics, generics_names, curr_name, read_env) => {
                        if let Statement::Struct { name: struct_name, fields: defined_fields, .. } = &definition {
                            if fields.len() != defined_fields.len() {
                                ast_error!(self, expr, format!(
                                    "Expecting {} fields but got {}",
                                    defined_fields.len(), fields.len()
                                ).as_str());
                                return SkyeValue::get_unknown();
                            }

                            let mut generics_to_find = HashMap::new();
                            for generic in generics {
                                generics_to_find.insert(Rc::clone(&generic.name.lexeme), None);
                            }

                            let mut fields_map = HashMap::new();
                            for field in defined_fields {
                                if fields_map.contains_key(&field.name.lexeme) {
                                    token_error!(self, field.name, "Cannot define the same struct field multiple times");
                                } else {
                                    fields_map.insert(Rc::clone(&field.name.lexeme), field.expr.clone());
                                }
                            }

                            let tmp_env = Rc::new(RefCell::new(
                                Environment::with_enclosing(Rc::clone(&read_env))
                            ));

                            let mut generics_found_at = HashMap::new();
                            let mut fields_output = String::new();
                            for (i, field) in fields.iter().enumerate() {
                                if let Some(def_field_expr) = fields_map.get(&field.name.lexeme) {
                                    let previous = Rc::clone(&self.environment);
                                    self.environment = Rc::clone(&tmp_env);

                                    let previous_name = self.curr_name.clone();
                                    self.curr_name = curr_name.clone();

                                    let def_evaluated = ctx.run(|ctx| self.evaluate(&def_field_expr, index, true, ctx)).await;

                                    self.curr_name   = previous_name;
                                    self.environment = previous;

                                    let literal_evaluated = ctx.run(|ctx| self.evaluate(&field.expr, index, false, ctx)).await;

                                    let def_type = {
                                        if matches!(def_evaluated.type_, SkyeType::Unknown(_)) {
                                            SkyeType::Type(Box::new(def_evaluated.type_))
                                        } else {
                                            def_evaluated.type_
                                        }
                                    };

                                    if !def_type.check_completeness() {
                                        ast_error!(self, def_field_expr, "Cannot use incomplete type directly");
                                        ast_note!(def_field_expr, "Define this type or reference it through a pointer");
                                        ast_note!(expr, "This error is a result of template generation originating from this compound literal");
                                    }

                                    if let SkyeType::Type(inner_type) = &def_type {
                                        if inner_type.equals(&literal_evaluated.type_, EqualsLevel::Permissive) {
                                            if let Some(inferred) = inner_type.infer_type_from_similar(&literal_evaluated.type_) {
                                                for (generic_name, generic_type) in inferred {
                                                    if let Some(generic_to_find) = generics_to_find.get(&generic_name) {
                                                        if generic_to_find.is_none() {
                                                            if matches!(generic_type, SkyeType::Void) {
                                                                generics_to_find.insert(Rc::clone(&generic_name), Some(generic_type));
                                                            } else {
                                                                generics_to_find.insert(Rc::clone(&generic_name), Some(SkyeType::Type(Box::new(generic_type))));
                                                            }

                                                            generics_found_at.insert(generic_name, i);
                                                        }
                                                    }
                                                }
                                            } else {
                                                ast_error!(
                                                    self, field.expr,
                                                    format!(
                                                        "Field type does not match definition field type (expecting {} but got {})",
                                                        inner_type.stringify_native(), literal_evaluated.type_.stringify_native()
                                                    ).as_ref()
                                                );
                                            }
                                        } else {
                                            ast_error!(
                                                self, field.expr,
                                                format!(
                                                    "Field type does not match definition field type (expecting {} but got {})",
                                                    inner_type.stringify_native(), literal_evaluated.type_.stringify_native()
                                                ).as_ref()
                                            );
                                        }
                                    } else {
                                        ast_error!(
                                            self, field.expr,
                                            format!(
                                                "Expecting type as field type (got {})",
                                                def_type.stringify_native()
                                            ).as_ref()
                                        );
                                    }

                                    fields_output.push('.');
                                    fields_output.push_str(&field.name.lexeme);
                                    fields_output.push_str(" = ");
                                    fields_output.push_str(&literal_evaluated.value);

                                    if i != fields.len() - 1 {
                                        fields_output.push_str(", ");
                                    }
                                } else {
                                    token_error!(self, field.name, "Unknown struct field");
                                }
                            }

                            for expr_generic in generics {
                                let generic_type = generics_to_find.get(&expr_generic.name.lexeme).unwrap();

                                let type_ = {
                                    if let Some(t) = generic_type {
                                        Some(t.clone())
                                    } else if let Some(default) = &expr_generic.default {
                                        let previous = Rc::clone(&self.environment);
                                        self.environment = Rc::clone(&tmp_env);

                                        let evaluated = ctx.run(|ctx| self.evaluate(&default, index, false, ctx)).await;

                                        self.environment = previous;

                                        if matches!(evaluated.type_, SkyeType::Type(_) | SkyeType::Void) {
                                            if evaluated.type_.check_completeness() {
                                                if evaluated.type_.can_be_instantiated(false) {
                                                    Some(evaluated.type_)
                                                } else {
                                                    ast_error!(self, default, format!("Cannot instantiate type {}", evaluated.type_.stringify_native()).as_ref());
                                                    None
                                                }
                                            } else {
                                                ast_error!(self, default, "Cannot use incomplete type directly");
                                                ast_note!(default, "Define this type or reference it through a pointer");
                                                None
                                            }
                                        } else {
                                            ast_error!(
                                                self, default,
                                                format!(
                                                    "Expecting type as default generic (got {})",
                                                    evaluated.type_.stringify_native()
                                                ).as_ref()
                                            );

                                            None
                                        }
                                    } else {
                                        None
                                    }
                                };

                                if let Some(inner_type) = type_ {
                                    if let Some(bounds) = &expr_generic.bounds {
                                        let previous = Rc::clone(&self.environment);
                                        self.environment = Rc::clone(&tmp_env);

                                        let evaluated = ctx.run(|ctx| self.evaluate(&bounds, index, false, ctx)).await;

                                        self.environment = previous;

                                        if evaluated.type_.is_type() || matches!(evaluated.type_, SkyeType::Void) {
                                            if evaluated.type_.is_respected_by(&inner_type) {
                                                let mut env = self.environment.borrow_mut();
                                                env.define(
                                                    Rc::clone(&expr_generic.name.lexeme),
                                                    SkyeVariable::new(
                                                        inner_type, true,
                                                        Some(Box::new(expr_generic.name.clone()))
                                                    )
                                                );
                                            } else {
                                                let at = *generics_found_at.get(&expr_generic.name.lexeme).unwrap();

                                                ast_error!(
                                                    self, fields[at].expr,
                                                    format!(
                                                        "Generic bound is not respected by this type (expecting {} but got {})",
                                                        evaluated.type_.stringify_native(), inner_type.stringify_native()
                                                    ).as_ref()
                                                );

                                                token_note!(expr_generic.name, "Generic defined here");
                                            }
                                        } else {
                                            ast_error!(
                                                self, bounds,
                                                format!(
                                                    "Expecting type or group as generic bound (got {})",
                                                    evaluated.type_.stringify_native()
                                                ).as_ref()
                                            );
                                        }
                                    } else {
                                        let mut env = self.environment.borrow_mut();
                                        env.define(
                                            Rc::clone(&expr_generic.name.lexeme),
                                            SkyeVariable::new(
                                                inner_type, true,
                                                Some(Box::new(expr_generic.name.clone()))
                                            )
                                        );
                                    }
                                } else {
                                    if self.errors == 0 { // avoids having inference errors caused by other errors
                                        ast_error!(self, identifier_expr, "Skye cannot infer the generic types for this struct literal");
                                        ast_note!(identifier_expr, "This expression is a template and requires generic typing");
                                        ast_note!(identifier_expr, "Manually specify the generic types");
                                    }

                                    return SkyeValue::get_unknown();
                                }
                            }

                            let final_name = self.get_generics(&name, &generics_names, &self.environment);
                            let search_tok = Token::dummy(Rc::clone(&final_name));

                            let mut env = self.globals.borrow_mut();
                            if let Some(var) = env.get(&search_tok) {
                                env = tmp_env.borrow_mut();

                                for generic in generics {
                                    env.undef(Rc::clone(&generic.name.lexeme));
                                }

                                if let SkyeType::Type(inner_type) = var.type_ {
                                    return SkyeValue::new(Rc::from(format!("({}) {{ {} }}", final_name, fields_output)), *inner_type, true);
                                } else if let Some(orig_tok) = var.tok {
                                    token_error!(self, struct_name, "This struct's generic type name resolves to an invalid type");
                                    token_note!(orig_tok, "This definition is invalid. Change the name of this symbol");
                                } else {
                                    token_error!(self, struct_name, "This struct's generic type name resolves to an invalid type. An invalid symbol definition is present in the code");
                                }
                            }

                            drop(env);

                            let previous = Rc::clone(&self.environment);
                            self.environment = Rc::clone(&tmp_env);

                            let previous_name = self.curr_name.clone();
                            self.curr_name = curr_name.clone();

                            let type_ = {
                                match ctx.run(|ctx| self.execute(&definition, 0, ctx)).await {
                                    Ok(item) => item.unwrap_or_else(|| {
                                        ast_error!(self, expr, "Could not process template generation for this expression");
                                        SkyeType::get_unknown()
                                    }),
                                    Err(_) => unreachable!("execution interrupt happened out of context")
                                }
                            };

                            self.curr_name   = previous_name;
                            self.environment = previous;

                            env = tmp_env.borrow_mut();
                            for generic in generics {
                                env.undef(Rc::clone(&generic.name.lexeme));
                            }

                            env.define(
                                Rc::clone(&final_name),
                                SkyeVariable::new(
                                    type_.clone(), true, None
                                )
                            );

                            if let SkyeType::Type(inner_type) = type_ {
                                return SkyeValue::new(Rc::from(format!("({}) {{ {} }}", final_name, fields_output)), *inner_type, true);
                            } else {
                                panic!("struct template generation resulted in not a type");
                            }
                        } else {
                            ast_error!(
                                self, identifier_expr,
                                format!(
                                    "Expecting struct, struct template, union, or bitfield type as compound literal identifier (got {})",
                                    identifier_type.type_.stringify_native()
                                ).as_ref()
                            );

                            SkyeValue::get_unknown()
                        }
                    }
                    _ => {
                        ast_error!(
                            self, identifier_expr,
                            format!(
                                "Expecting struct, struct template, union, or bitfield type as compound literal identifier (got {})",
                                identifier_type.type_.stringify_native()
                            ).as_ref()
                        );

                        SkyeValue::get_unknown()
                    }
                }
            }
            Expression::Subscript { subscripted: subscripted_expr, paren, args: arguments } => {
                let subscripted = ctx.run(|ctx| self.evaluate(&subscripted_expr, index, allow_unknown, ctx)).await;

                let new_subscripted = subscripted.follow_reference(self.external_zero_check(paren, index));

                match new_subscripted.type_ {
                    SkyeType::Pointer(inner_type, is_const, is_reference) => {
                        assert!(!is_reference); // if the references were followed correctly, this cannot be a reference

                        if arguments.len() != 1 {
                            token_error!(self, paren, "Expecting one subscript argument for pointer offset");
                            return SkyeValue::special(*inner_type.clone());
                        }

                        let arg = ctx.run(|ctx| self.evaluate(&arguments[0], index, allow_unknown, ctx)).await;

                        match arg.type_ {
                            SkyeType::U8  | SkyeType::I8  | SkyeType::U16 | SkyeType::I16 |
                            SkyeType::U32 | SkyeType::I32 | SkyeType::U64 | SkyeType::I64 |
                            SkyeType::Usz | SkyeType::AnyInt => {
                                let subscripted_value = ctx.run(|ctx| self.zero_check(&subscripted, paren, "Null pointer dereference", index, ctx)).await;
                                return SkyeValue::new(Rc::from(format!("{}[{}]", subscripted_value, arg.value)), *inner_type.clone(), is_const);
                            }
                            _ => {
                                ast_error!(
                                    self, &arguments[0],
                                    format!(
                                        "Expecting integer for subscripting operation (got {})",
                                        arg.type_.stringify_native()
                                    ).as_ref()
                                );

                                return SkyeValue::special(*inner_type.clone());
                           }
                        }
                    }
                    SkyeType::Template(name, definition, generics, generics_names, curr_name, read_env) => {
                        if arguments.len() != generics.len() {
                            let mut needed_cnt = 0;
                            for generic in &generics {
                                if generic.default.is_none() {
                                    needed_cnt += 1;
                                }
                            }

                            if arguments.len() < needed_cnt || arguments.len() > generics.len() {
                                ast_error!(
                                    self, expr,
                                    format!(
                                        "Expecting at least {} generic arguments and {} at most but got {}",
                                        needed_cnt, generics.len(), arguments.len()
                                    ).as_str()
                                );

                                return SkyeValue::get_unknown();
                            }
                        }

                        let offs = {
                            if generics.len() > 1 && generics.first().unwrap().default.is_some() && generics.last().unwrap().default.is_none() {
                                generics.len() - arguments.len()
                            } else {
                                0
                            }
                        };

                        let tmp_env = Rc::new(RefCell::new(
                            Environment::with_enclosing(Rc::clone(&read_env))
                        ));

                        for (i, generic) in generics.iter().enumerate() {
                            let evaluated = {
                                if i >= offs && i - offs < arguments.len() {
                                    ctx.run(|ctx| self.evaluate(&arguments[i - offs], index, allow_unknown, ctx)).await.type_
                                } else {
                                    let previous = Rc::clone(&self.environment);
                                    self.environment = Rc::clone(&tmp_env);

                                    let ret = ctx.run(|ctx| self.evaluate(generic.default.as_ref().unwrap(), index, allow_unknown, ctx)).await;

                                    self.environment = previous;

                                    ret.type_
                                }
                            };

                            match &evaluated {
                                SkyeType::Type(_) | SkyeType::Void | SkyeType::Unknown(_) => (),
                                _ => {
                                    ast_error!(
                                        self, arguments[i - offs],
                                        format!(
                                            "Expecting type as generic type (got {})",
                                            evaluated.stringify_native()
                                        ).as_ref()
                                    );

                                    continue;
                                }
                            }

                            if !evaluated.check_completeness() {
                                ast_error!(self, arguments[i - offs], "Cannot use incomplete type directly");
                                ast_note!(arguments[i - offs], "Define this type or reference it through a pointer");
                            }

                            if !evaluated.can_be_instantiated(true) {
                                ast_error!(self, arguments[i - offs], format!("Cannot instantiate type {}", evaluated.stringify_native()).as_ref());
                            }

                            if let Some(bounds) = &generic.bounds {
                                let previous = Rc::clone(&self.environment);
                                self.environment = Rc::clone(&tmp_env);

                                let evaluated_bound = ctx.run(|ctx| self.evaluate(&bounds, index, false, ctx)).await;

                                self.environment = previous;

                                if evaluated_bound.type_.is_type() || matches!(evaluated_bound.type_, SkyeType::Void) {
                                    if !evaluated_bound.type_.is_respected_by(&evaluated) {
                                        ast_error!(
                                            self, arguments[i - offs],
                                            format!(
                                                "Generic bound is not respected by this type (expecting {} but got {})",
                                                evaluated_bound.type_.stringify_native(), evaluated.stringify_native()
                                            ).as_ref()
                                        );

                                        token_note!(generic.name, "Generic defined here");
                                    }
                                } else {
                                    ast_error!(
                                        self, bounds,
                                        format!(
                                            "Expecting type or group as generic bound (got {})",
                                            evaluated_bound.type_.stringify_native()
                                        ).as_ref()
                                    );
                                }
                            }

                            let mut env = tmp_env.borrow_mut();
                            env.define(
                                Rc::clone(&generic.name.lexeme),
                                SkyeVariable::new(
                                    evaluated, true,
                                    Some(Box::new(generic.name.clone()))
                                )
                            );
                        }

                        let final_name = self.get_generics(&name, &generics_names, &tmp_env);
                        let search_tok = Token::dummy(Rc::clone(&final_name));

                        let mut env = self.globals.borrow_mut();
                        if let Some(var) = env.get(&search_tok) {
                            if let SkyeType::Function(.., has_body) = var.type_ {
                                if has_body {
                                    env = tmp_env.borrow_mut();

                                    for generic in generics {
                                        env.undef(Rc::clone(&generic.name.lexeme));
                                    }

                                    if let Some(self_info) = subscripted.self_info {
                                        return SkyeValue::with_self_info(final_name, var.type_, var.is_const, self_info);
                                    } else {
                                        return SkyeValue::new(final_name, var.type_, var.is_const);
                                    }
                                }
                            } else {
                                env = tmp_env.borrow_mut();

                                for generic in generics {
                                    env.undef(Rc::clone(&generic.name.lexeme));
                                }

                                if let Some(self_info) = subscripted.self_info {
                                    return SkyeValue::with_self_info(final_name, var.type_, var.is_const, self_info);
                                } else {
                                    return SkyeValue::new(final_name, var.type_, var.is_const);
                                }
                            }
                        }

                        drop(env);

                        let previous = Rc::clone(&self.environment);
                        self.environment = Rc::clone(&tmp_env);

                        let previous_name = self.curr_name.clone();
                        self.curr_name = curr_name;

                        let type_ = {
                            match ctx.run(|ctx| self.execute(&definition, 0, ctx)).await {
                                Ok(item) => item.unwrap_or_else(|| {
                                    ast_error!(self, expr, "Could not process template generation for this expression");
                                    SkyeType::get_unknown()
                                }),
                                Err(_) => unreachable!("execution interrupt happened out of context")
                            }
                        };

                        self.curr_name   = previous_name;
                        self.environment = previous;

                        env = tmp_env.borrow_mut();
                        for generic in generics {
                            env.undef(Rc::clone(&generic.name.lexeme));
                        }

                        env.define(
                            Rc::clone(&final_name),
                            SkyeVariable::new(
                                type_.clone(), true, None
                            )
                        );

                        if let Some(self_info) = subscripted.self_info {
                            SkyeValue::with_self_info(final_name, type_, true, self_info)
                        } else {
                            SkyeValue::new(final_name, type_, true)
                        }
                    }
                    _ => {
                        match new_subscripted.type_.implements_op(Operator::Subscript) {
                            ImplementsHow::Native(_) => SkyeValue::get_unknown(), // covers type any, for errors
                            ImplementsHow::ThirdParty => {
                                let search_tok = {
                                    if new_subscripted.is_const {
                                        Token::dummy(Rc::from("__constsubscript__"))
                                    } else {
                                        Token::dummy(Rc::from("__subscript__"))
                                    }
                                };

                                if let Some(value) = self.get_method(&new_subscripted, &search_tok, true, index) {
                                    let call_value = ctx.run(|ctx| self.call(&value, expr, &subscripted_expr, &arguments, index, allow_unknown, ctx)).await;

                                    if let SkyeType::Pointer(ref inner_type, is_const, _) = call_value.type_ {
                                        let call_value_value = ctx.run(|ctx| self.zero_check(&call_value, paren, "Null pointer dereference", index, ctx)).await;
                                        SkyeValue::new(Rc::from(format!("*{}", call_value_value).as_ref()), *inner_type.clone(), is_const)
                                    } else {
                                        ast_error!(
                                            self, subscripted_expr,
                                            format!(
                                                "Expecting pointer as return type of {} (got {})",
                                                search_tok.lexeme, call_value.type_.stringify_native()
                                            ).as_ref()
                                        );

                                        SkyeValue::get_unknown()
                                    }
                                } else {
                                    let search_tok = {
                                        if new_subscripted.is_const {
                                            Token::dummy(Rc::from("__subscript__"))
                                        } else {
                                            Token::dummy(Rc::from("__constsubscript__"))
                                        }
                                    };

                                    if let Some(value) = self.get_method(&new_subscripted, &search_tok, true, index) {
                                        let call_value = ctx.run(|ctx| self.call(&value, expr, &subscripted_expr, &arguments, index, allow_unknown, ctx)).await;

                                        if let SkyeType::Pointer(ref inner_type, is_const, _) = call_value.type_ {
                                            let call_value_value = ctx.run(|ctx| self.zero_check(&call_value, paren, "Null pointer dereference", index, ctx)).await;
                                            SkyeValue::new(Rc::from(format!("*{}", call_value_value).as_ref()), *inner_type.clone(), is_const)
                                        } else {
                                            ast_error!(
                                                self, subscripted_expr,
                                                format!(
                                                    "Expecting pointer as return type of {} (got {})",
                                                    search_tok.lexeme, call_value.type_.stringify_native()
                                                ).as_ref()
                                            );

                                            SkyeValue::get_unknown()
                                        }
                                    } else {
                                        ast_error!(
                                            self, subscripted_expr,
                                            format!(
                                                "Subscripting operation is not implemented for type {}",
                                                new_subscripted.type_.stringify_native()
                                            ).as_ref()
                                        );

                                        SkyeValue::get_unknown()
                                    }
                                }
                            }
                            ImplementsHow::No => {
                                ast_error!(
                                    self, subscripted_expr,
                                    format!(
                                        "Type {} cannot be subscripted",
                                        new_subscripted.type_.stringify_native()
                                    ).as_ref()
                                );

                                SkyeValue::get_unknown()
                            }
                        }
                    }
                }
            }
            Expression::Get(object_expr, name) => {
                let object = ctx.run(|ctx| self.evaluate(&object_expr, index, allow_unknown, ctx)).await;

                match object.type_.get(&object.value, name, object.is_const, self.external_zero_check(name, index)) {
                    GetResult::Ok(value, type_, is_const) => {
                        return SkyeValue::new(value, type_, is_const)
                    }
                    GetResult::InvalidType => {
                        ast_error!(
                            self, object_expr,
                            format!(
                                "Can only get properties from structs and sum type enums (got {})",
                                object.type_.stringify_native()
                            ).as_ref()
                        );
                    }
                    GetResult::FieldNotFound => {
                        if let Some(value) = self.get_method(&object, name, false, index) {
                            return value;
                        } else {
                            token_error!(self, name, format!("Undefined property \"{}\"", name.lexeme).as_ref());
                        }
                    }
                }

                SkyeValue::get_unknown()
            }
            Expression::StaticGet(object_expr, name, gets_macro) => {
                let object = ctx.run(|ctx| self.evaluate(&object_expr, index, allow_unknown, ctx)).await;

                match object.type_.static_get(name) {
                    GetResult::Ok(value, ..) => {
                        let mut search_tok = name.clone();
                        search_tok.set_lexeme(&value);

                        let env = self.globals.borrow();

                        if let Some(var) = env.get(&search_tok) {
                            if *gets_macro {
                                drop(env);

                                let mut operator_token = name.clone();
                                operator_token.set_type(TokenType::At);

                                let output_expr = Expression::Unary { op: operator_token, expr: Box::new(Expression::Variable(search_tok)), is_prefix: true };
                                return ctx.run(|ctx| self.evaluate(&output_expr, index, allow_unknown, ctx)).await;
                            } else {
                                return SkyeValue::new(value, var.type_, var.is_const);
                            }
                        } else if let SkyeType::Type(inner_type) = &object.type_ {
                            if let SkyeType::Enum(enum_name, ..) = &**inner_type {
                                search_tok.set_lexeme(format!("{}_DOT_{}", enum_name, name.lexeme).as_ref());

                                if let Some(var) = env.get(&search_tok) {
                                    return SkyeValue::new(search_tok.lexeme, var.type_, var.is_const)
                                } else {
                                    token_error!(self, name, "Undefined property");
                                }
                            } else {
                                token_error!(self, name, "Undefined property");
                            }
                        } else {
                            token_error!(self, name, "Undefined property");
                        }
                    }
                    GetResult::InvalidType => {
                        ast_error!(
                            self, object_expr,
                            format!(
                                "Can only statically access namespaces, structs, enums and instances (got {})",
                                object.type_.stringify_native()
                            ).as_ref()
                        );
                    }
                    _ => unreachable!()
                }

                SkyeValue::get_unknown()
            }
        }
    }

    async fn handle_deferred(&mut self, index: usize, ctx: &mut reblessive::Stk) {
        let deferred = self.deferred.borrow();
        if let Some(statements) = deferred.last() {
            let cloned = statements.clone();
            drop(deferred);

            for statement in cloned.iter().rev() {
                let _ = ctx.run(|ctx| self.execute(&statement, index, ctx)).await;
            }
        }
    }

    async fn handle_destructors<T: Ast>(&mut self, index: usize, global: bool, ast_item: &T, msg: &str, ctx: &mut reblessive::Stk) -> Result<Option<SkyeType>, ExecutionInterrupt> {
        if !global {
            let vars = self.environment.borrow().iter_local();

            for (name, var) in vars {
                if matches!(var.type_, SkyeType::Struct(..) | SkyeType::Enum(..)) {
                    let search_tok = Token::dummy(Rc::from("__destruct__"));
                    let var_value = SkyeValue::new(Rc::clone(&name), var.type_.clone(), var.is_const);

                    if let Some(value) = self.get_method(&var_value, &search_tok, true, index) {
                        let fake_expr = Expression::Variable(search_tok);
                        let v = Vec::new();

                        let call = ctx.run(|ctx| self.call(&value, &fake_expr, &fake_expr, &v, index, false, ctx)).await;

                        ast_info!(ast_item, format!("Skye inserted a destructor call for \"{}\" {}", name, msg).as_ref()); // +I-destructors

                        self.definitions[index].push_indent();
                        self.definitions[index].push(&call.value);
                        self.definitions[index].push(";\n");
                    }
                }
            }
        }

        Ok(None)
    }

    async fn handle_all_deferred<T: Ast>(&mut self, index: usize, global: bool, ast_item: &T, msg: &str, ctx: &mut reblessive::Stk) {
        let deferred = self.deferred.borrow().clone();

        for statements in deferred.iter().rev() {
            for statement in statements.iter().rev() {
                let _ = ctx.run(|ctx| self.execute(&statement, index, ctx)).await;
            }
        }

        let _ = ctx.run(|ctx| self.handle_destructors(index, global, ast_item, msg, ctx)).await;
    }

    async fn execute_block(&mut self, statements: &Vec<Statement>, environment: Rc<RefCell<Environment>>, index: usize, global: bool, ctx: &mut reblessive::Stk) {
        let previous = Rc::clone(&self.environment);
        self.environment = environment;

        self.deferred.borrow_mut().push(Vec::new());

        let mut destructors_called = false;
        for (i, statement) in statements.iter().enumerate() {
            if let Err(interrupt) = ctx.run(|ctx| self.execute(statement, index, ctx)).await {
                match interrupt {
                    ExecutionInterrupt::Interrupt(output) => {
                        ctx.run(|ctx| self.handle_deferred(index, ctx)).await;
                        let _ = ctx.run(|ctx| self.handle_destructors(index, global, statement, "before this statement", ctx)).await;
                        destructors_called = true;

                        self.definitions[index].push_indent();
                        self.definitions[index].push(&output);

                        if i != statements.len() - 1 {
                            ast_warning!(statements[i + 1], "Unreachable code");
                            break;
                        }
                    }
                    ExecutionInterrupt::Return(output) => {
                        ctx.run(|ctx| self.handle_all_deferred(index, global, statement, "before this statement", ctx)).await;
                        destructors_called = true;

                        self.definitions[index].push_indent();
                        self.definitions[index].push(&output);

                        if i != statements.len() - 1 {
                            ast_warning!(statements[i + 1], "Unreachable code");
                            break;
                        }
                    }
                }
            }
        }

        if statements.len() != 0 && !destructors_called {
            ctx.run(|ctx| self.handle_deferred(index, ctx)).await;
            let _ = ctx.run(|ctx| self.handle_destructors(index, global, statements.last().unwrap(), "after this statement", ctx)).await;
        }

        self.deferred.borrow_mut().pop();
        self.environment = previous;
    }

    pub async fn execute(&mut self, stmt: &Statement, index: usize, ctx: &mut reblessive::Stk) -> Result<Option<SkyeType>, ExecutionInterrupt> {
        match stmt {
            Statement::Empty => (),
            Statement::TransparentBlock(statements) => {
                for statement in statements {
                    ctx.run(|ctx| self.execute(&statement, index, ctx)).await?;
                }
            }
            Statement::Expression(expr) => {
                if matches!(self.curr_function, CurrentFn::None) {
                    ast_error!(self, expr, "Only declarations are allowed at top level");
                    ast_note!(expr, "Place this expression inside a function");
                }

                let value = ctx.run(|ctx| self.evaluate(&expr, index, false, ctx)).await;

                if !value.type_.can_be_instantiated(true) {
                    ast_error!(self, expr, "Cannot use compile-time type as a standalone expression");
                    ast_note!(
                        expr,
                        format!(
                            "This expression has type {}",
                            value.type_.stringify_native()
                        ).as_str()
                    );
                }

                if let SkyeType::Enum(.., base_name) = value.type_ {
                    if base_name.as_ref() == "core_DOT_Result" {
                        ast_warning!(expr, "Error is being ignored implictly");
                        ast_note!(expr, "Handle this error or discard it using the \"let _ = x\" syntax");
                    }
                }

                if value.value.as_ref() != "" { // can happen with expressions returning void
                    self.definitions[index].push_indent();
                    self.definitions[index].push(&value.value);
                    self.definitions[index].push(";\n");
                }
            }
            Statement::VarDecl { name, initializer, type_: type_spec_expr, is_const, qualifiers } => {
                let value = {
                    if let Some(init) = initializer {
                        Some(ctx.run(|ctx| self.evaluate(init, index, false, ctx)).await)
                    } else {
                        None
                    }
                };

                let type_spec = {
                    if let Some(type_) = type_spec_expr {
                        let type_spec_evaluated = ctx.run(|ctx| self.evaluate(type_, index, false, ctx)).await;

                        match type_spec_evaluated.type_ {
                            SkyeType::Type(inner_type) => {
                                if inner_type.check_completeness() {
                                    if inner_type.can_be_instantiated(false) {
                                        Some(*inner_type)
                                    } else {
                                        ast_error!(self, type_, format!("Cannot instantiate type {}", inner_type.stringify_native()).as_ref());
                                        Some(SkyeType::get_unknown())
                                    }
                                } else {
                                    ast_error!(self, type_, "Cannot use incomplete type directly");
                                    ast_note!(type_, "Define this type or reference it through a pointer");
                                    Some(SkyeType::get_unknown())
                                }
                            }
                            SkyeType::Group(..) => {
                                ast_error!(self, type_, "Cannot use type group for variable declaration");
                                Some(SkyeType::get_unknown())
                            }
                            _ => {
                                ast_error!(
                                    self, type_,
                                    format!(
                                        "Invalid expression as type specifier (expecting type but got {})",
                                        type_spec_evaluated.type_.stringify_native()
                                    ).as_ref()
                                );

                                Some(SkyeType::get_unknown())
                            }
                        }
                    } else {
                        None
                    }
                };

                if value.is_none() && type_spec.is_none() {
                    token_error!(self, name, "Variable declaration without initializer needs a type specifier");
                    token_note!(name, "Add a type specifier after the variable name");
                    return Ok(None);
                }

                if value.is_some() && type_spec.is_some() && !type_spec.as_ref().unwrap().equals(&value.as_ref().unwrap().type_, EqualsLevel::Strict) {
                    ast_error!(
                        self, initializer.as_ref().unwrap(),
                        format!(
                            "Initializer type ({}) does not match declared type ({})",
                            value.as_ref().unwrap().type_.stringify_native(),
                            type_spec.as_ref().unwrap().stringify_native()
                        ).as_ref()
                    );

                    ast_note!(initializer.as_ref().unwrap(), "Is this expression correct?");
                    ast_note!(type_spec_expr.as_ref().unwrap(), "If the initializer is correct, consider changing or removing the type specifier");
                }

                let type_ = {
                    if let Some(type_spec_) = type_spec {
                        type_spec_
                    } else {
                        value.as_ref().unwrap().type_.finalize()
                    }
                };

                let type_stringified = type_.stringify();
                if type_stringified.len() == 0 {
                    if type_spec_expr.is_some() {
                        ast_error!(
                            self, type_spec_expr.as_ref().unwrap(),
                            format!(
                                "Invalid expression as type specifier (expecting type but got {})",
                                type_.stringify_native()
                            ).as_ref()
                        );
                    } else {
                        ast_error!(
                            self, initializer.as_ref().unwrap(),
                            format!(
                                "The type of this expression ({}) cannot be assigned to a variable",
                                type_.stringify_native()
                            ).as_ref()
                        );
                    }
                }

                if !type_.can_be_instantiated(false) {
                    if let Some(expr) = type_spec_expr {
                        ast_error!(self, expr, format!("Cannot instantiate type {}", type_.stringify_native()).as_ref());
                    } else if let Some(expr) = initializer {
                        ast_error!(self, expr, format!("Cannot instantiate type {}", type_.stringify_native()).as_ref());
                    }
                }

                let is_global = matches!(self.curr_function, CurrentFn::None);
                let is_discard = name.lexeme.as_ref() == "_";

                if is_discard {
                    if let Some(init) = initializer {
                        if is_global {
                            ast_error!(self, init, "Cannot discard a value in the global scope");
                            ast_note!(init, "Move the statement inside a function");
                        }
                    } else {
                        token_error!(self, name, "Cannot use this name for variable declaration");
                        token_note!(name, "Rename this variable");
                    }

                    self.definitions[index].push_indent();
                    self.definitions[index].push(&value.as_ref().unwrap().value);
                    self.definitions[index].push(";\n");
                } else {
                    let full_name = {
                        if is_global {
                            self.get_name(&name.lexeme)
                        } else {
                            Rc::clone(&name.lexeme)
                        }
                    };

                    let mut buf = String::new();

                    for qualifier in qualifiers {
                        buf.push_str(&qualifier.lexeme);
                        buf.push(' ');
                    }

                    buf.push_str(&type_stringified);
                    buf.push(' ');
                    buf.push_str(&full_name);

                    if value.is_some() {
                        buf.push_str(" = ");
                        buf.push_str(&value.as_ref().unwrap().value);
                    }

                    buf.push_str(";\n");

                    if is_global {
                        if *is_const {
                            token_error!(self, name, "Global constants are not allowed");
                            token_note!(name, "If you want to create a compile-time constant, use a macro");
                        } else if let Some(init) = initializer {
                            ast_error!(self, init, "Cannot assign a value to a global variable directly");
                            ast_note!(init, "Remove the initializer and assign this value through a function");
                        }

                        self.declarations.push(CodeOutput::new());
                        self.declarations.last_mut().unwrap().push_indent();
                        self.declarations.last_mut().unwrap().push(&buf);
                    } else {
                        self.definitions[index].push_indent();
                        self.definitions[index].push(&buf);
                    }

                    let mut env = self.environment.borrow_mut();

                    if let Some(var) = env.get_in_scope(&Token::dummy(Rc::clone(&full_name))) {
                        token_error!(self, name, "Cannot declare variable with same name as existing symbol defined in the same scope");

                        if let Some(token) = &var.tok {
                            token_note!(*token, "Previously defined here");
                        }
                    }

                    env.define(
                        Rc::clone(&full_name), SkyeVariable::new(
                            type_, *is_const,
                            Some(Box::new(name.clone()))
                        )
                    );
                }
            }
            Statement::Block(kw, statements) => {
                if matches!(self.curr_function, CurrentFn::None) {
                    token_error!(self, kw, "Only declarations are allowed at top level");
                    token_note!(kw, "Place this block inside a function");
                }

                self.definitions[index].push_indent();

                if statements.len() == 0 {
                    self.definitions[index].push("{}\n");
                } else {
                    self.definitions[index].push("{\n");
                    self.definitions[index].inc_indent();

                    ctx.run(|ctx| self.execute_block(
                        statements,
                        Rc::new(RefCell::new(
                            Environment::with_enclosing(
                                Rc::clone(&self.environment)
                            )
                        )),
                        index, false, ctx
                    )).await;

                    self.definitions[index].dec_indent();
                    self.definitions[index].push_indent();
                    self.definitions[index].push("}\n");
                }
            }
            Statement::Function { name, params, return_type: return_type_expr, body, qualifiers, generics_names: generics, bind, init } => {
                let mut full_name = self.get_generics(&self.get_name(&name.lexeme), generics, &self.environment);

                let env = self.globals.borrow();
                let search_tok = Token::dummy(Rc::clone(&full_name));
                let existing = env.get(&search_tok);

                let has_decl = {
                    if let Some(var) = &existing {
                        if let SkyeType::Function(.., has_body) = var.type_ {
                            if has_body && body.is_some() {
                                token_error!(self, name, "Cannot redeclare functions");

                                if let Some(token) = &var.tok {
                                    token_note!(*token, "Previously defined here");
                                }

                                false
                            } else {
                                true
                            }
                        } else {
                            token_error!(self, name, "Cannot declare function with same name as existing symbol");

                            if let Some(token) = &var.tok {
                                token_note!(*token, "Previously defined here");
                            }

                            false
                        }
                    } else {
                        false
                    }
                };

                drop(env);

                let return_type = ctx.run(|ctx| self.get_return_type(return_type_expr, index, false, ctx)).await;

                if has_decl {
                    if let SkyeType::Function(_, existing_return_type, _) = &existing.as_ref().unwrap().type_ {
                        if !existing_return_type.equals(&return_type, EqualsLevel::Typewise) {
                            ast_error!(
                                self, return_type_expr,
                                format!(
                                    "Function return type ({}) does not match declaration return type ({})",
                                    return_type.stringify_native(), existing_return_type.stringify_native()
                                ).as_ref()
                            );
                        }
                    }
                }

                let return_stringified = return_type.stringify();
                let (params_string, params_output) = ctx.run(|ctx| self.get_params(params, existing, has_decl, index, false, ctx)).await;
                let type_ = SkyeType::Function(params_output.clone(), Box::new(return_type.clone()), body.is_some());
                self.generate_fn_signature(name, &type_, &return_stringified, &params_string);

                let has_body = body.is_some();

                if *init {
                    if params.len() != 0 {
                        token_error!(self, name, "#init function must take no parameters");
                    }

                    if !has_body {
                        token_error!(self, name, "#init function must have a body");
                    }

                    if full_name.as_ref() == "main" {
                        token_error!(self, name, "\"main\" function cannot be #init");
                    }

                    self.definitions[INIT_DEF_INDEX].push_indent();
                    self.definitions[INIT_DEF_INDEX].push(&full_name);
                    self.definitions[INIT_DEF_INDEX].push("();\n");
                }

                // main function handling
                if has_body && full_name.as_ref() == "main" {
                    if *bind {
                        token_error!(self, name, "Cannot bind \"main\" function");
                    }

                    let returns_void        = return_stringified == "void";
                    let returns_i32         = return_stringified == "i32";
                    let returns_i32_result  = return_stringified == "core_DOT_Result_GENOF_void_GENAND_i32_GENEND_";
                    let returns_void_result = return_stringified == "core_DOT_Result_GENOF_void_GENAND_void_GENEND_";

                    let has_stdargs = {
                        params_output.len() == 2 &&
                        params_output[0].type_.equals(&SkyeType::AnyInt, EqualsLevel::Typewise) &&
                        params_output[1].type_.equals(&SkyeType::Pointer(
                            Box::new(SkyeType::Pointer(
                                Box::new(SkyeType::Char),
                                false, false
                            )),
                            false, false
                        ), EqualsLevel::Typewise)
                    };

                    let has_args = {
                        params_output.len() == 1 &&
                        {
                            if let SkyeType::Struct(.., base_name) = &params_output[0].type_ {
                                base_name.as_ref() == "core_DOT_Array"
                            } else {
                                false
                            }
                        }
                    };

                    let no_args = params_output.len() == 0;

                    if (returns_void || returns_i32 || returns_i32_result || returns_void_result) && (no_args || has_args || has_stdargs) {
                        full_name = Rc::from("_SKYE_MAIN");

                        let real_main_idx = self.definitions.len();
                        self.definitions.push(CodeOutput::new());

                        if returns_void {
                            if has_stdargs {
                                self.definitions[real_main_idx].push(VOID_MAIN_PLUS_STD_ARGS);
                            } else if has_args {
                                self.definitions[real_main_idx].push(VOID_MAIN_PLUS_ARGS);
                            } else {
                                self.definitions[real_main_idx].push(VOID_MAIN);
                            }
                        } else if returns_i32 {
                            if has_stdargs {
                                self.definitions[real_main_idx].push(I32_MAIN_PLUS_STD_ARGS);
                            } else if has_args {
                                self.definitions[real_main_idx].push(I32_MAIN_PLUS_ARGS);
                            } else {
                                self.definitions[real_main_idx].push(I32_MAIN);
                            }
                        } else if returns_i32_result {
                            if has_stdargs {
                                self.definitions[real_main_idx].push(RESULT_I32_MAIN_PLUS_STD_ARGS);
                            } else if has_args {
                                self.definitions[real_main_idx].push(RESULT_I32_MAIN_PLUS_ARGS);
                            } else {
                                self.definitions[real_main_idx].push(RESULT_I32_MAIN);
                            }
                        } else if returns_void_result {
                            if has_stdargs {
                                self.definitions[real_main_idx].push(RESULT_VOID_MAIN_PLUS_STD_ARGS);
                            } else if has_args {
                                self.definitions[real_main_idx].push(RESULT_VOID_MAIN_PLUS_ARGS);
                            } else {
                                self.definitions[real_main_idx].push(RESULT_VOID_MAIN);
                            }
                        }
                    } else {
                        token_error!(self, name, "Invalid function signature for \"main\" function");
                    }
                }

                let mut buf = String::new();

                if !*bind {
                    for qualifier in qualifiers {
                        buf.push_str(&qualifier.lexeme);
                        buf.push(' ');
                    }

                    buf.push_str(&return_stringified);
                    buf.push(' ');
                    buf.push_str(&full_name);
                    buf.push('(');
                    buf.push_str(&params_string);
                    buf.push(')');

                    if (!has_decl) || (!has_body) {
                        self.declarations.push(CodeOutput::new());
                        self.declarations.last_mut().unwrap().push(&buf);
                        self.declarations.last_mut().unwrap().push(";\n");
                    }
                }

                let mut fn_environment = None;
                if has_body {
                    fn_environment = Some(Environment::with_enclosing(Rc::clone(&self.environment)));

                    for i in 0 .. params.len() {
                        fn_environment.as_mut().unwrap().define(
                            Rc::clone(&params[i].name.as_ref().unwrap().lexeme),
                            SkyeVariable::new(
                                params_output[i].type_.clone(),
                                params_output[i].is_const,
                                Some(Box::new(params[i].name.as_ref().unwrap().clone()))
                            )
                        );
                    }
                }

                let mut env = self.globals.borrow_mut();
                env.define(
                    Rc::clone(&full_name), SkyeVariable::new(
                        type_.clone(), true,
                        Some(Box::new(name.clone()))
                    )
                );
                drop(env);

                if has_body {
                    let enclosing_level = self.curr_function.clone();
                    self.curr_function = CurrentFn::Some(return_type, return_type_expr.clone());

                    let enclosing_deferred = Rc::clone(&self.deferred);
                    self.deferred = Rc::new(RefCell::new(Vec::new()));

                    let new_index = self.definitions.len();
                    self.definitions.push(CodeOutput::new());
                    self.definitions[new_index].push(&buf);
                    self.definitions[new_index].push(" {\n");
                    self.definitions[new_index].inc_indent();

                    ctx.run(|ctx| self.execute_block(
                        body.as_ref().unwrap(),
                        Rc::new(RefCell::new(fn_environment.unwrap())),
                        new_index, false, ctx
                    )).await;

                    self.curr_function = enclosing_level;
                    self.deferred = enclosing_deferred;

                    self.definitions[new_index].dec_indent();
                    self.definitions[new_index].push_indent();
                    self.definitions[new_index].push("}\n\n");
                }

                return Ok(Some(type_));
            }
            Statement::If { kw, condition: cond_expr, then_branch, else_branch } => {
                if matches!(self.curr_function, CurrentFn::None) {
                    token_error!(self, kw, "Only declarations are allowed at top level");
                    token_note!(kw, "Place this if statement inside a function");
                }

                let cond = ctx.run(|ctx| self.evaluate(cond_expr, index, false, ctx)).await;

                match cond.type_ {
                    SkyeType::U8  | SkyeType::I8  | SkyeType::U16 | SkyeType::I16 |
                    SkyeType::U32 | SkyeType::I32 | SkyeType::U64 | SkyeType::I64 |
                    SkyeType::Usz | SkyeType::AnyInt | SkyeType::Unknown(_) => (),
                    _ => {
                        ast_error!(
                            self, cond_expr,
                            format!(
                                "Expecting expression of primitive arithmetic type for if condition (got {})",
                                cond.type_.stringify_native()
                            ).as_ref()
                        );
                    }
                }

                let not_grouping = !matches!(cond_expr, Expression::Grouping(_));
                let not_block    = !matches!(**then_branch, Statement::Block(..));

                self.definitions[index].push_indent();
                self.definitions[index].push("if ");

                if not_grouping {
                    self.definitions[index].push("(");
                }

                self.definitions[index].push(&cond.value);

                if not_grouping {
                    self.definitions[index].push(")");
                }

                self.definitions[index].push("\n");

                if not_block {
                    self.definitions[index].push_indent();
                    self.definitions[index].push("{\n");
                    self.definitions[index].inc_indent();

                    let stmts = vec![*then_branch.clone()];
                    ctx.run(|ctx| self.execute_block(
                        &stmts,
                        Rc::new(RefCell::new(Environment::with_enclosing(
                            Rc::clone(&self.environment)
                        ))),
                        index, false, ctx
                    )).await;
                } else {
                    let _ = ctx.run(|ctx| self.execute(&then_branch, index, ctx)).await;
                }

                if not_block {
                    self.definitions[index].dec_indent();
                    self.definitions[index].push_indent();
                    self.definitions[index].push("}\n");
                }

                if let Some(else_branch_statement) = else_branch {
                    let not_block = !matches!(**else_branch_statement, Statement::Block(..));

                    self.definitions[index].push_indent();
                    self.definitions[index].push("else\n");

                    if not_block {
                        self.definitions[index].push_indent();
                        self.definitions[index].push("{\n");
                        self.definitions[index].inc_indent();

                        let stmts = vec![*else_branch_statement.clone()];
                        ctx.run(|ctx| self.execute_block(
                            &stmts,
                            Rc::new(RefCell::new(Environment::with_enclosing(
                                Rc::clone(&self.environment)
                            ))),
                            index, false, ctx
                        )).await;
                    } else {
                        let _ = ctx.run(|ctx| self.execute(&else_branch_statement, index, ctx)).await;
                    }

                    if not_block {
                        self.definitions[index].dec_indent();
                        self.definitions[index].push_indent();
                        self.definitions[index].push("}\n");
                    }
                }
            }
            Statement::While { kw, condition: cond_expr, body } => {
                if matches!(self.curr_function, CurrentFn::None) {
                    token_error!(self, kw, "Only declarations are allowed at top level");
                    token_note!(kw, "Place this while loop inside a function");
                }

                let not_grouping = !matches!(cond_expr, Expression::Grouping(_));
                let not_block    = !matches!(**body, Statement::Block(..));

                self.definitions[index].push_indent();
                self.definitions[index].push("while (1) {\n");
                self.definitions[index].inc_indent();

                let cond = ctx.run(|ctx| self.evaluate(cond_expr, index, false, ctx)).await;

                match cond.type_ {
                    SkyeType::U8  | SkyeType::I8  | SkyeType::U16 | SkyeType::I16 |
                    SkyeType::U32 | SkyeType::I32 | SkyeType::U64 | SkyeType::I64 |
                    SkyeType::Usz | SkyeType::AnyInt | SkyeType::Unknown(_) => (),
                    _ => {
                        ast_error!(
                            self, cond_expr,
                            format!(
                                "Expecting expression of primitive arithmetic type for while condition (got {})",
                                cond.type_.stringify_native()
                            ).as_ref()
                        );
                    }
                }

                self.definitions[index].push_indent();
                self.definitions[index].push("if (!");

                if not_grouping {
                    self.definitions[index].push("(");
                }

                self.definitions[index].push(&cond.value);

                if not_grouping {
                    self.definitions[index].push(")");
                }

                self.definitions[index].push(") break;\n");

                let continue_label = Rc::from(self.get_temporary_var());
                let break_label = Rc::from(self.get_temporary_var());

                let previous_loop = self.curr_loop.clone();
                self.curr_loop = Some((Rc::clone(&break_label), Rc::clone(&continue_label)));

                if not_block {
                    let stmts = vec![*body.clone()];
                    ctx.run(|ctx| self.execute_block(
                        &stmts,
                        Rc::new(RefCell::new(Environment::with_enclosing(
                            Rc::clone(&self.environment)
                        ))),
                        index, false, ctx
                    )).await;
                } else {
                    let _ = ctx.run(|ctx| self.execute(&body, index, ctx)).await;
                }

                self.curr_loop = previous_loop;

                self.definitions[index].push_indent();
                self.definitions[index].push(&continue_label);
                self.definitions[index].push(":;\n");

                self.definitions[index].dec_indent();
                self.definitions[index].push_indent();
                self.definitions[index].push("}\n");

                self.definitions[index].push_indent();
                self.definitions[index].push(&break_label);
                self.definitions[index].push(":;\n");
            }
            Statement::For { kw, initializer, condition: cond_expr, increments, body } => {
                if matches!(self.curr_function, CurrentFn::None) {
                    token_error!(self, kw, "Only declarations are allowed at top level");
                    token_note!(kw, "Place this for loop inside a function");
                }

                let not_block    = !matches!(**body, Statement::Block(..));
                let not_grouping = !matches!(cond_expr, Expression::Grouping(_));

                if let Some(init) = initializer {
                    let _ = ctx.run(|ctx| self.execute(&init, index, ctx)).await;
                }

                self.definitions[index].push_indent();
                self.definitions[index].push("while (1) {\n");
                self.definitions[index].inc_indent();

                let cond = ctx.run(|ctx| self.evaluate(cond_expr, index, false, ctx)).await;

                match cond.type_ {
                    SkyeType::U8  | SkyeType::I8  | SkyeType::U16 | SkyeType::I16 |
                    SkyeType::U32 | SkyeType::I32 | SkyeType::U64 | SkyeType::I64 |
                    SkyeType::Usz | SkyeType::AnyInt | SkyeType::Unknown(_) => (),
                    _ => {
                        ast_error!(
                            self, cond_expr,
                            format!(
                                "Expecting expression of primitive arithmetic type for for condition (got {})",
                                cond.type_.stringify_native()
                            ).as_ref()
                        );
                    }
                }

                self.definitions[index].push_indent();
                self.definitions[index].push("if (!");

                if not_grouping {
                    self.definitions[index].push("(");
                }

                self.definitions[index].push(&cond.value);

                if not_grouping {
                    self.definitions[index].push(")");
                }

                self.definitions[index].push(") break;\n");

                let continue_label = Rc::from(self.get_temporary_var());
                let break_label = Rc::from(self.get_temporary_var());

                let previous_loop = self.curr_loop.clone();
                self.curr_loop = Some((Rc::clone(&break_label), Rc::clone(&continue_label)));

                if not_block {
                    let stmts = vec![*body.clone()];
                    ctx.run(|ctx| self.execute_block(
                        &stmts,
                        Rc::new(RefCell::new(Environment::with_enclosing(
                            Rc::clone(&self.environment)
                        ))),
                        index, false, ctx
                    )).await;
                } else {
                    let _ = ctx.run(|ctx| self.execute(&body, index, ctx)).await;
                }

                self.curr_loop = previous_loop;

                self.definitions[index].push_indent();
                self.definitions[index].push(&continue_label);
                self.definitions[index].push(":;\n");

                for increment in increments {
                    let stmt = Statement::Expression(increment.clone());
                    let _ = ctx.run(|ctx| self.execute(&stmt, index, ctx)).await;
                }

                self.definitions[index].dec_indent();
                self.definitions[index].push_indent();
                self.definitions[index].push("}\n");

                self.definitions[index].push_indent();
                self.definitions[index].push(&break_label);
                self.definitions[index].push(":;\n");
            }
            Statement::DoWhile { kw, condition: cond_expr, body } => {
                if matches!(self.curr_function, CurrentFn::None) {
                    token_error!(self, kw, "Only declarations are allowed at top level");
                    token_note!(kw, "Place this do-while loop inside a function");
                }

                let not_grouping = !matches!(cond_expr, Expression::Grouping(_));
                let not_block    = !matches!(**body, Statement::Block(..));

                self.definitions[index].push_indent();
                self.definitions[index].push("while (1) {\n");
                self.definitions[index].inc_indent();

                let continue_label = Rc::from(self.get_temporary_var());
                let break_label = Rc::from(self.get_temporary_var());

                let previous_loop = self.curr_loop.clone();
                self.curr_loop = Some((Rc::clone(&break_label), Rc::clone(&continue_label)));

                if not_block {
                    let stmts = vec![*body.clone()];
                    ctx.run(|ctx| self.execute_block(
                        &stmts,
                        Rc::new(RefCell::new(Environment::with_enclosing(
                            Rc::clone(&self.environment)
                        ))),
                        index, false, ctx
                    )).await;
                } else {
                    let _ = ctx.run(|ctx| self.execute(&body, index, ctx)).await;
                }

                self.curr_loop = previous_loop;

                let cond = ctx.run(|ctx| self.evaluate(&cond_expr, index, false, ctx)).await;

                match cond.type_ {
                    SkyeType::U8  | SkyeType::I8  | SkyeType::U16 | SkyeType::I16 |
                    SkyeType::U32 | SkyeType::I32 | SkyeType::U64 | SkyeType::I64 |
                    SkyeType::Usz | SkyeType::AnyInt | SkyeType::Unknown(_) => (),
                    _ => {
                        ast_error!(
                            self, cond_expr,
                            format!(
                                "Expecting expression of primitive arithmetic type for while condition (got {})",
                                cond.type_.stringify_native()
                            ).as_ref()
                        );
                    }
                }

                self.definitions[index].push_indent();
                self.definitions[index].push(&continue_label);
                self.definitions[index].push(":;\n");

                self.definitions[index].push_indent();
                self.definitions[index].push("if (!");

                if not_grouping {
                    self.definitions[index].push("(");
                }

                self.definitions[index].push(&cond.value);

                if not_grouping {
                    self.definitions[index].push(")");
                }

                self.definitions[index].push(") break;\n");
                self.definitions[index].dec_indent();

                self.definitions[index].push_indent();
                self.definitions[index].push("}\n");

                self.definitions[index].push_indent();
                self.definitions[index].push(&break_label);
                self.definitions[index].push(":;\n");
            }
            Statement::Return { kw, value: ret_expr } => {
                if matches!(self.curr_function, CurrentFn::None) {
                    token_error!(self, kw, "Cannot return from top-level code");
                    token_note!(kw, "Remove this return statement");
                }

                let mut buf = String::new();

                if let Some(expr) = ret_expr {
                    let value = ctx.run(|ctx| self.evaluate(expr, index, false, ctx)).await;

                    let is_void;
                    if let CurrentFn::Some(type_, orig_ret_type) = &self.curr_function {
                        is_void = matches!(type_, SkyeType::Void);

                        if is_void && !matches!(value.type_, SkyeType::Void) {
                            ast_error!(self, expr, "Cannot return value in a function that returns void");
                            ast_note!(expr, "Remove this expression");
                            ast_note!(orig_ret_type, "Return type defined here");
                        } else if !type_.equals(&value.type_, EqualsLevel::Typewise) {
                            ast_error!(
                                self, expr,
                                format!(
                                    "Returned value type ({}) does not match function return type ({})",
                                    value.type_.stringify_native(), type_.stringify_native()
                                ).as_ref()
                            );

                            ast_note!(orig_ret_type, "Return type defined here");
                        }
                    } else {
                        unreachable!();
                    }

                    if is_void {
                        if value.value.as_ref() != "" {
                            self.definitions[index].push_indent();
                            self.definitions[index].push(&value.value);
                            self.definitions[index].push(";\n");
                        }

                        buf.push_str("return;\n");
                    } else {
                        let final_value = {
                            let search_tok = Token::dummy(Rc::from("__copy__"));
                            if let Some(method_value) = self.get_method(&value, &search_tok, true, index) {
                                let v = Vec::new();
                                let copy_constructor = ctx.run(|ctx| self.call(&method_value, expr, &expr, &v, index, false, ctx)).await;

                                ast_info!(expr, "Skye inserted a copy constructor call for this expression"); // +I-copies
                                copy_constructor
                            } else {
                                value
                            }
                        };

                        // return value is saved in a temporary variable so deferred statements get executed after evaluation
                        let tmp_var_name = self.get_temporary_var();

                        self.definitions[index].push_indent();
                        self.definitions[index].push(&final_value.type_.stringify());
                        self.definitions[index].push(" ");
                        self.definitions[index].push(&tmp_var_name);
                        self.definitions[index].push(" = ");
                        self.definitions[index].push(&final_value.value);
                        self.definitions[index].push(";\n");

                        buf.push_str("return ");
                        buf.push_str(&tmp_var_name);
                        buf.push_str(";\n");
                    }

                } else {
                    if let CurrentFn::Some(type_, expr) = &self.curr_function {
                        if !matches!(type_, SkyeType::Void) {
                            token_error!(self, kw, "Cannot return no value in this function");
                            token_note!(kw, "Add a return value");
                            ast_note!(expr, "Return type defined here");
                        }
                    } else {
                        unreachable!();
                    }

                    buf.push_str("return;\n");
                }

                return Err(ExecutionInterrupt::Return(Rc::from(buf.as_ref())))
            }
            Statement::Struct { name, fields, has_body, binding, generics_names: generics, bind_typedefed } => {
                let base_name = self.get_name(&name.lexeme);
                let full_name = self.get_generics(&base_name, generics, &self.environment);

                let env = self.globals.borrow();
                let existing = env.get(
                    &Token::dummy(Rc::clone(&full_name))
                );

                let has_decl = {
                    if let Some(var) = &existing {
                        if let SkyeType::Type(inner_type) = &var.type_ {
                            if let SkyeType::Struct(_, existing_fields, _) = &**inner_type {
                                if *has_body && existing_fields.is_some() {
                                    token_error!(self, name, "Cannot redefine structs");

                                    if let Some(token) = &var.tok {
                                        token_note!(*token, "Previously defined here");
                                    }

                                    false
                                } else {
                                    true
                                }
                            } else {
                                token_error!(self, name, "Cannot declare struct with same name as existing symbol");

                                if let Some(token) = &var.tok {
                                    token_note!(*token, "Previously defined here");
                                }

                                false
                            }
                        } else {
                            token_error!(self, name, "Cannot declare struct with same name as existing symbol");

                            if let Some(token) = &var.tok {
                                token_note!(*token, "Previously defined here");
                            }

                            false
                        }
                    } else {
                        false
                    }
                };

                drop(env);

                let mut buf = String::from("typedef ");
                let mut equal_binding = false;

                if let Some(bound_name) = binding {
                    if !*bind_typedefed {
                        buf.push_str("struct ");
                    } else {
                        equal_binding = bound_name.lexeme == full_name;
                    }

                    buf.push_str(&bound_name.lexeme);
                } else {
                    let base_struct_name = self.get_generics(&name.lexeme, generics, &self.environment);
                    let full_struct_name = self.get_name(&Rc::from(format!("SKYE_STRUCT_{}", base_struct_name)));
                    buf.push_str("struct ");
                    buf.push_str(&full_struct_name);
                }

                buf.push(' ');

                if (!equal_binding) && ((!has_decl) || (!*has_body)) {
                    self.declarations.push(CodeOutput::new());
                    self.declarations.last_mut().unwrap().push(&buf);
                    self.declarations.last_mut().unwrap().push(&full_name);
                    self.declarations.last_mut().unwrap().push(";\n");
                }

                let mut def_buf = CodeOutput::new();

                if *has_body && binding.is_none() {
                    def_buf.push(&buf);
                }

                let type_ = {
                    if *has_body {
                        if binding.is_none() {
                            def_buf.push("{\n");
                            def_buf.inc_indent();
                        }

                        let mut output_fields = HashMap::new();
                        for field in fields {
                            let field_type = {
                                let tmp = ctx.run(|ctx| self.evaluate(&field.expr, index, false, ctx)).await.type_;

                                match tmp {
                                    SkyeType::Type(inner_type) => {
                                        if inner_type.check_completeness() {
                                            *inner_type
                                        } else {
                                            ast_error!(self, field.expr, "Cannot use incomplete type directly");
                                            ast_note!(field.expr, "Define this type or reference it through a pointer");
                                            SkyeType::get_unknown()
                                        }
                                    }
                                    SkyeType::Unknown(_) => tmp,
                                    _ => {
                                        ast_error!(
                                            self, field.expr,
                                            format!(
                                                "Expecting type as field type (got {})",
                                                tmp.stringify_native()
                                            ).as_ref()
                                        );

                                        SkyeType::get_unknown()
                                    }
                                }
                            };

                            if output_fields.contains_key(&field.name.lexeme) {
                                token_error!(self, field.name, "Cannot define the same struct field multiple times");
                            } else {
                                let field_type_stringified = field_type.stringify();
                                output_fields.insert(Rc::clone(&field.name.lexeme), (field_type, field.is_const));

                                if binding.is_none() {
                                    def_buf.push_indent();
                                    def_buf.push(&field_type_stringified);
                                    def_buf.push(" ");
                                    def_buf.push(&field.name.lexeme);
                                    def_buf.push(";\n");
                                }
                            }
                        }

                        if binding.is_none() {
                            def_buf.dec_indent();
                            def_buf.push("} ");
                            def_buf.push(&full_name);
                            def_buf.push(";\n\n");
                        }

                        self.struct_definitions.insert(Rc::clone(&full_name), def_buf);
                        self.struct_defs_order.push(Rc::clone(&full_name));

                        SkyeType::Struct(Rc::clone(&full_name), Some(output_fields), base_name)
                    } else {
                        SkyeType::Struct(Rc::clone(&full_name), None, base_name)
                    }
                };

                let output_type = SkyeType::Type(Box::new(type_));

                let mut env = self.globals.borrow_mut();

                env.define(
                    Rc::clone(&full_name),
                    SkyeVariable::new(
                        output_type.clone(), true,
                        Some(Box::new(name.clone()))
                    )
                );

                return Ok(Some(output_type));
            }
            Statement::Impl { object: struct_expr, declarations: statements } => {
                let struct_name = ctx.run(|ctx| self.evaluate(&struct_expr, index, false, ctx)).await;

                match &struct_name.type_ {
                    SkyeType::Type(inner_type) => {
                        match inner_type.as_ref() {
                            SkyeType::Struct(..) |
                            SkyeType::Enum(..) => {
                                let mut env = self.globals.borrow_mut();
                                env.define(
                                    Rc::from("Self"),
                                    SkyeVariable::new(
                                        struct_name.type_.clone(),
                                        true, None
                                    )
                                );
                                drop(env);

                                let previous_name = self.curr_name.clone();
                                self.curr_name = struct_name.value.to_string();

                                ctx.run(|ctx| self.execute_block(
                                    statements, Rc::clone(&self.globals), index, true, ctx
                                )).await;

                                self.curr_name = previous_name;

                                env = self.globals.borrow_mut();
                                env.undef(Rc::from("Self"));
                            }
                            _ => {
                                ast_error!(
                                    self, struct_expr,
                                    format!(
                                        "Can only implement structs and enums or their templates (got {})",
                                        struct_name.type_.stringify_native()
                                    ).as_ref()
                                );
                            }
                        }
                    }
                    SkyeType::Template(_, definition, ..) => {
                        match definition {
                            Statement::Struct { .. } |
                            Statement::Enum { .. } => {
                                let mut env = self.globals.borrow_mut();
                                env.define(
                                    Rc::from("Self"),
                                    SkyeVariable::new(
                                        struct_name.type_.clone(),
                                        true, None
                                    )
                                );
                                drop(env);

                                let previous_name = self.curr_name.clone();
                                self.curr_name = struct_name.value.to_string();

                                ctx.run(|ctx| self.execute_block(
                                    statements, Rc::clone(&self.globals), index, true, ctx
                                )).await;

                                self.curr_name = previous_name;

                                env = self.globals.borrow_mut();
                                env.undef(Rc::from("Self"));
                            }
                            _ => {
                                ast_error!(
                                    self, struct_expr,
                                    format!(
                                        "Can only implement structs and enums or their templates (got {})",
                                        struct_name.type_.stringify_native()
                                    ).as_ref()
                                );
                            }
                        }
                    }
                    _ => {
                        ast_error!(
                            self, struct_expr,
                            format!(
                                "Can only implement structs and enums or their templates (got {})",
                                struct_name.type_.stringify_native()
                            ).as_ref()
                        );
                    }
                }
            }
            Statement::Namespace { name, body: statements } => {
                if matches!(self.curr_function, CurrentFn::Some(..)) {
                    token_error!(self, name, "Namespaces are only allowed in the global scope");
                }

                let full_name = self.get_name(&name.lexeme);

                let mut env = self.globals.borrow_mut();
                if let Some(var) = env.get(name) {
                    if !matches!(var.type_, SkyeType::Namespace(_)) {
                        token_error!(self, name, "Cannot declare namespace with same name as existing symbol");

                        if let Some(token) = &var.tok {
                            token_note!(*token, "Previously defined here");
                        }

                        return Ok(None);
                    }
                } else {
                    env.define(
                        Rc::clone(&full_name),
                        SkyeVariable::new(
                            SkyeType::Namespace(Rc::clone(&full_name)),
                            true,
                            Some(Box::new(name.clone()))
                        )
                    );
                }

                drop(env);

                if statements.len() == 0 {
                    token_error!(self, name, "Cannot create an empty namespace");
                } else {
                    let previous_name = self.curr_name.clone();
                    self.curr_name = full_name.to_string();

                    ctx.run(|ctx| self.execute_block(
                        statements, Rc::clone(&self.globals), index, true, ctx
                    )).await;

                    self.curr_name = previous_name;
                }
            }
            Statement::Use { use_expr, as_name: identifier, typedef, bind } => {
                let use_value = ctx.run(|ctx| self.evaluate(&use_expr, index, false, ctx)).await;

                if identifier.lexeme.as_ref() != "_" {
                    if use_value.value.len() != 0 && !*bind {
                        let mut buf = String::new();

                        if *typedef {
                            buf.push_str("typedef ");
                            buf.push_str(&use_value.value);
                            buf.push(' ');
                            buf.push_str(&identifier.lexeme);
                            buf.push(';');
                        } else {
                            buf.push_str("#define ");
                            buf.push_str(&identifier.lexeme);
                            buf.push(' ');
                            buf.push_str(&use_value.value);
                        }

                        buf.push('\n');

                        if matches!(self.curr_function, CurrentFn::None) {
                            self.declarations.push(CodeOutput::new());
                            self.declarations.last_mut().unwrap().push(&buf);
                        } else {
                            self.definitions[index].push_indent();
                            self.definitions[index].push(&buf);

                            if !*typedef {
                                self.deferred.borrow_mut().last_mut().unwrap()
                                    .insert(0, Statement::Undef(Rc::clone(&identifier.lexeme)));
                            }
                        }
                    }

                    let mut env = self.environment.borrow_mut();

                    if let Some(existing) = env.get(&identifier) {
                        token_error!(self, identifier, "Cannot redefine identifiers");

                        if let Some(token) = &existing.tok {
                            token_note!(*token, "Previously defined here");
                        }
                    }

                    env.define(
                        Rc::clone(&identifier.lexeme),
                        SkyeVariable::new(
                            use_value.type_, use_value.is_const,
                            Some(Box::new(identifier.clone()))
                        )
                    );
                }
            }
            Statement::Undef(name) => {
                self.definitions[index].push_indent();
                self.definitions[index].push("#undef ");
                self.definitions[index].push(&name);
                self.definitions[index].push("\n");

                let mut env = self.environment.borrow_mut();
                env.undef(Rc::clone(name));
            }
            Statement::Enum { name, kind_type: type_expr, variants, is_simple, has_body, binding, generics_names: generics, bind_typedefed } => {
                let base_name = self.get_name(&name.lexeme);
                let full_name = self.get_generics(&base_name, generics, &self.environment);

                let type_ = {
                    let enum_type = ctx.run(|ctx| self.evaluate(type_expr, index, false, ctx)).await.type_;

                    if let SkyeType::Type(inner_type) = &enum_type {
                        match **inner_type {
                            SkyeType::U8  | SkyeType::I8  | SkyeType::U16 | SkyeType::I16 |
                            SkyeType::U32 | SkyeType::I32 | SkyeType::U64 | SkyeType::I64 |
                            SkyeType::Usz => *inner_type.clone(),
                            _ => {
                                ast_error!(
                                    self, type_expr,
                                    format!(
                                        "Expecting primitive arithmetic type as enum type (got {})",
                                        enum_type.stringify_native()
                                    ).as_ref()
                                );

                                SkyeType::I32
                            }
                        }
                    } else {
                        ast_error!(
                            self, type_expr,
                            format!(
                                "Expecting type as enum type (got {})",
                                enum_type.stringify_native()
                            ).as_ref()
                        );

                        SkyeType::I32
                    }
                };

                let simple_enum_name = {
                    if *is_simple {
                        Rc::clone(&name.lexeme)
                    } else {
                        Rc::from(format!("{}_DOT_Kind", name.lexeme))
                    }
                };

                let simple_enum_full_name = self.get_name(&simple_enum_name);

                let output_type = {
                    if *has_body {
                        let simple_enum_type = SkyeType::Enum(
                            Rc::clone(&simple_enum_full_name), None,
                            Rc::clone(&simple_enum_full_name)
                        );

                        let env = self.globals.borrow();
                        let search_tok = Token::dummy(Rc::clone(&simple_enum_full_name));
                        if let Some(var) = env.get(&search_tok) {
                            drop(env);
                            if generics.len() == 0 {
                                token_error!(self, name, "Cannot redefine enums");

                                if let Some(token) = &var.tok {
                                    token_note!(*token, "Previously defined here");
                                }
                            }
                        } else {
                            if let Some(bound_name) = binding {
                                let mut write = true;
                                if !*bind_typedefed {
                                    self.declarations.push(CodeOutput::new());
                                    self.declarations.last_mut().unwrap().push("typedef enum ");
                                } else {
                                    write = bound_name.lexeme != full_name;

                                    if write {
                                        self.declarations.push(CodeOutput::new());
                                        self.declarations.last_mut().unwrap().push("typedef enum ");
                                    }
                                }

                                if write {
                                    self.declarations.last_mut().unwrap().push(&bound_name.lexeme);
                                    self.declarations.last_mut().unwrap().push(": ");
                                    self.declarations.last_mut().unwrap().push(&type_.stringify());
                                    self.declarations.last_mut().unwrap().push(" ");
                                    self.declarations.last_mut().unwrap().push(&full_name);
                                    self.declarations.last_mut().unwrap().push(";\n");
                                }
                            } else {
                                self.declarations.push(CodeOutput::new());
                                self.declarations.last_mut().unwrap().push("typedef enum ");

                                let full_struct_name = self.get_name(&Rc::from(format!("SKYE_ENUM_{}", simple_enum_name)));
                                self.declarations.last_mut().unwrap().push(&full_struct_name);
                                self.declarations.last_mut().unwrap().push(": ");
                                self.declarations.last_mut().unwrap().push(&type_.stringify());
                                self.declarations.last_mut().unwrap().push(" {\n");
                                self.declarations.last_mut().unwrap().inc_indent();

                                for (i, variant) in variants.iter().enumerate() {
                                    self.declarations.last_mut().unwrap().push_indent();
                                    self.declarations.last_mut().unwrap().push(&simple_enum_full_name);
                                    self.declarations.last_mut().unwrap().push("_DOT_");
                                    self.declarations.last_mut().unwrap().push(&variant.name.lexeme);

                                    if i != variants.len() - 1 {
                                        self.declarations.last_mut().unwrap().push(",\n");
                                    }
                                }

                                self.declarations.last_mut().unwrap().dec_indent();
                                self.declarations.last_mut().unwrap().push("\n} ");
                                self.declarations.last_mut().unwrap().push(&simple_enum_full_name);
                                self.declarations.last_mut().unwrap().push(";\n");
                            }

                            drop(env);
                            let mut env = self.globals.borrow_mut();
                            env.define(
                                Rc::clone(&simple_enum_full_name),
                                SkyeVariable::new(
                                    SkyeType::Type(Box::new(simple_enum_type.clone())),
                                    true, Some(Box::new(name.clone()))
                                )
                            );
                        }

                        let write_output = binding.is_none() && !*is_simple;
                        let base_struct_name = self.get_generics(&name.lexeme, generics, &self.environment);
                        let full_struct_name = self.get_name(&Rc::from(format!("SKYE_STRUCT_{}", base_struct_name)));

                        let mut def_buf = CodeOutput::new();

                        if write_output {
                            let mut buf = String::from("typedef struct ");
                            buf.push_str(&full_struct_name);
                            buf.push(' ');

                            self.declarations.push(CodeOutput::new());
                            self.declarations.last_mut().unwrap().push(&buf);
                            self.declarations.last_mut().unwrap().push(&full_name);
                            self.declarations.last_mut().unwrap().push(";\n");

                            def_buf.push(&buf);
                            def_buf.push("{\n");
                            def_buf.inc_indent();

                            def_buf.push_indent();
                            def_buf.push("union {\n");
                            def_buf.inc_indent();
                        }

                        let mut output_fields = HashMap::new();
                        let mut initializers = CodeOutput::new();
                        let mut evaluated_variants = Vec::with_capacity(variants.len());
                        for variant in variants {
                            let variant_type = {
                                let type_ = ctx.run(|ctx| self.evaluate(&variant.expr, index, false, ctx)).await.type_;
                                match type_ {
                                    SkyeType::Void | SkyeType::Unknown(_) => type_,
                                    SkyeType::Type(inner_type) => {
                                        if inner_type.check_completeness() {
                                            if inner_type.can_be_instantiated(false) {
                                                *inner_type
                                            } else {
                                                ast_error!(self, variant.expr, format!("Cannot instantiate type {}", inner_type.stringify_native()).as_ref());
                                                SkyeType::get_unknown()
                                            }
                                        } else {
                                            ast_error!(self, variant.expr, "Cannot use incomplete type directly");
                                            ast_note!(variant.expr, "Define this type or reference it through a pointer");
                                            SkyeType::get_unknown()
                                        }
                                    }
                                    _ => {
                                        ast_error!(
                                            self, variant.expr,
                                            format!(
                                                "Expecting type as enum variant type (got {})",
                                                type_.stringify_native()
                                            ).as_ref()
                                        );

                                        SkyeType::get_unknown()
                                    }
                                }
                            };

                            evaluated_variants.push(SkyeEnumVariant::new(
                                variant.name.clone(),
                                variant_type.clone())
                            );

                            let mut env = self.globals.borrow_mut();
                            if binding.is_some() {
                                env.define(
                                    Rc::clone(&variant.name.lexeme),
                                    SkyeVariable::new(
                                        simple_enum_type.clone(), true,
                                        Some(Box::new(variant.name.clone()))
                                    )
                                );
                            } else {
                                env.define(
                                    Rc::from(format!("{}_DOT_{}", simple_enum_full_name, variant.name.lexeme)),
                                    SkyeVariable::new(
                                        simple_enum_type.clone(), true,
                                        Some(Box::new(variant.name.clone()))
                                    )
                                );
                            }

                            drop(env);

                            let is_void = matches!(variant_type, SkyeType::Void);
                            let is_not_void = !is_void;

                            if write_output {
                                let mut buf = String::new();
                                buf.push_str(&full_name);
                                buf.push(' ');
                                buf.push_str(&full_name);
                                buf.push_str("_DOT_");

                                if is_void {
                                    buf.push_str("SKYE_ENUM_INIT_");
                                }

                                buf.push_str(&variant.name.lexeme);
                                buf.push_str("(");

                                if is_not_void {
                                    buf.push_str(&variant_type.stringify());
                                    buf.push_str(" value");
                                }

                                buf.push(')');

                                self.declarations.push(CodeOutput::new());
                                self.declarations.last_mut().unwrap().push(&buf);
                                self.declarations.last_mut().unwrap().push(";\n");

                                initializers.push_indent();
                                initializers.push(&buf);
                                initializers.push(" {\n");
                                initializers.inc_indent();

                                initializers.push_indent();
                                initializers.push(&full_name);
                                initializers.push(" tmp;\n");

                                initializers.push_indent();
                                initializers.push("tmp.kind = ");
                                initializers.push(&simple_enum_full_name);
                                initializers.push("_DOT_");
                                initializers.push(&variant.name.lexeme);
                                initializers.push(";\n");

                                if is_not_void {
                                    initializers.push_indent();
                                    initializers.push("tmp.");
                                    initializers.push(&variant.name.lexeme);
                                    initializers.push(" = value;\n");
                                }

                                initializers.push_indent();
                                initializers.push("return tmp;\n");
                                initializers.dec_indent();

                                initializers.push_indent();
                                initializers.push("}\n\n");
                            }

                            if is_void {
                                if write_output {
                                    self.declarations.push(CodeOutput::new());
                                    self.declarations.last_mut().unwrap().push("#define ");
                                    self.declarations.last_mut().unwrap().push(&full_name);
                                    self.declarations.last_mut().unwrap().push("_DOT_");
                                    self.declarations.last_mut().unwrap().push(&variant.name.lexeme);
                                    self.declarations.last_mut().unwrap().push(" ");
                                    self.declarations.last_mut().unwrap().push(&full_name);
                                    self.declarations.last_mut().unwrap().push("_DOT_SKYE_ENUM_INIT_");
                                    self.declarations.last_mut().unwrap().push(&variant.name.lexeme);
                                    self.declarations.last_mut().unwrap().push("()\n");
                                }
                            } else {
                                if write_output {
                                    def_buf.push_indent();
                                    def_buf.push(&variant_type.stringify());
                                    def_buf.push(" ");
                                    def_buf.push(&variant.name.lexeme);
                                    def_buf.push(";\n");
                                }

                                output_fields.insert(Rc::clone(&variant.name.lexeme), variant_type);
                            }
                        }

                        if write_output {
                            output_fields.insert(Rc::from("kind"), simple_enum_type.clone());

                            def_buf.dec_indent();
                            def_buf.push_indent();
                            def_buf.push("};\n\n");

                            def_buf.push_indent();
                            def_buf.push(&simple_enum_full_name);
                            def_buf.push(" kind;\n");
                            def_buf.dec_indent();

                            def_buf.push_indent();
                            def_buf.push("} ");
                            def_buf.push(&full_name);
                            def_buf.push(";\n\n");

                            def_buf.push(&initializers.code);

                            let struct_output_type = SkyeType::Enum(
                                Rc::clone(&full_name), Some(output_fields),
                                Rc::clone(&base_name)
                            );

                            for variant in evaluated_variants {
                                let mut env = self.globals.borrow_mut();

                                if matches!(variant.type_, SkyeType::Void) {
                                    env.define(
                                        Rc::from(format!("{}_DOT_{}", full_name, variant.name.lexeme)),
                                        SkyeVariable::new(
                                            struct_output_type.clone(),
                                            true,
                                            Some(Box::new(variant.name.clone()))
                                        )
                                    );

                                    let type_ = SkyeType::Function(
                                        Vec::new(), Box::new(struct_output_type.clone()),
                                        true
                                    );

                                    env.define(
                                        Rc::from(format!("{}_DOT_SKYE_ENUM_INIT_{}", full_name, variant.name.lexeme)),
                                        SkyeVariable::new(
                                            type_.clone(), true,
                                            Some(Box::new(variant.name.clone()))
                                        )
                                    );

                                    drop(env);

                                    self.generate_fn_signature(
                                        &variant.name, &type_,
                                        &struct_output_type.stringify(),
                                        &String::new()
                                    );
                                } else {
                                    let type_ = SkyeType::Function(
                                        vec![SkyeFunctionParam::new(variant.type_.clone(), true)],
                                        Box::new(struct_output_type.clone()),
                                        true
                                    );

                                    env.define(
                                        Rc::from(format!("{}_DOT_{}", full_name, variant.name.lexeme)),
                                        SkyeVariable::new(
                                            type_.clone(), true,
                                            Some(Box::new(variant.name.clone()))
                                        )
                                    );

                                    drop(env);

                                    self.generate_fn_signature(
                                        &variant.name, &type_,
                                        &struct_output_type.stringify(),
                                        &variant.type_.stringify()
                                    );
                                }
                            }

                            self.struct_definitions.insert(Rc::clone(&full_name), def_buf);
                            self.struct_defs_order.push(Rc::clone(&full_name));

                            Some(struct_output_type)
                        } else {
                            Some(simple_enum_type)
                        }
                    } else {
                        if binding.is_none() {
                            let full_struct_name = self.get_name(&Rc::from(format!("SKYE_STRUCT_{}", simple_enum_name)));
                            self.declarations.push(CodeOutput::new());
                            self.declarations.last_mut().unwrap().push("typedef struct ");
                            self.declarations.last_mut().unwrap().push(&full_struct_name);
                            self.declarations.last_mut().unwrap().push(" ");
                            self.declarations.last_mut().unwrap().push(&full_name);
                            self.declarations.last_mut().unwrap().push(";\n");
                        }

                        Some(SkyeType::Enum(Rc::clone(&full_name), None, base_name))
                    }
                };

                let mut env = self.globals.borrow_mut();
                let existing = env.get(&Token::dummy(Rc::clone(&full_name)));

                if let Some(var) = &existing {
                    if let SkyeType::Type(inner_type) = &var.type_ {
                        if let SkyeType::Enum(_, existing_fields, _) = &**inner_type {
                            if *has_body && existing_fields.is_some() {
                                token_error!(self, name, "Cannot redefine enums");

                                if let Some(token) = &var.tok {
                                    token_note!(*token, "Previously defined here");
                                }
                            }
                        } else {
                            token_error!(self, name, "Cannot declare enum with same name as existing symbol");

                            if let Some(token) = &var.tok {
                                token_note!(*token, "Previously defined here");
                            }
                        }
                    } else {
                        token_error!(self, name, "Cannot declare enum with same name as existing symbol");

                        if let Some(token) = &var.tok {
                            token_note!(*token, "Previously defined here");
                        }
                    }
                }

                if let Some(out) = output_type {
                    let final_type = SkyeType::Type(Box::new(out));

                    env.define(
                        Rc::clone(&full_name),
                        SkyeVariable::new(
                            final_type.clone(), true,
                            Some(Box::new(name.clone()))
                        )
                    );

                    return Ok(Some(final_type));
                }
            }
            Statement::Defer { kw, statement } => {
                if matches!(self.curr_function, CurrentFn::None) {
                    token_error!(self, kw, "Only declarations are allowed at top level");
                    token_note!(kw, "Remove this defer statement");
                }

                match &**statement {
                    Statement::Return { kw, .. } | Statement::Break(kw) |
                    Statement::Continue(kw) | Statement::Defer { kw, .. } => {
                        token_error!(self, kw, "Cannot use this statement inside a defer statement");
                    }
                    _ => ()
                }

                self.deferred.borrow_mut().last_mut().unwrap().push(*statement.clone());
            }
            Statement::Switch { kw, expr: switch_expr, cases } => {
                if matches!(self.curr_function, CurrentFn::None) {
                    token_error!(self, kw, "Only declarations are allowed at top level");
                    token_note!(kw, "Remove this switch statement");
                }

                let is_not_grouping = !matches!(switch_expr, Expression::Grouping(_));
                let switch = ctx.run(|ctx| self.evaluate(&switch_expr, index, false, ctx)).await;
                let mut is_classic = true;

                match &switch.type_ {
                    SkyeType::U8  | SkyeType::I8  | SkyeType::U16 | SkyeType::I16 |
                    SkyeType::U32 | SkyeType::I32 | SkyeType::U64 | SkyeType::I64 |
                    SkyeType::Usz | SkyeType::F32 | SkyeType::F64 | SkyeType::AnyInt |
                    SkyeType::AnyFloat | SkyeType::Char | SkyeType::Unknown(_) => (),
                    SkyeType::Type(inner) => {
                        is_classic = false;

                        if !inner.can_be_instantiated(false) {
                            ast_error!(self, switch_expr, format!("Cannot instantiate type {}", inner.stringify_native()).as_ref());
                        }
                    }
                    SkyeType::Void => is_classic = false,
                    SkyeType::Enum(_, variants, _) => {
                        if variants.is_some() {
                            ast_error!(
                                self, switch_expr,
                                format!(
                                    "Expecting expression of primitive arithmetic type, simple enum or type for switch condition (got {})",
                                    switch.type_.stringify_native()
                                ).as_ref()
                            );
                        }
                    }
                    _ => {
                        ast_error!(
                            self, switch_expr,
                            format!(
                                "Expecting expression of primitive arithmetic type, simple enum or type for switch condition (got {})",
                                switch.type_.stringify_native()
                            ).as_ref()
                        );
                    }
                }

                if is_classic {
                    self.definitions[index].push_indent();
                    self.definitions[index].push("switch ");

                    if is_not_grouping {
                        self.definitions[index].push("(");
                    }

                    self.definitions[index].push(&switch.value);

                    if is_not_grouping {
                        self.definitions[index].push(")");
                    }

                    self.definitions[index].push(" {\n");
                    self.definitions[index].inc_indent();
                }

                let mut entered_case = false;
                for case in cases {
                    let mut case_types = Vec::new();

                    if let Some(real_cases) = &case.cases {
                        for (i, real_case) in real_cases.iter().enumerate() {
                            if is_classic {
                                self.definitions[index].push_indent();
                                self.definitions[index].push("case ");
                            }

                            let real_case_evaluated = ctx.run(|ctx| self.evaluate(&real_case, index, false, ctx)).await;

                            if is_classic {
                                match real_case_evaluated.type_ {
                                    SkyeType::U8  | SkyeType::I8  | SkyeType::U16 | SkyeType::I16 |
                                    SkyeType::U32 | SkyeType::I32 | SkyeType::U64 | SkyeType::I64 |
                                    SkyeType::Usz | SkyeType::F32 | SkyeType::F64 | SkyeType::AnyInt |
                                    SkyeType::AnyFloat | SkyeType::Char | SkyeType::Unknown(_) => (),
                                    SkyeType::Enum(_, variants, _) => {
                                        if variants.is_some() {
                                            ast_error!(
                                                self, switch_expr,
                                                format!(
                                                    "Expecting expression of primitive arithmetic type or simple enum for case expression (got {})",
                                                    switch.type_.stringify_native()
                                                ).as_ref()
                                            );
                                        }
                                    }
                                    _ => {
                                        ast_error!(
                                            self, real_case,
                                            format!(
                                                "Expecting expression of primitive arithmetic type or simple enum for case expression (got {})",
                                                real_case_evaluated.type_.stringify_native()
                                            ).as_ref()
                                        );
                                    }
                                }

                                self.definitions[index].push(&real_case_evaluated.value);
                                self.definitions[index].push(":");

                                if i != real_cases.len() - 1 {
                                    self.definitions[index].push("\n");
                                } else {
                                    self.definitions[index].push(" ");
                                }
                            } else if !matches!(real_case_evaluated.type_, SkyeType::Type(_) | SkyeType::Void) {
                                ast_error!(
                                    self, real_case,
                                    format!(
                                        "Expecting type or void for case expression (got {})",
                                        real_case_evaluated.type_.stringify_native()
                                    ).as_ref()
                                );
                            } else {
                                case_types.push(real_case_evaluated.type_);
                            }
                        }
                    } else {
                        if is_classic {
                            self.definitions[index].push_indent();
                            self.definitions[index].push("default: ");
                        } else if !entered_case {
                            // use code from the default case if other cases weren't hit

                            ctx.run(|ctx| self.execute_block(
                                &case.code,
                                Rc::new(RefCell::new(
                                    Environment::with_enclosing(
                                        Rc::clone(&self.environment)
                                    )
                                )),
                                index, false, ctx
                            )).await;
                            continue;
                        }
                    }

                    if is_classic {
                        self.definitions[index].push("{\n");
                        self.definitions[index].inc_indent();
                    } else {
                        let no_exec = 'no_exec_block: {
                            for type_ in case_types {
                                if switch.type_.equals(&type_, EqualsLevel::Typewise) {
                                    break 'no_exec_block false;
                                }
                            }

                            true
                        };

                        if no_exec {
                            continue;
                        } else {
                            entered_case = true;
                        }
                    }

                    ctx.run(|ctx| self.execute_block(
                        &case.code,
                        Rc::new(RefCell::new(
                            Environment::with_enclosing(
                                Rc::clone(&self.environment)
                            )
                        )),
                        index, false, ctx
                    )).await;

                    if is_classic {
                        self.definitions[index].dec_indent();
                        self.definitions[index].push_indent();
                        self.definitions[index].push("} break;\n");
                    }
                }

                if is_classic {
                    self.definitions[index].dec_indent();
                    self.definitions[index].push_indent();
                    self.definitions[index].push("}\n");
                }
            }
            Statement::Template { name, declaration: definition, generics, generics_names } => {
                let full_name = self.get_name(&name.lexeme);
                let mut env = self.globals.borrow_mut();
                let cloned_globals = Rc::new(RefCell::new(env.clone()));
                env.define(
                    Rc::clone(&full_name),
                    SkyeVariable::new(
                        SkyeType::Template(
                            full_name, *definition.clone(),
                            generics.clone(), generics_names.clone(),
                            self.curr_name.clone(), cloned_globals
                        ),
                        true,
                        Some(Box::new(name.clone()))
                    )
                );
            }
            Statement::Break(kw) => {
                if let Some((break_label, _)) = &self.curr_loop {
                    return Err(ExecutionInterrupt::Interrupt(Rc::from(format!("goto {};\n", break_label))));
                } else {
                    token_error!(self, kw, "Can only use break inside loops");
                }
            }
            Statement::Continue(kw) => {
                if let Some((_, continue_label)) = &self.curr_loop {
                    return Err(ExecutionInterrupt::Interrupt(Rc::from(format!("goto {};\n", continue_label))));
                } else {
                    token_error!(self, kw, "Can only use continue inside loops");
                }
            }
            Statement::Import { path: path_tok, type_: import_type } => {
                // handle C imports
                let mut path: PathBuf = path_tok.lexeme.split('/').collect();

                let extension = OsString::from(path.extension().expect("missing import processor step: no extension"));
                assert!(extension != "skye", "missing import processor step: extension is skye");

                if *import_type == ImportType::Lib {
                    path = self.skye_path.join("lib").join(path)
                } else if path.is_relative() && self.source_path.is_some() && *import_type != ImportType::Ang {
                    path = PathBuf::from((**self.source_path.as_ref().unwrap()).clone()).join(path);
                } else {
                    path = path_tok.lexeme.split('/').collect();
                }

                let mut buf = String::from("#include ");

                let is_ang = *import_type == ImportType::Ang;
                if is_ang {
                    buf.push('<');
                } else {
                    buf.push('"');
                }

                buf.push_str(&escape_string(&path.to_str().expect("Error converting to string")));

                if is_ang {
                    buf.push('>');
                } else {
                    buf.push('"');
                }

                buf.push('\n');

                if matches!(self.curr_function, CurrentFn::None) {
                    self.includes.push(&buf);
                } else {
                    self.definitions[index].push(&buf);
                }
            }
            Statement::Union { name, fields, has_body, binding, bind_typedefed } => {
                let full_name = self.get_name(&name.lexeme);

                let env = self.globals.borrow();
                let existing = env.get(&Token::dummy(Rc::clone(&full_name)));

                let has_decl = {
                    if let Some(var) = &existing {
                        if let SkyeType::Type(inner_type) = &var.type_ {
                            if let SkyeType::Union(_, existing_fields) = &**inner_type {
                                if *has_body && existing_fields.is_some() {
                                    token_error!(self, name, "Cannot redefine unions");

                                    if let Some(token) = &var.tok {
                                        token_note!(*token, "Previously defined here");
                                    }

                                    false
                                } else {
                                    true
                                }
                            } else {
                                token_error!(self, name, "Cannot declare union with same name as existing symbol");

                                if let Some(token) = &var.tok {
                                    token_note!(*token, "Previously defined here");
                                }

                                false
                            }
                        } else {
                            token_error!(self, name, "Cannot declare union with same name as existing symbol");

                            if let Some(token) = &var.tok {
                                token_note!(*token, "Previously defined here");
                            }

                            false
                        }
                    } else {
                        false
                    }
                };

                drop(env);

                let mut buf = String::from("typedef ");
                let mut equal_binding = false;

                if let Some(bound_name) = binding {
                    if !*bind_typedefed {
                        buf.push_str("union ");
                    } else {
                        equal_binding = bound_name.lexeme == full_name;
                    }

                    buf.push_str(&bound_name.lexeme);
                } else {
                    buf.push_str("union ");
                    let full_union_name = self.get_name(&Rc::from(format!("SKYE_UNION_{}", name.lexeme)));
                    buf.push_str(&full_union_name);
                }

                buf.push(' ');

                if (!equal_binding) && ((!has_decl) || (!*has_body)) {
                    self.declarations.push(CodeOutput::new());
                    self.declarations.last_mut().unwrap().push(&buf);
                    self.declarations.last_mut().unwrap().push(&full_name);
                    self.declarations.last_mut().unwrap().push(";\n");
                }

                let mut def_buf = CodeOutput::new();

                if *has_body && binding.is_none() {
                    def_buf.push(&buf);
                }

                let type_ = {
                    if *has_body {
                        if binding.is_none() {
                            def_buf.push("{\n");
                            def_buf.inc_indent();
                        }

                        let mut output_fields = HashMap::new();
                        for field in fields {
                            let field_type = {
                                let inner_field_type = ctx.run(|ctx| self.evaluate(&field.expr, index, false, ctx)).await.type_;

                                if let SkyeType::Type(inner_type) = inner_field_type {
                                    if inner_type.check_completeness() {
                                        *inner_type
                                    } else {
                                        ast_error!(self, field.expr, "Cannot use incomplete type directly");
                                        ast_note!(field.expr, "Define this type or reference it through a pointer");
                                        SkyeType::get_unknown()
                                    }

                                } else {
                                    ast_error!(
                                        self, field.expr,
                                        format!(
                                            "Expecting type as field type (got {})",
                                            inner_field_type.stringify_native()
                                        ).as_ref()
                                    );

                                    SkyeType::get_unknown()
                                }
                            };

                            if output_fields.contains_key(&field.name.lexeme) {
                                token_error!(self, field.name, "Cannot define the same union field multiple times");
                            } else {
                                let field_type_stringified = field_type.stringify();
                                output_fields.insert(Rc::clone(&field.name.lexeme), field_type);

                                if binding.is_none() {
                                    def_buf.push_indent();
                                    def_buf.push(&field_type_stringified);
                                    def_buf.push(" ");
                                    def_buf.push(&field.name.lexeme);
                                    def_buf.push(";\n");
                                }
                            }
                        }

                        if binding.is_none() {
                            def_buf.dec_indent();
                            def_buf.push("} ");
                            def_buf.push(&full_name);
                            def_buf.push(";\n\n");
                        }

                        self.struct_definitions.insert(Rc::clone(&full_name), def_buf);
                        self.struct_defs_order.push(Rc::clone(&full_name));

                        SkyeType::Union(Rc::clone(&full_name), Some(output_fields))
                    } else {
                        SkyeType::Union(Rc::clone(&full_name), None)
                    }
                };

                let output_type = SkyeType::Type(Box::new(type_));

                let mut env = self.globals.borrow_mut();

                env.define(
                    Rc::clone(&full_name),
                    SkyeVariable::new(
                        output_type.clone(), true,
                        Some(Box::new(name.clone()))
                    )
                );
            }
            Statement::Bitfield { name, fields, has_body, binding, bind_typedefed } => {
                let full_name = self.get_name(&name.lexeme);

                let env = self.globals.borrow();
                let existing = env.get(&Token::dummy(Rc::clone(&full_name)));

                let has_decl = {
                    if let Some(var) = &existing {
                        if let SkyeType::Type(inner_type) = &var.type_ {
                            if let SkyeType::Bitfield(_, existing_fields) = &**inner_type {
                                if *has_body && existing_fields.is_some() {
                                    token_error!(self, name, "Cannot redefine bitfields");

                                    if let Some(token) = &var.tok {
                                        token_note!(*token, "Previously defined here");
                                    }

                                    false
                                } else {
                                    true
                                }
                            } else {
                                token_error!(self, name, "Cannot declare union with same name as existing symbol");

                                if let Some(token) = &var.tok {
                                    token_note!(*token, "Previously defined here");
                                }

                                false
                            }
                        } else {
                            token_error!(self, name, "Cannot declare union with same name as existing symbol");

                            if let Some(token) = &var.tok {
                                token_note!(*token, "Previously defined here");
                            }

                            false
                        }
                    } else {
                        false
                    }
                };

                drop(env);

                let mut buf = String::from("typedef ");
                let mut equal_binding = false;

                if let Some(bound_name) = binding {
                    if !*bind_typedefed {
                        buf.push_str("struct ");
                    } else {
                        equal_binding = bound_name.lexeme != full_name;
                    }

                    buf.push_str(&bound_name.lexeme);
                } else {
                    buf.push_str("struct ");
                    let full_struct_name = self.get_name(&Rc::from(format!("SKYE_STRUCT_{}", name.lexeme)));
                    buf.push_str(&full_struct_name);
                }

                buf.push(' ');

                if (!equal_binding) && ((!has_decl) || (!*has_body)) {
                    self.declarations.push(CodeOutput::new());
                    self.declarations.last_mut().unwrap().push(&buf);
                    self.declarations.last_mut().unwrap().push(&full_name);
                    self.declarations.last_mut().unwrap().push(";\n");
                }

                let mut def_buf = CodeOutput::new();

                if *has_body && binding.is_none() {
                    def_buf.push(&buf);
                }

                let type_ = {
                    if *has_body {
                        if binding.is_none() {
                            def_buf.push("{\n");
                            def_buf.inc_indent();
                        }

                        let mut output_fields = HashMap::new();
                        for field in fields {
                            let field_type = {
                                match field.bits {
                                    0  ..= 8  => SkyeType::U8,
                                    9  ..= 16 => SkyeType::U16,
                                    17 ..= 32 => SkyeType::U32,
                                    33 ..= 64 => SkyeType::U64,
                                    _ => unreachable!() // parser ensures this is unreachable
                                }
                            };

                            if output_fields.contains_key(&field.name.lexeme) {
                                token_error!(self, field.name, "Cannot define the same bitfield field multiple times");
                            } else {
                                let field_type_stringified = field_type.stringify();
                                output_fields.insert(Rc::clone(&field.name.lexeme), field_type);

                                if binding.is_none() {
                                    def_buf.push_indent();
                                    def_buf.push(&field_type_stringified);
                                    def_buf.push(" ");
                                    def_buf.push(&field.name.lexeme);
                                    def_buf.push(format!(": {}", field.bits).as_ref());
                                    def_buf.push(";\n");
                                }
                            }
                        }

                        if binding.is_none() {
                            def_buf.dec_indent();
                            def_buf.push("} ");
                            def_buf.push(&full_name);
                            def_buf.push(";\n\n");
                        }

                        self.struct_definitions.insert(Rc::clone(&full_name), def_buf);
                        self.struct_defs_order.push(Rc::clone(&full_name));

                        SkyeType::Bitfield(Rc::clone(&full_name), Some(output_fields))
                    } else {
                        SkyeType::Bitfield(Rc::clone(&full_name), None)
                    }
                };

                let output_type = SkyeType::Type(Box::new(type_));

                let mut env = self.globals.borrow_mut();

                env.define(
                    Rc::clone(&full_name),
                    SkyeVariable::new(
                        output_type.clone(), true,
                        Some(Box::new(name.clone()))
                    )
                );
            }
            Statement::Macro { name, params, body } => {
                let full_name = {
                    if matches!(body, MacroBody::Binding(_)) {
                        if self.curr_name != "" {
                            token_warning!(name, "C macro bindings do not support namespaces. This macro will be saved in the global namespace"); // +Wmacro-namespace
                        }

                        Rc::clone(&name.lexeme)
                    } else {
                        self.get_name(&name.lexeme)
                    }
                };

                let mut env = self.globals.borrow_mut();
                env.define(
                    Rc::clone(&full_name),
                    SkyeVariable::new(
                        SkyeType::Type(Box::new(
                            SkyeType::Macro(
                                full_name,
                                params.clone(),
                                body.clone()
                            )
                        )),
                        true,
                        Some(Box::new(name.clone()))
                    )
                );
            }
            Statement::Foreach { kw, variable_name: var_name, iterator: iterator_expr, body } => {
                if matches!(self.curr_function, CurrentFn::None) {
                    token_error!(self, kw, "Only declarations are allowed at top level");
                    token_note!(kw, "Place this for loop inside a function");
                }

                let iterator_raw = ctx.run(|ctx| self.evaluate(iterator_expr, index, false, ctx)).await;

                let tmp_iter_var_name = self.get_temporary_var();

                if !matches!(iterator_raw.type_, SkyeType::Struct(..) | SkyeType::Enum(..)) {
                    ast_error!(
                        self, iterator_expr,
                        format!(
                            "This type ({}) is not iterable",
                            iterator_raw.type_.stringify_native()
                        ).as_ref()
                    );

                    return Ok(None);
                }

                self.definitions[index].push_indent();
                self.definitions[index].push("{\n");
                self.definitions[index].inc_indent();

                self.definitions[index].push_indent();
                self.definitions[index].push(&iterator_raw.type_.stringify());
                self.definitions[index].push(" ");
                self.definitions[index].push(&tmp_iter_var_name);
                self.definitions[index].push(" = ");
                self.definitions[index].push(&iterator_raw.value);
                self.definitions[index].push(";\n");

                let iterator = SkyeValue::new(Rc::from(tmp_iter_var_name.as_ref()), iterator_raw.type_.clone(), iterator_raw.is_const);

                let mut search_tok = Token::dummy(Rc::from("next"));
                let method = {
                    if let Some(method) = self.get_method(&iterator, &search_tok, false, index) {
                        method
                    } else {
                        search_tok.set_lexeme("iter");

                        if let Some(method) = self.get_method(&iterator, &search_tok, false, index) {
                            let v = Vec::new();
                            let iterator_call = ctx.run(|ctx| self.call(&method, iterator_expr, &iterator_expr, &v, index, false, ctx)).await;

                            let iterator_type_stringified = iterator_call.type_.stringify();
                            if iterator_type_stringified.len() == 0 || !matches!(iterator.type_, SkyeType::Struct(..) | SkyeType::Enum(..)) {
                                ast_error!(
                                    self, iterator_expr,
                                    format!(
                                        "The implementation of iter for this type ({}) returns an invalid type (expecting struct or enum type but got {})",
                                        iterator.type_.stringify_native(), iterator_call.type_.stringify_native()
                                    ).as_ref()
                                );

                                return Ok(None);
                            }

                            let iterator_val = SkyeValue::new(Rc::clone(&iterator_call.value), iterator_call.type_, false);

                            search_tok.set_lexeme("next");
                            if let Some(final_method) = self.get_method(&iterator_val, &search_tok, false, index) {
                                final_method
                            } else {
                                ast_error!(
                                    self, iterator_expr,
                                    format!(
                                        "The iterator object (of type {}) returned by iter has no next method",
                                        iterator_val.type_.stringify_native()
                                    ).as_ref()
                                );

                                return Ok(None);
                            }
                        } else {
                            ast_error!(
                                self, iterator_expr,
                                format!(
                                    "Type {} is not iterable",
                                    iterator_raw.type_.stringify_native()
                                ).as_ref()
                            );

                            return Ok(None);
                        }
                    }
                };

                let previous = Rc::clone(&self.environment);
                self.environment = Rc::new(RefCell::new(Environment::with_enclosing(Rc::clone(&self.environment))));

                self.definitions[index].push_indent();
                self.definitions[index].push("while (1) {\n");
                self.definitions[index].inc_indent();

                let v = Vec::new();
                let next_call = ctx.run(|ctx| self.call(&method, iterator_expr, &iterator_expr, &v, index, false, ctx)).await;

                let item_type = {
                    if let SkyeType::Enum(_, variants, name) = &next_call.type_ {
                        if name.as_ref() != "core_DOT_Option" {
                            ast_error!(
                                self, iterator_expr,
                                format!(
                                    "The implementation of next for this iterator returns an invalid type (expecting core::Option but got {})",
                                    next_call.type_.stringify_native()
                                ).as_ref()
                            );

                            return Ok(None);
                        }

                        variants.as_ref().unwrap().get("Some").unwrap().clone()
                    } else {
                        ast_error!(
                            self, iterator_expr,
                            format!(
                                "The implementation of next for this iterator returns an invalid type (expecting core::Option but got {})",
                                next_call.type_.stringify_native()
                            ).as_ref()
                        );

                        return Ok(None);
                    }
                };

                let item_type_stringified = item_type.stringify();
                if item_type_stringified.len() == 0 {
                    ast_error!(
                        self, iterator_expr,
                        format!(
                            "The implementation of next for this iterator returns an invalid type (expecting core::Option but got {})",
                            next_call.type_.stringify_native()
                        ).as_ref()
                    );

                    return Ok(None);
                }

                let mut env = self.environment.borrow_mut();
                env.define(
                    Rc::clone(&var_name.lexeme),
                    SkyeVariable::new(
                        item_type,
                        true,
                        Some(Box::new(var_name.clone()))
                    )
                );
                drop(env);

                self.definitions[index].push_indent();
                self.definitions[index].push("if (");
                self.definitions[index].push(&next_call.value);
                self.definitions[index].push(".kind != core_DOT_Option_DOT_Kind_DOT_Some) break;\n");

                self.definitions[index].push_indent();
                self.definitions[index].push(&item_type_stringified);
                self.definitions[index].push(" ");
                self.definitions[index].push(&var_name.lexeme);
                self.definitions[index].push(" = ");
                self.definitions[index].push(&next_call.value);
                self.definitions[index].push(".Some;\n");

                let continue_label = Rc::from(self.get_temporary_var());
                let break_label = Rc::from(self.get_temporary_var());

                let previous_loop = self.curr_loop.clone();
                self.curr_loop = Some((Rc::clone(&break_label), Rc::clone(&continue_label)));

                if matches!(**body, Statement::Block(..)) {
                    let stmts = vec![*body.clone()];
                    ctx.run(|ctx| self.execute_block(
                        &stmts,
                        Rc::new(RefCell::new(Environment::with_enclosing(
                            Rc::clone(&self.environment)
                        ))),
                        index, false, ctx
                    )).await;
                } else {
                    let _ = ctx.run(|ctx| self.execute(&body, index, ctx)).await;
                }

                self.curr_loop = previous_loop;
                self.environment = previous;

                self.definitions[index].push_indent();
                self.definitions[index].push(&continue_label);
                self.definitions[index].push(":;\n");

                self.definitions[index].dec_indent();
                self.definitions[index].push_indent();
                self.definitions[index].push("}\n");

                self.definitions[index].dec_indent();
                self.definitions[index].push_indent();
                self.definitions[index].push("}\n");

                self.definitions[index].push_indent();
                self.definitions[index].push(&break_label);
                self.definitions[index].push(":;\n");
            }
            Statement::Interface { name, declarations, types } => {
                let full_name = self.get_name(&name.lexeme);

                if let Some(body) = declarations {
                    if let Some(bound_types) = types {
                        let mut variants = Vec::new();
                        let mut evaluated_types = Vec::new();

                        for bound_type in bound_types {
                            let evaluated = ctx.run(|ctx| self.evaluate(&bound_type, index, false, ctx)).await;
                            if matches!(evaluated.type_, SkyeType::Void) || !evaluated.type_.can_be_instantiated(true) {
                                ast_error!(self, bound_type, format!("Cannot instantiate type {}", evaluated.type_.stringify_native()).as_ref());
                            }

                            let mut name_tok = name.clone();
                            name_tok.set_lexeme(evaluated.type_.mangle().as_ref());
                            variants.push(EnumVariant::new(name_tok.clone(), bound_type.clone()));
                            evaluated_types.push(evaluated.type_);
                        }

                        let mut functions = Vec::new();

                        for statement in body {
                            let mut cases = Vec::new();

                            let mut self_type_tok = name.clone();
                            self_type_tok.set_lexeme("Self");

                            let mut self_tok = name.clone();
                            self_tok.set_lexeme("self");

                            let mut kind_type_tok = name.clone();
                            kind_type_tok.set_lexeme("Kind");

                            let mut kind_tok = name.clone();
                            kind_tok.set_lexeme("kind");

                            if let Statement::Function { name: fn_name, params, return_type, body: fn_body, qualifiers, generics_names, bind, init } = statement {
                                let mut args = Vec::new();
                                for (i, param) in params.iter().enumerate() {
                                    let name = param.name.as_ref().expect("param name wasn't available in interface");

                                    if i == 0 && name.lexeme.as_ref() == "self" {
                                        continue;
                                    }

                                    args.push(Expression::Variable(name.clone()))
                                }

                                for type_ in &evaluated_types {
                                    let type_name = type_.mangle();
                                    let mut name_tok = name.clone();
                                    name_tok.set_lexeme(type_name.as_ref());

                                    if let GetResult::Ok(obj_name, ..) = type_.static_get(&fn_name) {
                                        let mut search_tok = fn_name.clone();
                                        search_tok.set_lexeme(&obj_name);

                                        if let Some(_) = self.globals.borrow().get(&search_tok) {
                                            cases.push(SwitchCase::new(
                                                Some(vec![
                                                    Expression::StaticGet(
                                                        Box::new(Expression::StaticGet(
                                                            Box::new(Expression::Variable(self_type_tok.clone())),
                                                            kind_type_tok.clone(),
                                                            false
                                                        )),
                                                        name_tok.clone(),
                                                        false
                                                    )
                                                ]),
                                                vec![Statement::Return { kw: name_tok.clone(), value: Some(Expression::Call(Box::new(Expression::Get(Box::new(Expression::Get(Box::new(Expression::Variable(self_tok.clone())),name_tok.clone())),fn_name.clone())),name_tok,args.clone())) }]
                                            ));
                                        }
                                    } else {
                                        unreachable!();
                                    }
                                }

                                // if the interface function has a body, use that as default implementation
                                if let Some(body) = fn_body {
                                    cases.push(SwitchCase::new(None, body.clone()));
                                }

                                functions.push(Statement::Function {
                                    name: fn_name.clone(),
                                    params: params.clone(),
                                    return_type: return_type.clone(),
                                    body: Some(vec![Statement::Switch { kw: name.clone(), expr: Expression::Get(Box::new(Expression::Variable(self_tok)),kind_tok), cases: cases }]),
                                    qualifiers: qualifiers.clone(),
                                    generics_names: generics_names.clone(),
                                    bind: *bind,
                                    init: *init
                                });
                            } else {
                                ast_error!(self, statement, "Can only define functions in interface body");
                            }
                        }

                        let mut custom_tok = name.clone();
                        custom_tok.set_lexeme("i32");

                        let enum_def = Statement::Enum { name: name.clone(), kind_type: Expression::Variable(custom_tok.clone()), variants, is_simple: false, has_body: true, binding: None, generics_names: Vec::new(), bind_typedefed: false };

                        let _ = ctx.run(|ctx| self.execute(&enum_def, index, ctx)).await;

                        let old_errors = self.errors;

                        custom_tok.set_lexeme(&full_name);
                        let impl_def = Statement::Impl { object: Expression::Variable(custom_tok), declarations: functions };

                        if old_errors != self.errors {
                            token_note!(
                                name,
                                concat!(
                                    "This error is a result of code generation for this interface. ",
                                    "Perhaps one or more of the methods implementing the interface are incompatible with it"
                                )
                            );
                        }

                        let _ = ctx.run(|ctx| self.execute(&impl_def, index, ctx)).await;
                    } else {
                        let mut functions = Vec::new();

                        for statement in body {
                            if let Statement::Function { name: fn_name, params, return_type, body: fn_body, qualifiers, generics_names, bind, init } = statement {
                                // if the interface function has a body, use that as default implementation
                                if fn_body.is_some() {
                                    token_error!(self, fn_name, "Cannot define function body in forward declaration of interface");
                                }

                                functions.push(Statement::Function {
                                    name: fn_name.clone(),
                                    params: params.clone(),
                                    return_type: return_type.clone(),
                                    body: None,
                                    qualifiers: qualifiers.clone(),
                                    generics_names: generics_names.clone(),
                                    bind: *bind,
                                    init: *init
                                });
                            } else {
                                ast_error!(self, statement, "Can only define functions in interface body");
                            }
                        }

                        let mut custom_tok = name.clone();
                        custom_tok.set_lexeme("i32");

                        let enum_def = Statement::Enum {
                            name: name.clone(),
                            kind_type: Expression::Variable(custom_tok.clone()),
                            variants: Vec::new(),
                            is_simple: false,
                            has_body: false,
                            binding: None,
                            generics_names: Vec::new(),
                            bind_typedefed: false
                        };

                        let _ = ctx.run(|ctx| self.execute(&enum_def, index, ctx)).await;

                        custom_tok.set_lexeme(&full_name);
                        let impl_def = Statement::Impl { object: Expression::Variable(custom_tok), declarations: functions };

                        let _ = ctx.run(|ctx| self.execute(&impl_def, index, ctx)).await;
                    }
                } else {
                    assert!(types.is_none()); // ensured by parser

                    let mut custom_tok = name.clone();
                    custom_tok.set_lexeme("i32");

                    let enum_def = Statement::Enum { name: name.clone(), kind_type: Expression::Variable(custom_tok), variants: Vec::new(), is_simple: false, has_body: false, binding: None, generics_names: Vec::new(), bind_typedefed: false };

                    let _ = ctx.run(|ctx| self.execute(&enum_def, index, ctx)).await;
                }
            }
        }

        Ok(None)
    }

    fn compile_internal(&mut self, statements: Vec<Statement>, index: usize) {
        let mut stack = reblessive::Stack::new();

        for statement in statements {
            let _ = stack.enter(|ctx| self.execute(&statement, index, ctx)).finish();
        }
    }

    pub fn compile(&mut self, statements: Vec<Statement>) {
        self.compile_internal(statements, ANY_DEF_START_INDEX);
        self.definitions[INIT_DEF_INDEX].push("}\n\n"); // closes _SKYE_INIT block
    }

    pub fn get_output(&self) -> Option<String> {
        if self.errors == 0 {
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
                for declaration in &self.declarations {
                    if !declaration.code.contains("_UNKNOWN_") {
                        output.push_str(&declaration.code);
                    }
                }

                output.push('\n');
            }

            for definition in &self.struct_defs_order {
                if !definition.contains("_UNKNOWN_") {
                    output.push_str(&self.struct_definitions.get(definition).unwrap().code);
                }
            }

            for definition in &self.definitions {
                output.push_str(&definition.code);
            }

            Some(output)
        } else {
            None
        }
    }
}
