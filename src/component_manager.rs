use std::any::type_name;
// use core::any::Any;
use std::any::{Any, TypeId};
use fxhash::FxHashMap;
use crate::entity::*;
use crate::component::*;

pub type Map = FxHashMap<(Entity, String), Box<dyn Any>>;

pub type TypeName = String;

pub type Components = FxHashMap<(TypeId, TypeName), Box<dyn Any>>;

#[derive(Default)]
pub struct ComponentManager {
    map: Map,
    components: Components,
}

impl ComponentManager {
    // init
    pub fn new() -> Self {
        Self::default()
    }

    // register component in component manager by its typeid and typename
    pub fn register_component<C: Component>(&mut self) -> &mut Self {
        self.components.insert(
            (TypeId::of::<C>(), String::from(type_name::<C>())), 
            Box::new(String::from("test"))
        );
        self
    }

    // add component to a given entity
    pub fn add_component_to_entity<C: Component>(&mut self, entity: Entity, component: C) {
        if self.contain_component::<C>() {
            self.map.insert((entity, String::from(type_name::<C>())), Box::new(component));
        }
    }

    // shouldnt have this funtion here in component manager
    pub fn contains_entity(&self, entity: Entity) -> bool {
        self.map.iter().any(|(k, _)| k.0 == entity)
    }

    // check if a component is registered
    pub fn contain_component<C: Component>(&self) -> bool {
        self.components.iter().any(|(k, _)| k.0 == TypeId::of::<C>())
    }

    pub fn len_of_component(&self) -> usize {
        self.components.len()
    }

    pub fn print_registered_components(&self) {
        println!("Registered {} Components", self.len_of_component());
        for (k, _) in self.components.iter() {
            println!("-----------------------{}", k.1);
        }
        println!("");
    }

    pub fn print_components_of_entity(&self, entity: Entity) {
        println!("Components of {:?}", entity);
        for (k, _) in self.map.iter() {
            if k.0 == entity {
                println!("-----------------------{}", k.1)
            }
        }
        println!("");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_component() {
        let mut component_manager = ComponentManager::new();
        let entity_one = Entity::from(1);
        struct One {}
        struct Two {}
        struct Three {}
        component_manager.register_component::<One>();
        component_manager.register_component::<Two>();
        component_manager.register_component::<Three>();
        component_manager.add_component_to_entity(entity_one, One{});
        component_manager.add_component_to_entity(entity_one, Two{});
        component_manager.print_registered_components();
        component_manager.print_components_of_entity(entity_one);
        assert_eq!(component_manager.contain_component::<One>(), true);
        assert_eq!(component_manager.contain_component::<Two>(), true);
        assert_eq!(component_manager.contain_component::<Three>(), true);
        assert_eq!(component_manager.contains_entity(entity_one), true);
    }

    #[test]
    fn test_contain_entity() {

    }
}
