use core::any::Any;
use fxhash::FxHashMap;

use crate::entity::*;
use crate::entity_manager::*;
use crate::component::*;
use crate::component_manager::*;

// will use component manager module
pub struct EntityBuilder<'a, E>
    where E: EntityManager,
{
    pub entity: Entity,
    
    pub entity_manager: &'a mut E, // if a type is a trait, then you need dyn included as it's generic 

    pub component_manager: &'a mut ComponentManager,
}

impl<'a, E> EntityBuilder<'a, E>
    where E: EntityManager
{
    pub fn with<C: Component>(mut self, key: &str, component: C) -> Self {
        // here should be: component_manager.append(self.entity, component)
        self.component_manager.append(self.entity, "test", component); 
        self
    }

    pub fn build(mut self) -> Entity {
        self.entity_manager.add();
        // here is when ecs system add the entity and component
        self.entity
    }
}
