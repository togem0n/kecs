use std::any::type_name;
// use core::any::Any;
use std::any::{Any, TypeId};
use fxhash::FxHashMap;
use std::collections::HashSet;
use crate::entity::*;
use crate::component::*;

pub type Map = FxHashMap<(Entity, String), Box<dyn Any>>;

pub type TypeName = String;

pub type Components = FxHashMap<TypeId, TypeName>;

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
            TypeId::of::<C>(), String::from(type_name::<C>()) 
        );
        self
    }

    // add component to a given entity
    pub fn add_component_to_entity<C: Component>(&mut self, entity: Entity, component: C) -> &mut Self {
        if self.contain_component::<C>() && !self.has_component::<C>(entity) {
            self.map.insert((entity, String::from(type_name::<C>())), Box::new(component));
        }
        self
    }

    // remove component from a given entity
    pub fn remove_component_from_entity<C: Component>(&mut self, entity: Entity) -> &mut Self {
       if self.contains_entity(entity) {
            let compoennts_to_remove: Vec<(Entity, String)> = self
                .map
                .iter()
                .filter(|(k, _)| k.1 == String::from(type_name::<C>()))
                .map(|(k, _)| k.clone())
                .collect();
            
            for k in compoennts_to_remove {
                self.map.remove(&k);
            }
        } 
        self
    }

    pub fn has_component<C: Component>(&mut self, entity: Entity) -> bool { 
        self
            .map
            .iter()
            .any(|(k, _)| k.0 == entity && k.1 == String::from(type_name::<C>()))
    }

    // remove all components associated with the entity (also remove entity itself.)
    pub fn remove_entity(&mut self, entity: impl Into<Entity>) -> &mut Self {
        let entity = entity.into();

        let target_keys: Vec<(Entity, String)> = self
            .map
            .iter()
            .filter(|(k, _)| k.0 == entity)
            .map(|(k, _)| k.clone())
            .collect();

        for k in target_keys {
            self.map.remove(&k);
        }

        self
    }

    // shouldnt have this funtion here in component manager
    pub fn contains_entity(&self, entity: Entity) -> bool {
        self.map.iter().any(|(k, _)| k.0 == entity)
    }

    // check if a component is registered
    pub fn contain_component<C: Component>(&self) -> bool {
        self.components.iter().any(|(k, _)| *k == TypeId::of::<C>())
    }

    pub fn len_of_component(&self) -> usize {
        self.components.len()
    }

    // TODO:: remake logging using debug display trait
    pub fn print_general_infos(&mut self) -> &mut Self{
        self.print_num_entities_with_components();
        self.print_registered_components();
        self
    }

    pub fn print_registered_components(&mut self) -> &mut Self {
        println!("Registered {} Components-----------------------------------", self.len_of_component());
        for (_, v) in self.components.iter() {
            println!("                                                          {}", v);
        }
        println!("");
        self
    }

    pub fn print_num_entities_with_components(&mut self) -> &mut Self{
        let mut s = HashSet::new();
        for (k, _) in self.map.iter() {
            s.insert(k.0);
        }
        println!("Num of entities with components---------------------------");
        println!("                                                          {}", s.len());
        self
    }

    pub fn print_components_of_entity(&mut self, entity: Entity) -> &mut Self {
        println!("Components of {:?}", entity);
        for (k, _) in self.map.iter() {
            if k.0 == entity {
                println!("----------------------- {}", k.1)
            }
        }
        println!("");
        self
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
        component_manager
            .register_component::<One>()
            .register_component::<Two>()
            .register_component::<Three>();
        // component_manager.register_component::<(One, &Two, &Three)>();
        component_manager.register_component::<i32>();
        component_manager
            .add_component_to_entity(entity_one, One{})
            .add_component_to_entity(entity_one, Two{});
        component_manager
            .print_registered_components()
            .print_components_of_entity(entity_one);
        assert_eq!(component_manager.contain_component::<One>(), true);
        assert_eq!(component_manager.contain_component::<Two>(), true);
        assert_eq!(component_manager.contain_component::<Three>(), true);
        assert_eq!(component_manager.contains_entity(entity_one), true);
    }

    #[test]
    fn test_contain_entity() {
        let mut component_manager = ComponentManager::new();
        let entity_one = Entity::from(1);
        struct One {}
        struct Two {}
        struct Three {}
        component_manager
            .register_component::<One>()
            .register_component::<Two>()
            .register_component::<Three>();
        component_manager
            .add_component_to_entity(entity_one, One{})
            .add_component_to_entity(entity_one, Two{});
        component_manager.print_general_infos();
        assert_eq!(component_manager.contains_entity(entity_one), true);
        component_manager.remove_entity(entity_one);
        assert_eq!(component_manager.contains_entity(entity_one), false);
    }
    
    #[test]
    fn test_remove_component() {
        let mut component_manager = ComponentManager::new();
        let entity_one = Entity::from(1);
        struct One {}
        struct Two {}
        struct Three {}
        component_manager
            .register_component::<One>()
            .register_component::<Two>()
            .register_component::<Three>();
        component_manager
            .add_component_to_entity(entity_one, One{});
        assert_eq!(component_manager.has_component::<One>(entity_one), true);
        component_manager.remove_component_from_entity::<One>(entity_one); 
        assert_eq!(component_manager.has_component::<One>(entity_one), false);
    }
}
