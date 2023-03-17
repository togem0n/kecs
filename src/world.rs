use crate::{
    ecs::*,
    entity_manager::*,
};

pub struct World
{
    entity_component_manager: EntityComponentSystem,
}

impl World
{
    /// Creates a new world from the given entity store.
    pub fn from_entity_store(entity_store: EntityManager) -> Self {
        World {
            entity_component_manager: EntityComponentSystem::new(entity_store),
        }
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
