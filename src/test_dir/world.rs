use std::any::{Any, TypeId};
use std::collections::{hash_map::DefaultHasher, HashMap};
use std::hash::{Hash, Hasher};
use std::sync::RwLock;


use crate::test_dir::entity::*;
use crate::test_dir::component::*;
use crate::test_dir::archetype::*;

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
/// A bundle of components
/// Used to spawn new
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
    fn tust_world_spawn() {
        let mut world = World::new();
        let first_entity = world.spawn((true, 21));
        for e in world.entities {
            println!("nmsl");
        }
    }
}

