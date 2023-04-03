use std::any::{TypeId};
use std::collections::{hash_map::DefaultHasher, HashMap};
use std::hash::{Hash, Hasher};

use crate::entity::*;
use crate::component::*;
use crate::archetype::*;
use crate::query::*;
use crate::error::*;

/// The world holds all components and associated entities.
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

    // Spawn new entity
    pub fn spawn(&mut self, b: impl ComponentBundle) -> Entity {
        let (index, generation) = if let Some(index) = self.free_entities.pop() {
            let (generation, _) = self.entities[index as usize].generation.overflowing_add(1);
            (index, generation)
        } else {
            // Push placeholder data
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

    /// Remove an entity and all its components from the world.
    /// An error is returned if the entity does not exist.
    pub fn despawn(&mut self, entity: Entity) -> Result<(), NoSuchEntity> {
        // Remove an entity
        // Update swapped entity position if an entity was moved.
        let entity_info = self.entities[entity.index as usize];
        if entity_info.generation == entity.generation {
            self.entities[entity.index as usize].generation += 1;
            let moved_entity = self.archetypes[entity_info.location.archetype_index as usize]
                .remove_entity(entity_info.location.index_in_archetype);
            self.free_entities.push(entity.index);

            // Update the position of an entity that was moved.
            self.entities[moved_entity as usize].location = entity_info.location;

            Ok(())
        } else {
            Err(NoSuchEntity)
        }
    }

    /// Gets mutable access to a single component on an `Entity`.
    pub fn get_component_mut<T: 'static>(
        &mut self,
        entity: Entity,
    ) -> Result<&mut T, ComponentError> {
        let entity_info = self.entities[entity.index as usize];
        if entity_info.generation == entity.generation {
            let archetype = &mut self.archetypes[entity_info.location.archetype_index as usize];
            archetype
                .get_component_mut(entity_info.location.index_in_archetype)
                .map_err(|e| ComponentError::EntityMissingComponent(e))
        } else {
            // Entity no longer exists
            Err(ComponentError::NoSuchEntity(NoSuchEntity))
        }
    }

    pub fn remove_component<T: 'static>(&mut self, entity: Entity) -> Result<T, ComponentError> {
        let entity_info = self.entities[entity.index as usize];

        if entity_info.generation == entity.generation {
            let current_archetype = &self.archetypes[entity_info.location.archetype_index as usize];

            let type_id = TypeId::of::<T>();
            let mut type_ids: Vec<TypeId> = current_archetype
                .components
                .iter()
                .map(|c| c.type_id)
                .collect();
            let binary_search_index = type_ids.binary_search(&type_id);

            if let Ok(remove_index) = binary_search_index {
                type_ids.remove(remove_index);
                let bundle_id = calculate_bundle_id(&type_ids);
                let new_archetype_index = if let Some(new_archetype_index) =
                    self.bundle_id_to_archetype.get(&bundle_id)
                {
                    *new_archetype_index
                } else {
                    // Create a new archetype
                    let mut archetype = Archetype::new();
                    for c in current_archetype.components.iter() {
                        if c.type_id != type_id {
                            archetype.components.push(c.new_same_type());
                        }
                    }

                    let new_archetype_index = self.archetypes.len();

                    self.bundle_id_to_archetype
                        .insert(bundle_id, new_archetype_index);
                    self.archetypes.push(archetype);
                    new_archetype_index
                };

                // Much of this code is similar to the code for adding a component.

                // index_twice lets us mutably borrow from the world twice.
                let (old_archetype, new_archetype) = index_twice(
                    &mut self.archetypes,
                    entity_info.location.archetype_index as usize,
                    new_archetype_index,
                );

                // If an entity is being moved then update its location
                if let Some(last) = old_archetype.entities.last() {
                    self.entities[*last as usize].location = entity_info.location;
                }

                // First update the entity's location to reflect the changes about to be made.
                self.entities[entity.index as usize].location = EntityLocation {
                    archetype_index: new_archetype_index as EntityId,
                    index_in_archetype: (new_archetype.len()) as EntityId,
                };

                // The new archetype is the same as the old one but with one fewer components.
                for i in 0..remove_index {
                    old_archetype.migrate_component(
                        i,
                        entity_info.location.index_in_archetype,
                        new_archetype,
                        i,
                    );
                }

                let components_in_archetype = old_archetype.components.len();

                for i in (remove_index + 1)..components_in_archetype {
                    old_archetype.migrate_component(
                        i,
                        entity_info.location.index_in_archetype,
                        new_archetype,
                        i - 1,
                    );
                }

                old_archetype
                    .entities
                    .swap_remove(entity_info.location.index_in_archetype as usize);
                new_archetype.entities.push(entity.index);

                Ok(
                    component_vec_to_mut::<T>(&mut *old_archetype.components[remove_index].data)
                        .swap_remove(entity_info.location.index_in_archetype as usize),
                )
            } else {
                // Component is not in entity
                Err(ComponentError::EntityMissingComponent(
                    EntityMissingComponent::new::<T>(entity.index),
                ))
            }
        } else {
            // Entity is not in world
            Err(ComponentError::NoSuchEntity(NoSuchEntity))
        }
    }

    /// Adds a component to an entity.
    /// If the component already exists its data will be replaced.
    pub fn add_component<T: 'static + Send + Sync>(
        &mut self,
        entity: Entity,
        t: T,
    ) -> Result<(), NoSuchEntity> {
        // In an archetypal ECS adding and removing components are the most expensive operations.
        // The volume of code in this function reflects that.
        // When a component is added the entity can be either migrated to a brand new archetype
        // or migrated to an existing archetype.

        // First find if the entity exists
        let entity_info = self.entities[entity.index as usize];
        if entity_info.generation == entity.generation {
            let type_id = TypeId::of::<T>();

            // First check if the component already exists for this entity.
            let current_archetype = &self.archetypes[entity_info.location.archetype_index as usize];

            let mut type_ids: Vec<TypeId> = current_archetype
                .components
                .iter()
                .map(|c| c.type_id)
                .collect();
            let binary_search_index = type_ids.binary_search(&type_id);

            if let Ok(insert_index) = binary_search_index {
                // The component already exists, replace it.
                let current_archetype =
                    &mut self.archetypes[entity_info.location.archetype_index as usize];

                // Replace the existing component.
                current_archetype.replace_component(
                    insert_index,
                    entity_info.location.index_in_archetype,
                    t,
                );
            } else {
                // The component does not already exist in the current archetype.
                // Find an existing archetype to migrate to or create a new archetype

                let insert_index = binary_search_index.unwrap_or_else(|i| i);

                type_ids.insert(insert_index, type_id);
                let bundle_id = calculate_bundle_id(&type_ids);

                let new_archetype_index = if let Some(new_archetype_index) =
                    self.bundle_id_to_archetype.get(&bundle_id)
                {
                    // Found an existing archetype to migrate data to
                    *new_archetype_index
                } else {
                    // Create a new archetype with the structure of the current archetype and one additional component.
                    let mut archetype = Archetype::new();
                    for c in current_archetype.components.iter() {
                        archetype.components.push(c.new_same_type());
                    }

                    let new_archetype_index = self.archetypes.len();
                    archetype
                        .components
                        .insert(insert_index, ComponentStore::new::<T>());
                    self.bundle_id_to_archetype
                        .insert(bundle_id, new_archetype_index);

                    self.archetypes.push(archetype);

                    new_archetype_index
                };

                // index_twice lets us mutably borrow from the world twice.
                let (old_archetype, new_archetype) = index_twice(
                    &mut self.archetypes,
                    entity_info.location.archetype_index as usize,
                    new_archetype_index,
                );

                // If an entity is being moved then update its location
                if let Some(last) = old_archetype.entities.last() {
                    self.entities[*last as usize].location = entity_info.location;
                }

                // First update the entity's location to reflect the changes about to be made.
                self.entities[entity.index as usize].location = EntityLocation {
                    archetype_index: new_archetype_index as EntityId,
                    index_in_archetype: (new_archetype.len()) as EntityId,
                };

                // The new archetype is the same as the old one but with one additional component.
                for i in 0..insert_index {
                    old_archetype.migrate_component(
                        i,
                        entity_info.location.index_in_archetype,
                        new_archetype,
                        i,
                    );
                }

                // Push the new component to the new archetype
                new_archetype.push(insert_index, t);

                let components_in_archetype = old_archetype.components.len();

                for i in insert_index..components_in_archetype {
                    old_archetype.migrate_component(
                        i,
                        entity_info.location.index_in_archetype,
                        new_archetype,
                        i + 1,
                    );
                }

                old_archetype
                    .entities
                    .swap_remove(entity_info.location.index_in_archetype as usize);
                new_archetype.entities.push(entity.index);
            }

            Ok(())
        } else {
            Err(NoSuchEntity)
        }
    }

    /// Query for an immutable reference to the first instance of a component found.
    pub fn get_single<T: 'static>(&self) -> Result<Single<T>, FetchError> {
        <&T>::fetch(self)
    }

    /// Query for a mutable reference to the first instance of a component found.
    pub fn get_single_mut<T: 'static>(&self) -> Result<SingleMut<T>, FetchError> {
        <&mut T>::fetch(self)
    }

    /// Get a query from the world.
    /// # Example
    /// ```
    /// # use kudo::*;
    /// # let mut world = World::new();
    /// let query = world.query<(&bool, &String)>();
    /// ```
    pub fn query<'world_borrow, T: QueryParameters>(
        &'world_borrow self,
    ) -> Result<Query<T>, FetchError> {
        Ok(QueryFetch::<T>::fetch(self)?.take().unwrap())
    }

}

