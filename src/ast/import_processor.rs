use std::{ffi::OsString, path::{Path, PathBuf}};

use crate::{ast::{Ast, ImportType, MacroBody, Statement}, astpos_note, parse_file, token_error, token_note};

pub struct ImportProcessor {
    source_path: Option<Box<PathBuf>>,
    skye_path:   PathBuf,

    pub errors: usize,
}

impl ImportProcessor {
    pub fn new(path: Option<&Path>, skye_path: PathBuf) -> Self {
        ImportProcessor { 
            skye_path,
            source_path: path.map(|x| Box::new(PathBuf::from(x))),
            errors: 0
        }
    }

    async fn process_one(&mut self, stmt: &mut Statement, ctx: &mut reblessive::Stk) {
        match stmt {
            Statement::Import { path: path_tok, type_ } => {
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
            Statement::Block(_, body) |  
            Statement::Impl { declarations: body, .. } | 
            Statement::Namespace { body, .. } => {
                ctx.run(|ctx| self.process_many(body, ctx)).await;
            }
            Statement::ImportedBlock { statements, source } => {
                let old_errors = self.errors;

                ctx.run(|ctx| self.process_many(statements, ctx)).await;

                if self.errors != old_errors {
                    astpos_note!(source, "The error(s) were a result of this import");
                }
            }
            Statement::Function { body, .. } |
            Statement::Interface { declarations: body, .. } => {
                if let Some(body) = body {
                    ctx.run(|ctx| self.process_many(body, ctx)).await;
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