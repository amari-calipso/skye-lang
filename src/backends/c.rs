use std::rc::Rc;

use crate::{backend::Backend, ir::IrStatement};

pub struct C {

}

impl C {
    pub fn new() -> Self {
        todo!()
    }
}

impl Backend for C {
    fn name(&self) -> Rc<str> {
        Rc::from("c")
    }

    fn generate(&mut self, definitions: Vec<IrStatement>) -> Option<String> {
        todo!()
    }
}