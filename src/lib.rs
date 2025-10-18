use std::{ffi::{OsStr, OsString}, fs::{self, create_dir, read_dir, remove_file, File}, io::{Error, Read, Write}, path::{Path, PathBuf}, process::Command, rc::Rc, sync::OnceLock};

use ast::{ImportType, Statement};
use clap::ValueEnum;
use irgen::IrGen;
use constant_folder::ConstantFolder;
use import_processor::ImportProcessor;
use macro_expander::MacroExpander;
use parser::Parser;
use scanner::Scanner;
use tokens::{Token, TokenType};
use zip::{write::SimpleFileOptions, ZipWriter};

use crate::codegen::CodeGen;

mod utils;
mod tokens;
mod scanner;
mod ast;
mod ir;
mod parser;
mod skye_type;
mod environment;
mod irgen;
mod codegen;
mod import_processor;
mod macro_expander;
mod constant_folder;

pub const MAX_PACKAGE_SIZE_BYTES: u128 = 2u128.pow(32); // Max uncompressed package size is 4 GB (basic protection against malicious ZIPs)

pub static NAMESPACE_SEP: OnceLock<String> = OnceLock::new();

pub fn parse(source: &String, filename: Rc<str>) -> Option<Vec<Statement>> {
    let mut scanner = Scanner::new(source, filename);
    scanner.scan_tokens();

    if scanner.had_error {
        return None;
    }

    let mut parser = Parser::new(scanner.tokens);
    let statements = parser.parse();

    if parser.errors != 0 {
        return None;
    }

    Some(statements)
}

#[derive(ValueEnum, Clone, Copy, Default, Debug)]
pub enum Checks {
    #[default]
    Debug,
    Release,
    ReleaseUnsafe
}

#[derive(ValueEnum, Clone, Copy, Default, Debug)]
pub enum TargetOS {
    Linux,
    MacOS,
    Windows,
    Unknown,
    #[default]
    Current
}

impl TargetOS {
    pub fn get_filename(self, name: &Path) -> PathBuf {
        if matches!(self, TargetOS::Windows) || (matches!(self, TargetOS::Current) && cfg!(windows)) {
            name.with_extension("exe")
        } else {
            name.to_path_buf()
        }
    }
}

#[derive(Clone)]
pub struct CompilerConfig {
    pub skye_path: PathBuf,
    pub skyec: PathBuf,
    pub no_builtins: bool, 
    pub no_panic: bool,
    pub primitives: String,
    pub checks: Checks,
    pub ptr_size: u8,
    pub target_os: TargetOS
}

impl CompilerConfig {
    pub fn new(skyec: PathBuf, skye_path: PathBuf, primitives: String, no_builtins: bool, no_panic: bool, ptr_size: u8, target_os: TargetOS) -> Self {
        CompilerConfig { 
            skye_path, 
            skyec, 
            primitives,
            no_builtins, 
            no_panic,
            ptr_size: {
                if ptr_size == 0 {
                    std::mem::size_of::<usize>() as u8
                } else {
                    ptr_size
                }
            },
            target_os: {
                if matches!(target_os, TargetOS::Current) {
                    match std::env::consts::OS {
                        "linux"   => TargetOS::Linux,
                        "windows" => TargetOS::Windows,
                        "macos"   => TargetOS::MacOS,
                        _         => TargetOS::Unknown
                    }
                } else {
                    target_os
                }
            },
            checks: Checks::Debug
        }
    }
}

fn prepare_base_imports(statements: &mut Vec<Statement>, source: &String, filename: Rc<str>, compiler_conf: &CompilerConfig) {
    statements.insert(
        0,
        Statement::Import { 
            path: Token::new(
                Rc::from(source.as_ref()),
                Rc::clone(&filename),
                TokenType::Identifier,
                Rc::from("core/core"),
                0, 1, 0
            ), 
            type_: ImportType::Default,
            is_include: false,
            flags: Vec::new()
        }
    );

    if compiler_conf.no_builtins {
        return;
    }

    statements.insert(
        1,
        Statement::Import { 
            path: Token::new(
                Rc::from(source.as_ref()),
                Rc::clone(&filename),
                TokenType::Identifier,
                Rc::from(compiler_conf.primitives.as_ref()),
                0, 1, 0
            ), 
            type_: ImportType::Default,
            is_include: false,
            flags: Vec::new()
        }
    );

    statements.insert(
        2,
        Statement::Import {
            path: Token::new(
                Rc::from(source.as_ref()),
                Rc::clone(&filename),
                TokenType::Identifier,
                Rc::from("core/builtins"),
                0, 1, 0
            ), 
            type_: ImportType::Default,
            is_include: false,
            flags: Vec::new()
        }
    );

    if !compiler_conf.no_panic {
        statements.insert(
            3,
            Statement::Import { 
                path: Token::new(
                    Rc::from(source.as_ref()),
                    filename,
                    TokenType::Identifier,
                    Rc::from("core/panic"),
                    0, 1, 0
                ), 
                type_: ImportType::Default,
                is_include: false,
                flags: Vec::new()
            }
        );
    }
}

