use std::any::{Any, TypeId};
use std::collections::{hash_map::DefaultHasher, HashMap};
use std::hash::{Hash, Hasher};
use std::sync::RwLock;

pub(crate) type EntityId = u32;

pub trait Component: Sync + Send + 'static {}
impl<T: Sync + Send + 'static> Component for T {}

/// The ComponentVec trait is used to define a set of things that can be done on
/// an Any without knowing its exact type.
trait ComponentVec: Sync + Send {
    fn to_any(&self) -> &dyn Any;
    fn to_any_mut(&mut self) -> &mut dyn Any;
    fn len(&mut self) -> usize;
    fn swap_remove(&mut self, index: EntityId);
    fn migrate(&mut self, entity_index: EntityId, other_archetype: &mut dyn ComponentVec);
    fn new_same_type(&self) -> Box<dyn ComponentVec + Send + Sync>;
}

// This could be made unchecked in the future if there's a high degree of confidence in everything else.
fn component_vec_to_mut<T: 'static>(c: &mut dyn ComponentVec) -> &mut Vec<T> {
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

fn component_vec_to_mut<T: 'static>(c: &mut dyn ComponentVec) -> &mut Vec<T> {
    c.to_any_mut()
        .downcast_mut::<RwLock<Vec<T>>>()
        .unwrap()
        .get_mut()
        .unwrap()
}

// for a component type, store its instances
pub(crate) struct ComponentStore {
    pub(crate) type_id: TypeId,
    data: Box<dyn ComponentVec + Send + Sync>,
}

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

#[doc(hidden)]
/// An archetype stores entities with the same set of components type.
/// components are their real component instance
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

    // basically get components of the given entity
    pub(crate) fn get<T: 'static>(&self, index: usize) -> &RwLock<Vec<T>> {
        self.components[index]
            .data
            .to_any()
            .downcast_ref::<RwLock<Vec<T>>>()
            .unwrap()
    }

    /// Returns the index of the entity moved
    fn remove_entity(&mut self, index: EntityId) -> EntityId {
        for c in self.components.iter_mut() {
            c.data.swap_remove(index)
        }
        let moved = *self.entities.last().unwrap();
        self.entities.swap_remove(index as usize);
        moved
    }
    
    fn mutable_component_store<T: 'static>(&mut self, component_index: usize) -> &mut Vec<T> {
        component_vec_to_mut(&mut *self.components[component_index].data)
    }

    fn replace_component<T: 'static>(&mut self, component_index: usize, index: EntityId, t: T) {
        self.mutable_component_store(component_index)[index as usize] = t;
    }

    fn push<T: 'static>(&mut self, component_index: usize, t: T) {
        self.mutable_component_store(component_index).push(t)
    }

    // get component from component store for a given entity
    fn get_component_mut<T: 'static>(
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
    fn migrate_component(
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
    fn len(&mut self) -> usize {
        self.entities.len()
    }
}

/// An entity's location within Archetype array
#[derive(Debug, Clone, Copy)]
#[doc(hidden)]
pub struct EntityLocation {
    archetype_index: EntityId,   // index of archetype
    index_in_archetype: EntityId,// inside archetype array, index of their components
}

#[derive(Clone, Copy)]
pub(crate) struct EntityInfo {
    pub(crate) generation: EntityId,     // index of generation
    pub(crate) location: EntityLocation, // location info
}

/// A handle to an entity within the world.
/// this seems to be useless
#[derive(Debug, Clone, Copy, Hash, Eq, Ord, PartialEq, PartialOrd)]
pub struct Entity {
    pub(crate) index: EntityId,
    pub(crate) generation: EntityId,
}

// the given components bundle to a given entity
// like when we do world.spawn(42, true, Name{})
// the arechetype would be (42, true, Name{}) and along with its entity index
// when we create some entity same components type as u32, bool, Name struct
// it will be store in the same archetype?
pub trait ComponentBundle: 'static + Send + Sync {
    #[doc(hidden)]
    fn new_archetype(&self) -> Archetype;
    #[doc(hidden)]
    fn spawn_in_world(self, world: &mut World, entity_index: EntityId) -> EntityLocation;
}

// calculate bundle id throught the hash value of all components type
// then we get a unique id for the component bundle
fn calculate_bundle_id(types: &[TypeId]) -> u64 {
    let mut s = DefaultHasher::new();
    types.hash(&mut s);
    s.finish()
}

pub struct World {
    pub(crate) archetypes: Vec<Archetype>,
    bundle_id_to_archetype: HashMap<u64, usize>,
    pub(crate) entities: Vec<EntityInfo>,
    free_entities: Vec<EntityId>,
}

impl World {
    /// Create the world.
    pub fn new() -> Self {
        Self {
            archetypes: Vec::new(),
            bundle_id_to_archetype: HashMap::new(),
            entities: Vec::new(),
            free_entities: Vec::new(),
        }
    }

    /// Spawn an entity with components passed in through a tuple.
    /// Multiple components can be passed in through the tuple.
    /// # Example
    /// ```
    /// # use kudo::*;
    /// let mut world = World::new();
    /// let entity = world.spawn((456, true));
    /// ```
    pub fn spawn(&mut self, b: impl ComponentBundle) -> Entity {
        let (index, generation) = if let Some(index) = self.free_entities.pop() {
            let (generation, _) = self.entities[index as usize].generation.overflowing_add(1);
            (index, generation)
        } else {
            // Push placeholder data
            // here we push the first entity of the world
            self.entities.push(EntityInfo {
                location: EntityLocation {
                    archetype_index: 0,
                    index_in_archetype: 0,
                },
                generation: 0,
            });
            // Error if too many entities are allocated.
            debug_assert!(self.entities.len() <= EntityId::MAX as usize);
            ((self.entities.len() - 1) as EntityId, 0)
        };
        let location = b.spawn_in_world(self, index);
        self.entities[index as usize] = EntityInfo {
            location,
            generation: generation,
        };
        Entity { index, generation }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test_create_entity_with_component() {
//         let world = World::from_entity_store(EntityManager::default());
//     }
// }
