#[derive(Copy, Clone, Eq, PartialEq, PartialOrd, Default)]
pub struct Entity(pub u32);

impl From<u32> for Entity {
    fn from(u: u32) -> Self {
        Entity(u)
    }
}

// pub trait EntityManager {
//     fn new() -> Self;
//
//     fn add(&mut self, entity: impl Into<Entity>);
//
//     fn remove(&mut self, entity: impl Into<Entity>);
// }
//
// // for better performance maybe array entities?
// // use vec entities for now to save my lifetime
// #[derive(Default)]
// pub struct VecEntityManager {
//     pub entities: Vec<Entity>, 
// }
//
// impl EntityManager for VecEntityManager {
//     fn new() -> Self {
//         VecEntityManager {
//             entities: vec![]
//         }
//     }
//
//     fn add(&mut self, entity: impl Into<Entity>) {
//         self.entities.push(entity.into());
//     }
//
//     fn remove(&mut self, entity: impl Into<Entity>) {
//         let entity = entity.into();
//         self.entities
//             .iter()
//             .position(|&n| n == entity)
//             .map(|e| self.entities.remove(e));
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_from() {
        let entity = Entity::from(2);
        assert_eq!(entity.0, 2);

        let entity = Entity::from(5);
        assert_eq!(entity.0, 5);
    }
}
