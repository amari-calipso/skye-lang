use std::rc::Rc;

use enum_dispatch::enum_dispatch;

use crate::ir::IrStatement;

#[enum_dispatch(AnyBackend)]
pub trait Backend {
    fn name(&self) -> Rc<str>;
    fn generate(&mut self, definitions: Vec<IrStatement>) -> Option<String>;
}