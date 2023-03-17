use crate::entity::*;
use crate::entity_manager::*;
use crate::component_manager::*;
use crate::entity_builder::*;

#[derive(Default)]
pub struct EntityComponentSystem
{
    component_manager: ComponentManager,
    entity_manager: EntityManager,
    entity_counter: u32,
}

impl EntityComponentSystem
{
    pub fn new() -> Self {
        EntityComponentSystem { 
            component_manager: ComponentManager::default(),
            entity_manager: EntityManager::default(),  
            entity_counter: 0 
        }
    }

    pub fn entity_manager(&self) -> &EntityManager {
        &self.entity_manager
    }

    pub fn entity_manager_mut(&mut self) -> &mut EntityManager {
        &mut self.entity_manager
    }

    pub fn component_manager(&self) -> &ComponentManager {
        &self.component_manager
    }

    pub fn component_manager_mut(&mut self) -> &mut ComponentManager {
        &mut self.component_manager
    }

    pub fn create_entity(&mut self) -> EntityBuilder<'_> {
        let entity: Entity = self.entity_counter.into();
        self.entity_counter += 1;

        EntityBuilder {
            entity, 
            entity_manager: &mut self.entity_manager, 
            component_manager: &mut self.component_manager,
        }

    }
} 


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_entity_with_component() {
    }
}