/// A bundle of Component 
/// this shouldnt belongs to world.rs, no?
pub trait ComponentBundle: 'static + Send + Sync {
    #[doc(hidden)]
    fn new_archetype(&self) -> Archetype;
    #[doc(hidden)]
    fn spawn_in_world(self, world: &mut World, entity_index: EntityId) -> EntityLocation;
}

fn calculate_bundle_id(types: &[TypeId]) -> u64 {
    let mut s = DefaultHasher::new();
    types.hash(&mut s);
    s.finish()
}

/// A helper to get two mutable borrows from the same slice.
fn index_twice<T>(slice: &mut [T], first: usize, second: usize) -> (&mut T, &mut T) {
    if first < second {
        let (a, b) = slice.split_at_mut(second);
        (&mut a[first], &mut b[0])
    } else {
        let (a, b) = slice.split_at_mut(first);
        (&mut b[0], &mut a[second])
    }
}

macro_rules! component_bundle_impl {
    ($count: expr, $(($name: ident, $index: tt)),*) => {
        impl< $($name: 'static + Send + Sync),*> ComponentBundle for ($($name,)*) {
            fn new_archetype(&self) -> Archetype {
                let mut components = vec![$(ComponentStore::new::<$name>()), *];
                components.sort_unstable_by(|a, b| a.type_id.cmp(&b.type_id));
                Archetype { components, entities: Vec::new() }
            }

            fn spawn_in_world(self, world: &mut World, entity_index: EntityId) -> EntityLocation {
                let mut types = [$(($index, TypeId::of::<$name>())), *];
                types.sort_unstable_by(|a, b| a.1.cmp(&b.1));
                debug_assert!(
                    types.windows(2).all(|x| x[0].1 != x[1].1),
                    "`ComponentBundle`s cannot have duplicate types"
                );

                // Is there a better way to map the original ordering to the sorted ordering?
                let mut order = [0; $count];
                for i in 0..order.len() {
                    order[types[i].0] = i;
                }
                let types = [$(types[$index].1), *];

                let bundle_id = calculate_bundle_id(&types);

                // Find the appropriate archetype
                // If it doesn't exist create a new archetype.
                let archetype_index = if let Some(archetype) = world.bundle_id_to_archetype.get(&bundle_id) {
                    *archetype
                } else {
                    let archetype = self.new_archetype();
                    let index = world.archetypes.len();

                    world.bundle_id_to_archetype.insert(bundle_id, index);
                    world.archetypes.push(archetype);
                    index
                };

                world.archetypes[archetype_index].entities.push(entity_index);
                $(world.archetypes[archetype_index].push(order[$index], self.$index);)*
                EntityLocation {
                    archetype_index: archetype_index as EntityId,
                    index_in_archetype: (world.archetypes[archetype_index].len() - 1) as EntityId
                }
            }
        }
    }
}

