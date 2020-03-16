use crate::{Entity, World};
use legion::borrow::Ref;
use legion::storage::Component;

pub struct EntityRef<'a> {
    pub(crate) world: &'a World,
    pub(crate) entity: Entity,
}

impl<'a> EntityRef<'a> {
    pub fn get<C>(&self) -> Ref<C>
    where
        C: Component,
    {
        self.try_get().unwrap()
    }

    pub fn try_get<C>(&self) -> Option<Ref<C>>
    where
        C: Component,
    {
        self.world.try_get(self.entity)
    }

    pub fn entity(&self) -> Entity {
        self.entity
    }
}
