use crate::{
    component::*,
    entity::*,
    ecs::*,
    component_manager::*,
    entity_manager::*,
};

pub struct World<E: 'static>
where
    E: EntityManager
{
    entity_component_manager: EntityComponentSystem<E>,
}

impl<E> World<E>
where
    E: EntityManager,
{
    /// Creates a new world from the given entity store.
    pub fn from_entity_store(entity_store: E) -> Self {
        World {
            entity_component_manager: EntityComponentSystem::new(entity_store),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_entity_with_component() {
        let world = World::from_entity_store(EntityManager::default());
    }
}
