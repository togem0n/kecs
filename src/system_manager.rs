use std::any::type_name;
use std::any::{Any, TypeId};
use crate::component::*;
use crate::component_manager::*;
use fxhash::FxHashMap;

// so when we make a system we need to pass in a query parameter 
// just to iterate over all the required components
// The query itself is just a bunch of components

struct Query<C>(Vec<Box<C>>);

impl<C: Component> Query<C> {
    fn new(components: impl IntoIterator<Item = C>) -> Self {
        let boxed_components: Vec<Box<C>> = components
            .into_iter()
            .map(|c| Box::new(c) as Box<C>)
            .collect();
        Query(boxed_components)
    }

    fn add(&mut self, components: impl IntoIterator<Item = C>) {
        let boxed_components: Vec<Box<C>> = components
            .into_iter()
            .map(|c| Box::new(c) as Box<C>)
            .collect();
        self.0.extend(boxed_components);
    }
}

struct Health {}
struct Position {}

fn testSystem(query: Query<(Health, Position)>) {
    println!("nmsl");
}

// fn system(query: Query<(&Health, &Position)>){
//
// }
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system() {
        let query_h_t_p = Query::new(vec![
            (Health{}, Position{}),
            (Health{}, Position{}),
        ]);

        testSystem(query_h_t_p);
    }
}
