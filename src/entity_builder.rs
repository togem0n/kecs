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
    pub fn with<C: Component>(self, key: &str, component: C) -> Self {
        // here should be: component_manager.append(self.entity, component)
        self.component_manager.append(self.entity, key, component); 
        self
    }

    pub fn build(self) -> Entity {
        self.entity_manager.add();
        // here is when ecs system add the entity and component
        self.entity
    }
}
