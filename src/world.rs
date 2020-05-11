use crate::entity_ref::EntityRef;
use crate::query::{Query, QueryBorrow};
use legion::borrow::{Ref, RefMut};
use legion::entity::Entity;
use legion::query::IntoQuery;
use legion::storage::Component;
use legion::world::{EntityMutationError, IntoComponentSource, ComponentTypeTupleSet};

type LegionWorld = legion::world::World;

/// Contains queryable collections of data associated with `Entity`s.
#[derive(Default)]
pub struct World {
    inner: LegionWorld,
}

impl World {
    /// Creates a new Fecs World
    pub fn new() -> Self {
        World {
            inner: LegionWorld::default(),
        }
    }

    /// Spawns multiple new entities into the world with the given components,
    /// the `EntityBuilder` and `BuiltEntity::spawn_in` is prefered for spawning
    /// a single entity. You can use the `EntityBuilder::build` to create multiple
    /// entities, this method can then be used to batch insert them.
    ///
    /// Returns a slice of entity handlers for the spawned entities.
    pub fn spawn(&mut self, components: impl IntoComponentSource) -> &[Entity] {
        self.inner.insert((), components)
    }

    /// Despawns the given `Entity` from the `World`.
    ///
    /// Returns `true` if the entity was despawned; else `false`.
    pub fn despawn(&mut self, entity: Entity) -> bool {
        self.inner.delete(entity)
    }

    /// Adds a component to an entity, or sets its value if the component is already present.
    ///
    /// # Notes
    /// This function has the overhead of moving the entity to either an existing or new archetype,
    /// causing a memory copy of the entity to a new location. This function should not be used
    /// multiple times in successive order.
    pub fn add(
        &mut self,
        entity: Entity,
        component: impl Component,
    ) -> Result<(), EntityMutationError> {
        self.inner.add_component(entity, component)
    }


    /// Removes a component from an entity.
    ///
    /// # Notes
    /// This function has the overhead of moving the entity to either an existing or new archetype,
    /// causing a memory copy of the entity to a new location. This function should not be used
    /// multiple times in successive order.
    ///
    /// `World::batch_remove` should be used for removing multiple components from an entity at once., 
    pub fn remove<C>(&mut self, entity: Entity) -> Result<(), EntityMutationError>
    where
        C: Component,
    {
        self.inner.remove_component::<C>(entity)
    }

    /// Removes multiple components from an entity
    ///
    /// # Notes
    /// This function is provided for bulk deleting components from an entity. This difference between this
    /// function and `remove_component` is this allows us to remove multiple components and still only
    /// perform a single move operation of the entity.
    pub fn batch_remove<C>(&mut self, entity: Entity) -> Result<(), EntityMutationError>
    where
        C: ComponentTypeTupleSet,
    {
        self.inner.remove_components::<C>(entity)
    }

    /// Borrows component data `C` for the given entity.
    ///
    /// Panics if the entity was not found or did not contain the specified component.
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

    /// Mutably borrows component data `C` for the given entity.
    ///
    /// Panics if the neity was not found or did not contain the specified component.
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

    /// # Safety
    /// The caller must ensure that there exists at most one
    /// mutable reference to a given component at any time.
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

    /// Borrows component data `C` for the given entity.
    ///
    /// Returns `Some(data)` if the entity was found and contains the specified data.
    /// Otherwise `None` is returned.
    pub fn try_get<C>(&self, entity: Entity) -> Option<Ref<C>>
    where
        C: Component,
    {
        self.inner.get_component(entity)
    }

    /// Mutably borrows component data `C` for the given entity.
    ///
    /// Returns `Some(data)` if the entity was found and contains the specified data.
    /// Otherwise `None` is returned.
    pub fn try_get_mut<C>(&mut self, entity: Entity) -> Option<RefMut<C>>
    where
        C: Component,
    {
        self.inner.get_component_mut(entity)
    }

    /// # Safety
    /// The caller must ensure that there exists at most one
    /// mutable reference to a given component at any time.
    pub unsafe fn try_get_mut_unchecked<C>(&self, entity: Entity) -> Option<RefMut<C>>
    where
        C: Component,
    {
        self.inner.get_component_mut_unchecked(entity)
    }

    /// Checks if the given entity contains the component `C`.
    pub fn has<C>(&self, entity: Entity) -> bool
    where
        C: Component,
    {
        self.try_get::<C>(entity).is_some()
    }

    /// Creates a refrence for the world and the given entity.
    ///
    /// Returns `Some(refrence)` if the entity is alive otherwise.
    /// Otherwise `None` is returned.
    pub fn entity(&self, entity: Entity) -> Option<EntityRef> {
        if self.is_alive(entity) {
            Some(EntityRef {
                world: self,
                entity,
            })
        } else {
            None
        }
    }

    /// Creates a query for the world. 
    pub fn query<Q>(&mut self) -> QueryBorrow<Q>
    where
        Q: Query,
    {
        QueryBorrow {
            world: self,
            inner: Q::Legion::query(),
        }
    }

    /// Determines if the given `Entity` is alive within this `World`.
    pub fn is_alive(&self, entity: Entity) -> bool {
        self.inner.is_alive(entity)
    }

    /// Iteratively defragments the world's internal memory.
    ///
    /// This compacts entities into fewer more continuous chunks.
    ///
    /// `budget` describes the maximum number of entities that can be moved
    /// in one call. Subsequent calls to `defrag` will resume progress from the
    /// previous call.
    pub fn defrag(&mut self, budget: Option<usize>) {
        self.inner.defrag(budget)
    }

    /// Delete all entities and their associated data. 
    /// This leaves subscriptions and the command buffer intact.
    pub fn clear(&mut self) {
        self.inner.delete_all()
    }

    /// Borrows `Legion::World` which `Fecs::World` is based on.
    pub fn inner(&self) -> &LegionWorld {
        &self.inner
    }

    /// Mutable borrows `Legion::World` which `Fecs::World` is based on.
    pub fn inner_mut(&mut self) -> &mut LegionWorld {
        &mut self.inner
    }
}
