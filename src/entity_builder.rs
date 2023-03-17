use crate::entity::*;
use crate::entity_manager::*;
use crate::component::*;
use crate::component_manager::*;

// will use component manager module
pub struct EntityBuilder<'a>
{
    pub entity: Entity,
    
    pub entity_manager: &'a mut EntityManager, // if a type is a trait, then you need dyn included as it's generic 

    pub component_manager: &'a mut ComponentManager,
}

impl<'a> EntityBuilder<'a>
{
    pub fn with<C: Component>(self, component: C) -> Self {
        self.component_manager.add_component_to_entity(self.entity, component); 
        self
    }

    pub fn build(self) -> Entity {
        self.entity_manager.add();
        // here is when ecs system add the entity and component
        self.entity
    }
}
