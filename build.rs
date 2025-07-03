use std::{fs::{read_dir, File}, io::{Error, Write}, path::{Path, PathBuf}};

use convert_case::{Case, Casing};

const INDENT_SIZE: usize = 4;
const DEFAULT_BACKEND: &str = "C";

macro_rules! log {
    ($($tokens: tt)*) => {
        println!("cargo:warning={}", format_args!($($tokens)*))
    };
}

macro_rules! info {
    ($($tokens: tt)*) => {
        log!("INFO: {}", format_args!($($tokens)*))
    };
}

const BACKENDS_SUM_TYPE_HEADER: &str = concat!(
    "use crate::*;\n\n",
    "#[enum_dispatch::enum_dispatch]\n",
    "pub enum AnyBackend {\n"
);

const BACKEND_OPTION_ENUM_HEADER: &str = concat!(
    "#[derive(clap::ValueEnum, Clone, Copy, Default, Debug, PartialEq, Eq, Hash)]\n",
    "pub enum BackendOption {\n"
);

fn push_indent(str: &mut String, level: usize) {
    for _ in 0 .. level * INDENT_SIZE {
        str.push(' ');
    }
}

fn generate_backends_module(folder: &str, src: &Path) -> Result<(), Error> {
    let mut mods     = String::new();
    let mut imports  = String::new();
    let mut sum_type = String::from(BACKENDS_SUM_TYPE_HEADER);
    let mut backend_option = String::from(BACKEND_OPTION_ENUM_HEADER);

    let mut load_fn = String::from("pub fn load() -> std::collections::HashMap<BackendOption, AnyBackend> {\n");
    push_indent(&mut load_fn, 1);
    load_fn.push_str("let mut output = std::collections::HashMap::new();\n");

    let folder_path = src.join(folder);

    let mut count = 0;
    for file in read_dir(&folder_path)? {
        let path = PathBuf::from(file?.file_name());
        let stem = path.file_stem()
            .expect("Invalid backend file name")
            .to_str()
            .expect("Invalid backend file name");

        if let Some(ext) = path.extension() {
            if ext != "rs" || stem == "mod" {
                continue;
            }
        } else {
            continue;
        }

        info!("Found backend \"{}\"", stem);
        count += 1;

        mods.push_str("mod ");
        mods.push_str(stem);
        mods.push_str(";\n");

        let pascal_name = stem.to_case(Case::Pascal);

        imports.push_str("use ");
        imports.push_str(stem);
        imports.push_str("::");
        imports.push_str(&pascal_name);
        imports.push_str(";\n");

        push_indent(&mut sum_type, 1);
        sum_type.push_str(&pascal_name);
        sum_type.push_str(",\n");

        push_indent(&mut backend_option, 1);

        if pascal_name == DEFAULT_BACKEND {
            backend_option.push_str("#[default] ");
        }

        backend_option.push_str(&pascal_name);
        backend_option.push_str(",\n");

        push_indent(&mut load_fn, 1);
        load_fn.push_str("output.insert(BackendOption::");
        load_fn.push_str(&pascal_name);
        load_fn.push_str(", ");
        load_fn.push_str(&pascal_name);
        load_fn.push_str("::new().into());\n");
    }

    if count == 0 {
        return Err(Error::other("No backend module found"));
    }

    sum_type.push_str("}\n");
    backend_option.push_str("}\n");

    push_indent(&mut load_fn, 1);
    load_fn.push_str("output\n}");

    let mut f = File::create(folder_path.join("mod.rs"))?;
    f.write_all(format!("{}\n{}\n{}\n{}\n{}", mods, imports, sum_type, backend_option, load_fn).as_bytes())?;

    Ok(())
}

fn main() -> Result<(), Error> {
    let src = PathBuf::from("src");

    info!("Generating backends module");
    generate_backends_module("backends", &src)?;

    info!("Building Skye");
    Ok(())
}