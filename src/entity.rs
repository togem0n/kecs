#[derive(Copy, Clone, Eq, Debug, Hash, PartialEq, PartialOrd, Default)]
pub struct Entity(pub u32);

impl From<u32> for Entity {
    fn from(u: u32) -> Self {
        Entity(u)
    }
}

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
