use core::any::Any;
use fxhash::FxHashMap;
use crate::entity::*;
use crate::component::*;

pub type Components = FxHashMap<(Entity, String), Box<dyn Any>>;

#[derive(Default)]
pub struct ComponentManager {
    components: Components,
}

impl ComponentManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn append<C: Component>(&mut self, entity: Entity, key: &str, component: C) {
        // from here to add element to component hashmap
        self.components.insert((entity, key.into()), Box::new(component));
    }

    pub fn contains_entity(&self, entity: Entity) -> bool {
        self.components.iter().any(|(k, _)| k.0 == entity)
    }

    /// Returns the number of components in the store.
    pub fn len(&self) -> usize {
        self.components.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_component() {
        let mut component_manager = ComponentManager::new();
        let entity_one = Entity::from(1);
        let entity_two = Entity::from(2);
        let component_one = String::from("componen_one");
        let component_two = String::from("componen_two");
        component_manager.append(entity_one, "entity_one", component_one);
        component_manager.append(entity_two, "entity_two", component_two);
        assert_eq!(component_manager.contains_entity(entity_one), true);
        assert_eq!(component_manager.contains_entity(entity_two), true);
    }

    #[test]
    fn test_len() {
        let mut component_manager = ComponentManager::new();
        let entity_one = Entity::from(1);
        let entity_two = Entity::from(2);
        component_manager.append(entity_one, "component_one", String::from("one"));
        component_manager.append(entity_one, "component_two", String::from("two"));
        component_manager.append(entity_two, "compoennt_one", String::from("three"));
        component_manager.append(entity_two, "component_two", String::from("four"));
        println!("{}", component_manager.len());
        assert_eq!(component_manager.len(), 4);
    }
}
