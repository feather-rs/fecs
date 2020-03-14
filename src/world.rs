use crate::{Query, QueryBorrow};
use legion::borrow::{Ref, RefMut};
use legion::entity::Entity;
use legion::query::IntoQuery;
use legion::storage::Component;
use legion::world::EntityMutationError;
use legion::world::IntoComponentSource;

type LegionWorld = legion::world::World;

#[derive(Default)]
pub struct World {
    inner: LegionWorld,
}

impl World {
    pub fn new() -> Self {
        World {
            inner: LegionWorld::default(),
        }
    }

    pub fn spawn(&mut self, components: impl IntoComponentSource) -> &[Entity] {
        self.inner.insert((), components)
    }

    pub fn despawn(&mut self, entity: Entity) -> bool {
        self.inner.delete(entity)
    }

    pub fn add(
        &mut self,
        entity: Entity,
        component: impl Component,
    ) -> Result<(), EntityMutationError> {
        self.inner.add_component(entity, component)
    }

    pub fn remove<C>(&mut self, entity: Entity) -> Result<(), EntityMutationError>
    where
        C: Component,
    {
        self.inner.remove_component::<C>(entity)
    }

    pub fn get<C>(&self, entity: Entity) -> Ref<C>
    where
        C: Component,
    {
        self.try_get(entity).unwrap_or_else(|| {
            panic!(
                "failed to immutably borrow component with type {}",
                std::any::type_name::<C>()
            )
        })
    }

    pub fn get_mut<C>(&mut self, entity: Entity) -> RefMut<C>
    where
        C: Component,
    {
        self.try_get_mut(entity).unwrap_or_else(|| {
            panic!(
                "failed to mutably borrow component with type {}",
                std::any::type_name::<C>()
            )
        })
    }

    pub unsafe fn get_mut_unchecked<C>(&self, entity: Entity) -> RefMut<C>
    where
        C: Component,
    {
        self.try_get_mut_unchecked(entity).unwrap_or_else(|| {
            panic!(
                "failed to mutably borrow component with type {}",
                std::any::type_name::<C>()
            )
        })
    }

    pub fn try_get<C>(&self, entity: Entity) -> Option<Ref<C>>
    where
        C: Component,
    {
        self.inner.get_component(entity)
    }

    pub fn try_get_mut<C>(&mut self, entity: Entity) -> Option<RefMut<C>>
    where
        C: Component,
    {
        self.inner.get_component_mut(entity)
    }

    pub unsafe fn try_get_mut_unchecked<C>(&self, entity: Entity) -> Option<RefMut<C>>
    where
        C: Component,
    {
        self.inner.get_component_mut_unchecked(entity)
    }

    pub fn query<Q>(&mut self) -> QueryBorrow<Q>
    where
        Q: Query,
    {
        QueryBorrow {
            world: self,
            inner: Q::Legion::query(),
        }
    }

    pub fn defrag(&mut self, budget: Option<usize>) {
        self.inner.defrag(budget)
    }

    pub fn clear(&mut self) {
        self.inner.delete_all()
    }

    pub fn inner(&self) -> &LegionWorld {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut LegionWorld {
        &mut self.inner
    }
}