pub struct CompilationResult {
    pub code: String,
    pub extern_libs: Vec<Rc<str>>
}

pub fn compile(source: &String, path: Option<&Path>, filename: Rc<str>, compiler_conf: CompilerConfig) -> Option<CompilationResult> {
    let mut statements = parse(source, Rc::clone(&filename))?;
    prepare_base_imports(&mut statements, source, filename, &compiler_conf);

    let mut import_processor = ImportProcessor::new(path, compiler_conf.skye_path.clone());
    import_processor.process(&mut statements);

    if import_processor.errors != 0 {
        return None;
    }

    let mut constant_folder = ConstantFolder::new(compiler_conf.ptr_size);
    constant_folder.fold(&mut statements);

    if constant_folder.errors != 0 {
        return None;
    }

    let mut macro_expander = MacroExpander::new(compiler_conf.clone());
    macro_expander.expand(&mut statements);

    if macro_expander.errors != 0 {
        return None;
    }

    constant_folder.reset();
    constant_folder.fold(&mut statements);

    if constant_folder.errors != 0 {
        return None;
    }

    let mut irgen = IrGen::new(path, compiler_conf);
    irgen.compile(statements);

    if irgen.errors != 0 {
        return None;
    }

    let mut codegen = CodeGen::new();
    let code = codegen.generate(IrGen::get_definitions(irgen.definitions))?;
    Some(CompilationResult { code, extern_libs: IrGen::get_extern(irgen.extern_libs) })
}

pub fn parse_file(path: &OsStr) -> Result<Vec<Statement>, Error> {
    let mut f = File::open(path)?;
    let mut input = String::new();
    f.read_to_string(&mut input)?;

    if let Some(statements) = parse(&input, Rc::from(path.to_str().unwrap())) {
        Ok(statements)
    } else {
        Err(Error::other("Compilation failed"))
    }
}

pub fn compile_file(path: &OsStr, compiler_conf: CompilerConfig) -> Result<CompilationResult, Error> {
    let mut f = File::open(path)?;
    let mut input = String::new();
    f.read_to_string(&mut input)?;

    compile(&input, PathBuf::from(path).parent(), Rc::from(path.to_str().unwrap()), compiler_conf)
        .ok_or(Error::other("Compilation failed"))
}

pub fn compile_file_to_c(input: &OsStr, output: &OsStr, compiler_conf: CompilerConfig) -> Result<Vec<Rc<str>>, Error> {
    let result = compile_file(input, compiler_conf)?;
    let mut f = File::create(output)?;
    f.write_all(result.code.as_bytes())?;
    Ok(result.extern_libs)
}

pub fn compile_c(input: &OsStr, output: &OsStr, extern_libs: &Vec<Rc<str>>) -> Result<(), Error> {
    let mut command = 'cc_command: {
        match std::env::var("CC") {
            Ok(cc) => Command::new(cc),
            Err(e) => {
                if cfg!(unix) {
                    if Command::new("cc").arg("--version")
                        .output()?.status.success() 
                    {
                        break 'cc_command Command::new("cc");
                    }

                    return Err(Error::other(format!(
                        concat!(
                            "Could not find C compiler: {}\n",
                            "Install a default C compiler or set the CC environment variable"
                        ), e
                    ).as_str()));
                }

                return Err(Error::other(format!(
                    concat!(
                        "Could not find C compiler: {}\n",
                        "Is the CC environment variable set?"
                    ), e
                ).as_str()));
            }
        }
    };

    command.arg("-w").arg(input);

    for lib in extern_libs {
        command.arg(format!("-l{lib}"));
    }

    command.arg("-o").arg(output);

    if !command.status()?.success() {
        return Err(Error::other("Build failed"));
    }

    Ok(())
}