component_bundle_impl! {1, (A, 0)}
component_bundle_impl! {2, (A, 0), (B, 1)}
component_bundle_impl! {3, (A, 0), (B, 1), (C, 2)}
component_bundle_impl! {4, (A, 0), (B, 1), (C, 2), (D, 3)}
component_bundle_impl! {5, (A, 0), (B, 1), (C, 2), (D, 3), (E, 4)}
component_bundle_impl! {6, (A, 0), (B, 1), (C, 2), (D, 3), (E, 4), (F, 5)}
component_bundle_impl! {7, (A, 0), (B, 1), (C, 2), (D, 3), (E, 4), (F, 5), (G, 6)}
component_bundle_impl! {8, (A, 0), (B, 1), (C, 2), (D, 3), (E, 4), (F, 5), (G, 6), (H, 7)}
component_bundle_impl! {9, (A, 0), (B, 1), (C, 2), (D, 3), (E, 4), (F, 5), (G, 6), (H, 7), (I, 8)}
component_bundle_impl! {10, (A, 0), (B, 1), (C, 2), (D, 3), (E, 4), (F, 5), (G, 6), (H, 7), (I, 8), (J, 9)}
component_bundle_impl! {11, (A, 0), (B, 1), (C, 2), (D, 3), (E, 4), (F, 5), (G, 6), (H, 7), (I, 8), (J, 9), (K, 10)}
component_bundle_impl! {12, (A, 0), (B, 1), (C, 2), (D, 3), (E, 4), (F, 5), (G, 6), (H, 7), (I, 8), (J, 9), (K, 10), (L, 11)}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_spawn_entities() {
        let mut world = World::new();
        struct Name {}
        struct Move {}
        let entity_0 = world.spawn((21, true));
        let entity_1 = world.spawn((2, false));
        let entity_2 = world.spawn((2, true));
        let entity_3 = world.spawn((5, true));
        let entity_4 = world.spawn((21, Name{}, Move{}));
        println!("{:?}", entity_0);
        println!("{:?}", entity_1);
        println!("{:?}", entity_2);
        println!("{:?}", entity_3);
        println!("{:?}", entity_4);
        println!("Num of archetypes: {}", world.archetypes.len());
        println!("Num of entities: {}", world.entities.len());
        for e in world.entities {
            println!("{:?}", e);
        }
    }

    #[test]
    fn test_world_add_component() {
        let mut world = World::new();
        #[derive(Debug)]
        #[allow(dead_code)]
        struct Name {id: u32, name: String}
        #[derive(Debug)]
        #[allow(dead_code)]
        struct Move {dir: u32, speed: f32}
        let entity_0 = world.spawn((21, true)); 
        println!("{:?}", entity_0);
        let name0 = world.add_component(entity_0, Name{id: 130, name: String::from("nmsl")});
        let move0 = world.add_component(entity_0, Move{dir: 130, speed: 130.0});
        println!("{:?}", name0);
        println!("{:?}", move0);
        for e in world.entities {
            println!("{:?}", e);
        }
    }

    #[test]
    fn test_world_query() {
        let mut world = World::new();
        #[derive(Debug)]
        #[allow(dead_code)]
        struct Name {id: u32, name: String}
        #[derive(Debug)]
        #[allow(dead_code)]
        struct Move {dir: u32, speed: f32}
        let entity_0 = world.spawn((21, true, Name {id: 130, name: String::from("nmsl_0")}));
        let entity_1 = world.spawn((21, true, Name {id: 140, name: String::from("nmsl_1")}));
        let entity_2 = world.spawn((21, true, Name {id: 150, name: String::from("nmsl_2")}));
        let entity_3 = world.spawn((21, true, Name {id: 160, name: String::from("nmsl_3")}));
        let entity_4 = world.spawn((21, true, Name {id: 170, name: String::from("nmsl_4")}));
        let mut query = world.query::<(&Name, &bool)>().unwrap();
        // Iterate over all the components we found and check if their health is below 0.
        for component in query.iter() {
            println!("{}, {}", component.0.id, component.0.name);
        }
    }
}

