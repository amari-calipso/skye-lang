use std::{cell::RefCell, collections::HashMap, fmt::Debug, rc::Rc};

use crate::{type_system::SkyeType, parser::tokens::Token};

#[derive(Debug, Clone, PartialEq)]
pub struct SkyeVariable {
    pub type_: SkyeType,
    pub is_const: bool,
    pub tok: Option<Box<Token>>
}

impl SkyeVariable {
    pub fn new(type_: SkyeType, is_const: bool, tok: Option<Box<Token>>) -> Self {
        SkyeVariable { type_, is_const, tok }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Environment {
    types: HashMap<Rc<str>, SkyeVariable>,
    pub enclosing: Option<Rc<RefCell<Environment>>>
}

impl Environment {
    pub fn new() -> Self {
        Environment { types: HashMap::new(), enclosing: None }
    }

    pub fn with_enclosing(enclosing: Rc<RefCell<Environment>>) -> Self {
        Environment { types: HashMap::new(), enclosing: Some(enclosing) }
    }

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
}