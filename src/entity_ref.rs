use crate::{Entity, World};
use legion::borrow::Ref;
use legion::storage::Component;

/// A refrence to a `World` and `Entity` to allow for easy retrival of components.
pub struct EntityRef<'a> {
    pub(crate) world: &'a World,
    pub(crate) entity: Entity,
}

impl<'a> EntityRef<'a> {
    /// Borrows component data `C` from the referenced world and entity.
    ///
    /// Panics if the entity was not found or did not contain the specified component.
    pub fn get<C>(&self) -> Ref<C>
    where
        C: Component,
    {
        self.try_get().unwrap()
    }

    /// Borrows component data `C` from the referenced world and entity.
    ///
    /// Returns `Some(data)` if the entity was found and contains the specified data.
    /// Otherwise `None` is returned.
    pub fn try_get<C>(&self) -> Option<Ref<C>>
    where
        C: Component,
    {
        self.world.try_get(self.entity)
    }

    /// Returns the referenced entity.
    pub fn entity(&self) -> Entity {
        self.entity
    }

    /// Returns the referenced world.
    pub fn world(&self) -> &'a World {
        self.world
    }
}
