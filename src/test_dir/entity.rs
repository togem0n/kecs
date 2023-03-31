// This can be used to easily change the size of an EntityId.
pub(crate) type EntityId = u32;

/// An entity's location within archetype list
#[derive(Debug, Clone, Copy)]
#[doc(hidden)]
pub struct EntityLocation {
    pub archetype_index: EntityId,
    pub index_in_archetype: EntityId,
}

/// An entity's info inside entity list
#[derive(Clone, Copy)]
pub(crate) struct EntityInfo {
    pub(crate) generation: EntityId,
    pub(crate) location: EntityLocation,
}

/// A handle to an entity within the world.
#[derive(Debug, Clone, Copy, Hash, Eq, Ord, PartialEq, PartialOrd)]
pub struct Entity {
    pub(crate) index: EntityId,
    pub(crate) generation: EntityId,
}

/// This entity has been despawned so operations can no longer
/// be performed on it.
#[derive(Debug)]
pub struct NoSuchEntity;

impl std::fmt::Display for NoSuchEntity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "The entity no longer exists so the operation cannot be performed"
        )
    }
}

impl std::error::Error for NoSuchEntity {}

#[derive(Debug)]
pub struct EntityMissingComponent(EntityId, &'static str);

impl EntityMissingComponent {
    pub fn new<T>(entity_id: EntityId) -> Self {
        Self(entity_id, std::any::type_name::<T>())
    }
}

impl std::fmt::Display for EntityMissingComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Entity {:?} does not have a [{}] component",
            self.0, self.1
        )
    }
}

impl std::error::Error for EntityMissingComponent {}

#[derive(Debug)]
pub enum ComponentError {
    EntityMissingComponent(EntityMissingComponent),
    NoSuchEntity(NoSuchEntity),
}

