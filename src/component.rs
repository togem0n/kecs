use std::any::{Any, TypeId};
use std::sync::RwLock;

use crate::entity::*;

pub trait Component: Sync + Send + 'static {}
impl<T: Sync + Send + 'static> Component for T {}

/// The ComponentVec trait is used to define a set of things that can be done on
/// an Any without knowing its exact type.
pub trait ComponentVec: Sync + Send {
    fn to_any(&self) -> &dyn Any;
    fn to_any_mut(&mut self) -> &mut dyn Any;
    fn len(&mut self) -> usize;
    fn swap_remove(&mut self, index: EntityId);
    fn migrate(&mut self, entity_index: EntityId, other_archetype: &mut dyn ComponentVec);
    fn new_same_type(&self) -> Box<dyn ComponentVec + Send + Sync>;
}

// This could be made unchecked in the future if there's a high degree of confidence in everything else.
pub fn component_vec_to_mut<T: 'static>(c: &mut dyn ComponentVec) -> &mut Vec<T> {
    c.to_any_mut()
        .downcast_mut::<RwLock<Vec<T>>>()
        .unwrap()
        .get_mut()
        .unwrap()
}

impl<T: Component> ComponentVec for RwLock<Vec<T>> {
    fn to_any(&self) -> &dyn Any {
        self
    }
    fn to_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn len(&mut self) -> usize {
        // Perhaps this call to read incurs unnecessary overhead.
        self.get_mut().unwrap().len()
    }

    fn swap_remove(&mut self, index: EntityId) {
        self.get_mut().unwrap().swap_remove(index as usize);
    }

    fn migrate(&mut self, entity_index: EntityId, other_component_vec: &mut dyn ComponentVec) {
        let data: T = self.get_mut().unwrap().swap_remove(entity_index as usize);
        component_vec_to_mut(other_component_vec).push(data);
    }

    fn new_same_type(&self) -> Box<dyn ComponentVec + Send + Sync> {
        Box::new(RwLock::new(Vec::<T>::new()))
    }
}

/// Stores components for a component type
pub(crate) struct ComponentStore {
    pub(crate) type_id: TypeId,
    pub data: Box<dyn ComponentVec + Send + Sync>,
}

#[allow(dead_code)]
impl ComponentStore {
    pub fn new<T: 'static + Send + Sync>() -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            data: Box::new(RwLock::new(Vec::<T>::new())),
        }
    }

    /// Creates a new ComponentStore with the same internal storage type as self
    pub fn new_same_type(&self) -> Self {
        Self {
            type_id: self.type_id,
            data: self.data.new_same_type(),
        }
    }
}
