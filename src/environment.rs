use std::{cell::RefCell, collections::HashMap, fmt::Debug, rc::Rc};

use crate::{skye_type::{SkyeType, ValueFrom}, tokens::Token};

#[derive(Debug, Clone, PartialEq)]
pub struct SkyeVariable {
    pub type_: SkyeType,
    pub is_const: bool,
    pub from: ValueFrom,
    pub tok: Option<Box<Token>>
}

impl SkyeVariable {
    pub fn new(type_: SkyeType, is_const: bool, tok: Option<Box<Token>>) -> Self {
        SkyeVariable { type_, is_const, tok, from: ValueFrom::Default }
    }

    pub fn with_from(type_: SkyeType, is_const: bool, tok: Option<Box<Token>>, from: ValueFrom) -> Self {
        SkyeVariable { type_, is_const, tok, from }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Environment {
    types: HashMap<Rc<str>, SkyeVariable>,
    pub enclosing: Option<Rc<RefCell<Environment>>>,
    pub is_function: bool
}

impl Environment {
    pub fn new() -> Self {
        Environment { types: HashMap::new(), enclosing: None, is_function: false }
    }

    pub fn with_enclosing(enclosing: Rc<RefCell<Environment>>) -> Self {
        Environment { types: HashMap::new(), enclosing: Some(enclosing), is_function: false }
    }

    pub fn function(enclosing: Rc<RefCell<Environment>>) -> Self {
        Environment { types: HashMap::new(), enclosing: Some(enclosing), is_function: true }
    }

    #[allow(dead_code)] // useful for debugging
    pub fn iter_local(&self) -> HashMap<Rc<str>, SkyeVariable> {
        self.types.clone()
    }

    pub fn define(&mut self, name: Rc<str>, var: SkyeVariable) {
        self.types.insert(name, var);
    }

    pub fn undef(&mut self, name: Rc<str>) -> Option<SkyeVariable> {
        self.types.remove(&name)
    }

    pub fn get(&self, name: &Token) -> Option<SkyeVariable> {
        if let Some(var) = self.types.get(&name.lexeme) {
            Some(var.clone())
        } else if let Some(enclosing) = &self.enclosing {
            enclosing.borrow().get(name)
        } else {
            None
        }
    }

    pub fn get_in_scope(&self, name: &Token) -> Option<SkyeVariable> {
        if let Some(var) = self.types.get(&name.lexeme) {
            Some(var.clone())
        } else {
            None
        }
    }

    pub fn get_in_fn_scope(&self, name: &Token) -> Option<SkyeVariable> {
        if let Some(var) = self.types.get(&name.lexeme) {
            return Some(var.clone());
        } else if !self.is_function {
            if let Some(enclosing) = &self.enclosing {
                return enclosing.borrow().get_in_fn_scope(name);
            }
        }

        None
    }
}