use std::any::Any;

pub trait Component: Any {}
impl<E: Any> Component for E {}
