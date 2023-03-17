use super::entity::*;

#[derive(Default)]
pub struct EntityManager {
    pub entities: Vec<Entity>, 
    pub num_of_entities: u32,
}

impl EntityManager {
    pub fn add(&mut self) {
        let entity = Entity::from(self.num_of_entities);
        self.entities.push(entity);
        self.num_of_entities += 1;
    }

    pub fn remove(&mut self, entity: impl Into<Entity>) {
        let entity = entity.into();
        self.entities
            .iter()
            .position(|&n| n == entity)
            .map(|e| self.entities.remove(e));
        self.num_of_entities -= 1;
    }

    pub fn has(self, entity: impl Into<Entity>) -> bool {
        let entity = entity.into();
        self.entities.contains(&entity) 
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        let mut manager = EntityManager::default();
        manager.add();
        manager.add();
        assert!(manager.entities.contains(&Entity::from(0)));
        assert!(manager.entities.contains(&Entity::from(1)));
    }

    #[test]
    fn test_remove() {
        let mut manager = EntityManager::default();
        manager.add();
        manager.remove(Entity::from(0));
        assert!(!manager.entities.contains(&Entity::from(0)));
    }

    #[test]
    fn test_has() {
        let mut manager = EntityManager::default();
        manager.add();
        assert!(manager.has(Entity::from(0)));
    }

}
