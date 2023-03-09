use super::entity::*;

// only use of entitymanager is to add and remove entity from manager
// we later call its function from something like ECS manager within world
pub trait EntityManager {
    fn new() -> Self;

    fn add(&mut self);

    fn remove(&mut self, entity: impl Into<Entity>);

    fn has(self, entity: impl Into<Entity>) -> bool;
}

// for better performance maybe array entities?
// use vec entities for now to save my lifetime
#[derive(Default)]
pub struct VecEntityManager {
    pub entities: Vec<Entity>, 
    pub num_of_entities: u32,
}

impl EntityManager for VecEntityManager {
    fn new() -> Self {
        VecEntityManager {
            entities: vec![],
            num_of_entities: 0,
        }
    }

    fn add(&mut self) {
        let entity = Entity::from(self.num_of_entities);
        self.entities.push(entity);
        self.num_of_entities += 1;
    }

    fn remove(&mut self, entity: impl Into<Entity>) {
        let entity = entity.into();
        self.entities
            .iter()
            .position(|&n| n == entity)
            .map(|e| self.entities.remove(e));
        self.num_of_entities -= 1;
    }

    fn has(self, entity: impl Into<Entity>) -> bool {
        let entity = entity.into();
        self.entities.contains(&entity) 
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        let mut manager = VecEntityManager::new();
        manager.add();
        manager.add();
        assert!(manager.entities.contains(&Entity::from(0)));
        assert!(manager.entities.contains(&Entity::from(1)));
    }

    #[test]
    fn test_remove() {
        let mut manager = VecEntityManager::new();
        manager.add();
        manager.remove(Entity::from(0));
        assert!(!manager.entities.contains(&Entity::from(0)));
    }

    #[test]
    fn test_has() {
        let mut manager = VecEntityManager::new();
        manager.add();
        assert!(manager.has(Entity::from(0)));
    }

}
