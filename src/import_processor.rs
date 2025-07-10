use std::{collections::HashMap, ffi::OsString, path::{Path, PathBuf}, rc::Rc};

use crate::{ast::{Ast, ImportType, MacroBody, Statement}, astpos_note, parse_file, token_error, token_note, tokens::Token};

struct ImportInfo {
    pub token: Token,
    pub namespace: String,
}

pub struct ImportProcessor {
    source_path: Option<Box<PathBuf>>,
    skye_path:   PathBuf,

    curr_name: String,
    imports:   HashMap<PathBuf, ImportInfo>,

    in_function: bool,
    in_impl: bool,
    in_interface: bool,

    pub errors: usize,
}

impl ImportProcessor {
    pub fn new(path: Option<&Path>, skye_path: PathBuf) -> Self {
        ImportProcessor { 
            skye_path,
            imports: HashMap::new(),
            curr_name: String::new(),
            in_function: false,
            in_impl: false,
            in_interface: false,
            source_path: path.map(|x| Box::new(PathBuf::from(x))),
            errors: 0
        }
    }

    fn get_name(&self, name: &Rc<str>) -> Rc<str> {
        if self.curr_name == "" {
            Rc::clone(&name)
        } else {
            Rc::from(format!("{}_DOT_{}", self.curr_name, name))
        }
    }

    async fn process_one(&mut self, stmt: &mut Statement, ctx: &mut reblessive::Stk) {
        match stmt {
            Statement::Import { path: path_tok, type_, is_include } => {
                let mut path: PathBuf = path_tok.lexeme.split('/').collect();

                let skye_import = {
                    let fetched_extension = {
                        if let Some(extension) = path.extension() {
                            Some(OsString::from(extension))
                        } else {
                            None
                        }
                    };

                    if let Some(extension) = fetched_extension {
                        if *type_ == ImportType::Lib {
                            path = self.skye_path.join("lib").join(path)
                        } else if path.is_relative() && self.source_path.is_some() && *type_ != ImportType::Ang {
                            path = PathBuf::from((**self.source_path.as_ref().unwrap()).clone()).join(path);
                        } else {
                            path = path_tok.lexeme.split('/').collect();
                        }

                        extension == "skye"
                    } else if path.is_relative() {
                        path = self.skye_path.join("lib").join(path).with_extension("skye");
                        true
                    } else {
                        token_error!(self, path_tok, "A file extension is required on absolute path imports for Skye to know what kind of import to perform");
                        token_note!(path_tok, "Add the file extension (\".skye\", \".c\", \".h\", ...)");
                        return;
                    }
                };

                if !*is_include {
                    if let Some(info) = self.imports.get(&path) {
                        token_error!(self, path_tok, "Cannot import the same file multiple times");
                        token_note!(info.token, "This file was previously imported here");
                        token_note!(path_tok, "If this is intentional, use an 'include' statement instead of 'import'");

                        if info.namespace != self.curr_name {
                            let prefix = {
                                if self.curr_name == "" {
                                    ""
                                } else {
                                    "::"
                                }
                            };

                            let namespace = info.namespace.replace("_DOT_", "::");

                            token_note!(
                                info.token, 
                                format!(
                                    concat!(
                                        "This import was performed behind the \"{}\" namespace. ",
                                        "If you want to reuse it from this namespace, add \"use {}{}\" somewhere in your current namespace"
                                    ),
                                    namespace, prefix, namespace
                                ).as_str()
                            );
                        }

                        return;
                    }

                    self.imports.insert(path.clone(), ImportInfo { token: path_tok.clone(), namespace: self.curr_name.clone() });
                }

                if skye_import {
                    match parse_file(path.as_os_str()) {
                        Ok(mut statements) => {
                            ctx.run(|ctx| self.process_many(&mut statements, ctx)).await;
                            *stmt = Statement::ImportedBlock { statements, source: stmt.get_pos() };
                        }
                        Err(e) => {
                            token_error!(self, path_tok, format!("Could not import this file. Error: {}", e.to_string()).as_ref());
                        }
                    }
                }
            }
            Statement::Block(_, body) => {
                ctx.run(|ctx| self.process_many(body, ctx)).await;
            }
            Statement::ImportedBlock { statements, source } => {
                let old_errors = self.errors;

                ctx.run(|ctx| self.process_many(statements, ctx)).await;

                if self.errors != old_errors {
                    astpos_note!(source, "The error(s) were a result of this import");
                }
            }
            Statement::Namespace { name, body } => {
                let full_name = self.get_name(&name.lexeme);

                let previous_name = self.curr_name.clone();
                self.curr_name = full_name.to_string();
                ctx.run(|ctx| self.process_many(body, ctx)).await;
                self.curr_name = previous_name;
            }
            Statement::Impl { declarations: body, .. } => {
                let previous_impl = self.in_impl;
                self.in_impl = true;
                ctx.run(|ctx| self.process_many(body, ctx)).await;
                self.in_impl = previous_impl;
            }
            Statement::Function { body, .. } => {
                if let Some(body) = body {
                    let previous_fn = self.in_function;
                    self.in_function = true;
                    ctx.run(|ctx| self.process_many(body, ctx)).await;
                    self.in_function = previous_fn;
                }
            }
            Statement::Interface { declarations: body, .. } => {
                if let Some(body) = body {
                    let previous_interface = self.in_interface;
                    self.in_interface = true;
                    ctx.run(|ctx| self.process_many(body, ctx)).await;
                    self.in_interface = previous_interface;
                }
            }
            Statement::While { body, .. } | 
            Statement::DoWhile { body, .. } |
            Statement::For { body, .. } |
            Statement::Template { declaration: body, .. } |
            Statement::Defer { statement: body, .. } |
            Statement::Foreach { body, .. } => {
                ctx.run(|ctx| self.process_one(body, ctx)).await;
            }
            Statement::Switch { cases, .. } => {
                for case in cases {
                    ctx.run(|ctx| self.process_many(&mut case.code, ctx)).await;
                }
            }
            Statement::If { then_branch, else_branch, .. } => {
                ctx.run(|ctx| self.process_one(then_branch, ctx)).await;

                if let Some(else_branch) = else_branch {
                    ctx.run(|ctx| self.process_one(else_branch, ctx)).await;
                }
            }
            Statement::Macro { body, .. } => {
                if let MacroBody::Block(body) = body {
                    ctx.run(|ctx| self.process_many(body, ctx)).await;
                }
            }
            _ => ()
        }
    }

    async fn process_many(&mut self, statements: &mut Vec<Statement>, ctx: &mut reblessive::Stk) {
        for statement in statements {
            ctx.run(|ctx| self.process_one(statement, ctx)).await;
        }
    }

    pub fn process(&mut self, statements: &mut Vec<Statement>) {
        let mut stack = reblessive::Stack::new();
        stack.enter(|ctx| self.process_many(statements, ctx)).finish()
    }
}