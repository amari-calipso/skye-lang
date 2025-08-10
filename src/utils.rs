use std::{collections::HashMap, rc::Rc};

use crate::{ast::Expression, tokens::Token};

#[macro_export]
macro_rules! get_expect {
    ($obj: expr) => {
        $obj.get().expect(concat!("Could not get Once expression: ", stringify!($obj)))
    };
}

#[macro_export]
macro_rules! dot {
    () => {
        crate::get_expect!(crate::NAMESPACE_SEP)
    };
}

#[derive(Debug, Clone, PartialEq)]
pub struct OrderedNamedMap<T> {
    pub map: HashMap<Rc<str>, T>,
    pub order: Vec<Rc<str>>
}

impl<T> OrderedNamedMap<T> {
    pub fn new() -> Self {
        Self { 
            map: HashMap::new(), 
            order: Vec::new() 
        }
    }

    pub fn get(&self, k: &str) -> Option<&T> {
        self.map.get(k)
    }

    pub fn contains_key(&self, k: &str) -> bool {
        self.map.contains_key(k)
    }

    pub fn len(&self) -> usize {
        let len = self.order.len();
        debug_assert_eq!(len, self.map.len());
        len
    }

    pub fn insert(&mut self, k: Rc<str>, v: T) -> Option<T> {
        let old = self.map.insert(Rc::clone(&k), v);

        // if the key was already there, removal is O(n). 
        // this shouldn't be used where you expect duplicates (which we don't)
        if old.is_some() {
            self.order.retain(|x| x.as_ref() != k.as_ref());
        }

        self.order.push(k);
        old
    }
}

pub fn escape_string(str: &str) -> String {
    str.replace('\\', "\\\\")
}

pub fn literal_as_string(expr: Expression) -> Option<(Rc<str>, Token)> {
     match expr {
        Expression::StringLiteral { value, tok, .. } => Some((value, tok)),
        Expression::VoidLiteral(tok) => Some((Rc::from(""), tok)),
        Expression::SignedIntLiteral { value, tok, .. } => Some((value.to_string().into(), tok)),
        Expression::UnsignedIntLiteral { value, tok, .. } => Some((value.to_string().into(), tok)),
        _ => None
    }
}