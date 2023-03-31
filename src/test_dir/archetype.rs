use crate::test_dir::entity::*;
use crate::test_dir::component::*;
use std::sync::RwLock;
use std::any::{Any, TypeId};

/// An archetype stores entities with the same set of components.
#[doc(hidden)]
pub struct Archetype {
    pub(crate) entities: Vec<EntityId>,
    pub(crate) components: Vec<ComponentStore>,
}

impl Archetype {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
            components: Vec::new(),
        }
    }

    pub(crate) fn get<T: 'static>(&self, index: usize) -> &RwLock<Vec<T>> {
        self.components[index]
            .data
            .to_any()
            .downcast_ref::<RwLock<Vec<T>>>()
            .unwrap()
    }

    /// Returns the index of the entity moved
    pub fn remove_entity(&mut self, index: EntityId) -> EntityId {
        for c in self.components.iter_mut() {
            c.data.swap_remove(index)
        }

        let moved = *self.entities.last().unwrap();
        self.entities.swap_remove(index as usize);
        moved
    }

    pub fn mutable_component_store<T: 'static>(&mut self, component_index: usize) -> &mut Vec<T> {
        component_vec_to_mut(&mut *self.components[component_index].data)
    }

    pub fn replace_component<T: 'static>(&mut self, component_index: usize, index: EntityId, t: T) {
        self.mutable_component_store(component_index)[index as usize] = t;
    }

    pub fn push<T: 'static>(&mut self, component_index: usize, t: T) {
        self.mutable_component_store(component_index).push(t)
    }

    pub fn get_component_mut<T: 'static>(
        &mut self,
        index: EntityId,
    ) -> Result<&mut T, EntityMissingComponent> {
        let type_id = TypeId::of::<T>();
        let mut component_index = None;
        for (i, c) in self.components.iter().enumerate() {
            if c.type_id == type_id {
                component_index = Some(i);
                break;
            }
        }

        if let Some(component_index) = component_index {
            Ok(&mut self.mutable_component_store(component_index)[index as usize])
        } else {
            Err(EntityMissingComponent::new::<T>(index))
        }
    }

    /// Removes the component from an entity and pushes it to the other archetype
    /// The type does not need to be known to call this function.
    /// But the types of component_index and other_index need to match.
    pub fn migrate_component(
        &mut self,
        component_index: usize,
        entity_index: EntityId,
        other_archetype: &mut Archetype,
        other_index: usize,
    ) {
        self.components[component_index].data.migrate(
            entity_index,
            &mut *other_archetype.components[other_index].data,
        );
    }

    /// This takes a mutable reference so that the inner RwLock does not need to be locked
    /// by instead using get_mut.
    pub fn len(&mut self) -> usize {
        self.entities.len()
    }
}