pub fn compile_file_to_exec(input: &OsStr, output: &OsStr, compiler_conf: CompilerConfig) -> Result<Vec<Rc<str>>, Error> {
    let buf = compiler_conf.skye_path.join("tmp.c");
    let tmp_c = OsStr::new(buf.to_str().expect("Couldn't convert PathBuf to &str"));

    let extern_libs = compile_file_to_c(input, tmp_c, compiler_conf)?;
    compile_c(tmp_c, output, &extern_libs)?;
    remove_file(tmp_c)?;
    Ok(extern_libs)
}

pub fn run_skye(file: OsString, program_args: &Option<Vec<String>>, compiler_conf: CompilerConfig) -> Result<(), Error> {
    let buf = compiler_conf.skye_path.join("tmp");
    let output = compiler_conf.target_os.get_filename(buf.as_path());

    compile_file_to_exec(&file, output.as_os_str(), compiler_conf)?;
    let mut com = Command::new(&output);

    if let Some(args) = program_args {
        com.args(args);
    }

    com.status()?;
    remove_file(&output)?;
    Ok(())
}

pub fn get_package_data(orig_path: &str) -> Result<(Vec<PathBuf>, Vec<PathBuf>, PathBuf), Error> {
    let mut file_count: usize = 0;
    let mut fold_count: usize = 0;
    let mut project_name = PathBuf::new();
    let mut files_absolute = Vec::new();
    let mut files_relative = Vec::new();

    let orig_path_buf = PathBuf::from(orig_path);

    for dir_entry in read_dir(orig_path)? {
        if file_count + fold_count > 2 {
            break;
        }

        let path = PathBuf::from(&dir_entry?.file_name());

        if let Some(extension) = path.extension() {
            if extension == "skye" {
                let name = path.file_stem().unwrap();

                if name == "setup" {
                    files_absolute.push(orig_path_buf.join(&path));
                    files_relative.push(path);
                    continue;
                } else if project_name.as_os_str() == "" {
                    project_name = PathBuf::from(name);
                } else if project_name != name {
                    return Ok((Vec::new(), Vec::new(), PathBuf::new()));
                }

                files_absolute.push(orig_path_buf.join(&path));
                files_relative.push(path);
                file_count += 1;
                continue;
            }
        } else if let Some(name) = path.file_name()  {
            if project_name.as_os_str() == "" {
                project_name = PathBuf::from(name);
            } else if project_name.as_os_str() != name {
                return Ok((Vec::new(), Vec::new(), PathBuf::new()));
            }

            files_absolute.push(orig_path_buf.join(&path));
            files_relative.push(path);
            fold_count += 1;
        } else {
            return Ok((Vec::new(), Vec::new(), PathBuf::new()));
        }
    }

    if file_count != 1 || fold_count > 1 {
        return Ok((Vec::new(), Vec::new(), PathBuf::new()));
    }

    Ok((files_absolute, files_relative, project_name))
}

pub fn write_package(data_absolute: &Vec<PathBuf>, data_relative: &Vec<PathBuf>, options: SimpleFileOptions, writer: &mut ZipWriter<File>) -> Result<(), Error> {
    for (i, item) in data_absolute.iter().enumerate() {
        if item.is_file() {
            let mut file = File::open(&item)?;
            let output_name = data_relative[i].to_str().unwrap();

            println!("Exporting {}", output_name);

            writer.start_file(output_name, options)?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)?;
            writer.write_all(&buffer)?;
        } else {
            writer.add_directory_from_path(&data_relative[i], options)?;

            let inner_data_absolute = read_dir(item)?
                .filter(|x| x.is_ok())
                .map(|x| item.join(x.unwrap().file_name()))
                .collect();

            let inner_data_relative = read_dir(item)?
                .filter(|x| x.is_ok())
                .map(|x| data_relative[i].join(x.unwrap().file_name()))
                .collect();

            write_package(&inner_data_absolute, &inner_data_relative, options, writer)?;
        }
    }

    Ok(())
}

pub fn copy_dir_recursive(src: &PathBuf, dst: &PathBuf) -> Result<(), Error> {
    for entry in read_dir(src)? {
        let path = src.join(&entry?.file_name());

        if path.is_file() {
            fs::copy(&path, &dst.join(path.file_name().unwrap()))?;
        } else {
            let dst_new = dst.join(path.file_name().unwrap());
            create_dir(&dst_new)?;
            copy_dir_recursive(&path, &dst_new)?;
        }
    }
    Ok(())
}
