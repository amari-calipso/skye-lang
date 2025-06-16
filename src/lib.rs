use std::{ffi::{OsStr, OsString}, fs::{self, create_dir, read_dir, remove_file, File}, io::{Error, Read, Write}, path::{Path, PathBuf}, process::Command, rc::Rc};

use ast::{ImportType, Statement};
use clap::ValueEnum;
use codegen::CodeGen;
use constant_folder::ConstantFolder;
use import_processor::ImportProcessor;
use macro_expander::MacroExpander;
use parser::Parser;
use scanner::Scanner;
use tokens::{Token, TokenType};
use zip::{write::SimpleFileOptions, ZipWriter};

mod utils;
mod tokens;
mod scanner;
mod ast;
mod parser;
mod skye_type;
mod environment;
mod codegen;
mod import_processor;
mod macro_expander;
mod constant_folder;

pub const MAX_PACKAGE_SIZE_BYTES: u128 = 2u128.pow(32); // Max uncompressed package size is 4 GB (basic protection against malicious ZIPs)

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
pub enum CompileMode {
    #[default]
    Debug,
    Release,
    ReleaseUnsafe
}

#[derive(Clone)]
pub struct CompilerFlags {
    pub no_builtins: bool, 
    pub no_panic: bool,
    pub primitives: String,
    pub compile_mode: CompileMode
}

fn prepare_base_imports(statements: &mut Vec<Statement>, source: &String, filename: Rc<str>, compiler_flags: &CompilerFlags) {
    statements.insert(
        0,
        Statement::Import { path: Token::new(
            Rc::from(source.as_ref()),
            Rc::clone(&filename),
            TokenType::Identifier,
            Rc::from("core/core"),
            0, 1, 0
        ), type_: ImportType::Default }
    );

    if compiler_flags.no_builtins {
        return;
    }

    statements.insert(
        1,
        Statement::Import { path: Token::new(
            Rc::from(source.as_ref()),
            Rc::clone(&filename),
            TokenType::Identifier,
            Rc::from(compiler_flags.primitives.as_ref()),
            0, 1, 0
        ), type_: ImportType::Default }
    );

    statements.insert(
        2,
        Statement::Import { path: Token::new(
            Rc::from(source.as_ref()),
            Rc::clone(&filename),
            TokenType::Identifier,
            Rc::from("core/builtins"),
            0, 1, 0
        ), type_: ImportType::Default }
    );

    if !compiler_flags.no_panic {
        statements.insert(
            3,
            Statement::Import { path: Token::new(
                Rc::from(source.as_ref()),
                filename,
                TokenType::Identifier,
                Rc::from("core/panic"),
                0, 1, 0
            ), type_: ImportType::Default }
        );
    }
}

pub fn compile(source: &String, path: Option<&Path>, filename: Rc<str>, compiler_flags: CompilerFlags, skye_path: PathBuf) -> Option<String> {
    let mut statements = parse(source, Rc::clone(&filename))?;
    prepare_base_imports(&mut statements, source, filename, &compiler_flags);

    let mut import_processor = ImportProcessor::new(path, skye_path.clone());
    import_processor.process(&mut statements);

    if import_processor.errors != 0 {
        return None;
    }

    let mut constant_folder = ConstantFolder::new();
    constant_folder.fold(&mut statements);

    if constant_folder.errors != 0 {
        return None;
    }

    let mut macro_expander = MacroExpander::new(compiler_flags.compile_mode);
    macro_expander.expand(&mut statements);

    if macro_expander.errors != 0 {
        return None;
    }

    constant_folder.reset();
    constant_folder.fold(&mut statements);

    if constant_folder.errors != 0 {
        return None;
    }

    let mut codegen = CodeGen::new(path, compiler_flags.compile_mode, skye_path);
    codegen.compile(statements);
    codegen.get_output()
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

pub fn compile_file(path: &OsStr, compiler_flags: CompilerFlags, skye_path: PathBuf) -> Result<String, Error> {
    let mut f = File::open(path)?;
    let mut input = String::new();
    f.read_to_string(&mut input)?;

    compile(&input, PathBuf::from(path).parent(), Rc::from(path.to_str().unwrap()), compiler_flags, skye_path)
        .ok_or(Error::other("Compilation failed"))
}

pub fn compile_file_to_c(input: &OsStr, output: &OsStr, compiler_flags: CompilerFlags, skye_path: PathBuf) -> Result<(), Error> {
    let code = compile_file(input, compiler_flags, skye_path)?;
    let mut f = File::create(output)?;
    f.write_all(code.as_bytes())?;
    Ok(())
}

pub fn basic_compile_c(input: &OsStr, output: &OsStr) -> Result<(), Error> {
    let mut needs_std = true;
    let mut command = {
        if cfg!(target_os = "macos") {
            Command::new("cc")
        } else if cfg!(unix) {
            // while c99 is in the posix standard, some platforms still don't support it,
            // using "cc" instead
            if Command::new("cc").arg("--version")
                .output()?.status.success()
            {
                Command::new("cc")
            } else {
                needs_std = false;
                Command::new("c99")
            }
        } else {
            Command::new(std::env::var("CC")
                .map_err(|e| Error::other(format!(
                    concat!(
                        "Could not find C compiler: {}\n",
                        "Is the CC environment variable set?"
                    ), e
                ).as_str()))?)
        }
    };

    if needs_std {
        command.arg("--std=c99");
    }

    command.arg("-w").arg(input);

    if cfg!(not(windows)) {
        command.arg("-lm");
    }

    command.arg("-o").arg(output);

    if !command.status()?.success() {
        return Err(Error::other("Build failed"));
    }

    Ok(())
}

pub fn compile_file_to_exec(input: &OsStr, output: &OsStr, compiler_flags: CompilerFlags, skye_path: PathBuf) -> Result<(), Error> {
    let buf = skye_path.join("tmp.c");
    let tmp_c = OsStr::new(buf.to_str().expect("Couldn't convert PathBuf to &str"));

    compile_file_to_c(input, tmp_c, compiler_flags, skye_path)?;
    println!("Skye compilation was successful. Calling C compiler...\n");
    basic_compile_c(tmp_c, output)?;
    remove_file(tmp_c)?;
    Ok(())
}

pub fn run_skye(file: OsString, program_args: &Option<Vec<String>>, compiler_flags: CompilerFlags, skye_path: PathBuf) -> Result<(), Error> {
    let buf = skye_path.join("tmp");
    let tmp = OsStr::new(buf.to_str().expect("Couldn't convert PathBuf to OsStr"));

    compile_file_to_exec(&file, &OsString::from(tmp), compiler_flags, skye_path)?;
    let mut com = Command::new(tmp);

    if let Some(args) = program_args {
        com.args(args);
    }

    com.status()?;
    remove_file(tmp)?;
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
