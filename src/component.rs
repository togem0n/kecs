use std::any::Any;

pub trait Component: Any {}
impl<E: Any> Componene for E {}
