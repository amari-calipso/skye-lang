use std::{collections::HashMap, ffi::OsString, path::{Path, PathBuf}, rc::Rc};

use alanglib::{ast::WithPosition, error, report::{note, note_pos, warning}};

use crate::{ast::{ImportType, MacroBody, Statement}, dot, parse_file, tokens::Token};

pub struct ImportProcessor {
    source_path: Option<Box<PathBuf>>,
    skye_path:   PathBuf,

    curr_name: String,
    imports: HashMap<PathBuf, HashMap</* namespace */ String, Token>>,

    pub errors: usize,
}

impl ImportProcessor {
    pub fn new(path: Option<&Path>, skye_path: PathBuf) -> Self {
        ImportProcessor { 
            skye_path,
            imports: HashMap::new(),
            curr_name: String::new(),
            source_path: path.map(|x| Box::new(PathBuf::from(x))),
            errors: 0
        }
    }

    fn get_name(&self, name: &Rc<str>) -> Rc<str> {
        if self.curr_name == "" {
            Rc::clone(&name)
        } else {
            Rc::from(format!("{}{}{}", self.curr_name, dot!(), name))
        }
    }

    async fn process_one(&mut self, stmt: &mut Statement, ctx: &mut reblessive::Stk) {
        match stmt {
            Statement::Import { path: path_tok, type_, is_include, flags } => {
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
                        error!(self, path_tok, "A file extension is required on absolute path imports for Skye to know what kind of import to perform");
                        note(path_tok, "Add the file extension (\".skye\", \".c\", \".h\", ...)");
                        return;
                    }
                };

                if skye_import {
                    if flags.len() != 0 {
                        error!(self, flags.first().unwrap(), "Cannot use flags in a Skye import");
                        note(path_tok, "Flags are meant to set functionality for C header-only libraries");
                    }
                } else {
                    if self.curr_name != "" {
                        warning(path_tok, "C imports cannot be namespaced. This import will be performed in the global namespace");
                    }
                }

                if !*is_include {
                    if let Some(info) = self.imports.get_mut(&path) {
                        warning(path_tok, "A duplicate import was performed");

                        for (_, import_tok) in info.iter() {
                            note(import_tok, "This file was previously imported here");
                        }
                        
                        note(path_tok, "If this is intentional, use an 'include' statement instead of 'import', otherwise remove this import");

                        if !skye_import || info.contains_key(&self.curr_name) {
                            // if this import was previously performed in the same namespace as the current one, no need to perform the import again
                            *stmt = Statement::Empty;
                            return;
                        } 

                        if skye_import {
                            if info.len() == 1 {
                                note(
                                    path_tok, 
                                    concat!(
                                        "The previous import was performed behind another namespace. ",
                                        "It is recommended to specify full paths or add \"use\" statements instead of importing the file again"
                                    )
                                );
                            } else {
                                note(
                                    path_tok, 
                                    concat!(
                                        "The previous imports were performed behind other namespaces. ",
                                        "It is recommended to specify full paths or add \"use\" statements instead of importing the file again"
                                    )
                                );
                            }
                        }
                        
                        info.insert(self.curr_name.clone(), path_tok.clone());
                    } else {
                        self.imports.insert(path.clone(), HashMap::from([(self.curr_name.clone(), path_tok.clone())]));
                    }
                }

                if skye_import {
                    match parse_file(path.as_os_str()) {
                        Ok(mut statements) => {
                            let old_errors = self.errors;

                            ctx.run(|ctx| self.process_many(&mut statements, ctx)).await;

                            if self.errors != old_errors {
                                note(stmt, "The error(s) were a result of this import");
                            }

                            *stmt = Statement::ImportedBlock { statements, source: stmt.get_pos() };
                        }
                        Err(e) => {
                            error!(self, path_tok, format!("Could not import file {:?}. Error: {}", path, e.to_string()).as_ref());
                        }
                    }
                }
            }
            Statement::Block(_, body) | 
            Statement::Impl { declarations: body, .. } => {
                ctx.run(|ctx| self.process_many(body, ctx)).await;
            }
            Statement::ImportedBlock { statements, source } => {
                let old_errors = self.errors;

                ctx.run(|ctx| self.process_many(statements, ctx)).await;

                if self.errors != old_errors {
                    note_pos(source, "The error(s) were a result of this import");
                }
            }
            Statement::Namespace { name, body } => {
                let full_name = self.get_name(&name.lexeme);

                let previous_name = self.curr_name.clone();
                self.curr_name = full_name.to_string();
                ctx.run(|ctx| self.process_many(body, ctx)).await;
                self.curr_name = previous_name;
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