mod builder;
mod entity_ref;
mod events;
mod query;
mod resources;
mod system;
mod world;

pub use builder::{BuiltEntity, EntityBuilder};
pub use fecs_macros::{event_handler, system};
pub use legion::entity::Entity;
// pub use query::{Query, QueryBorrow, QueryElement};
pub use entity_ref::EntityRef;
pub use events::{Event, EventHandlers, RawEventHandler};
pub use resources::{OwnedResources, Ref, RefMut, RefResources, ResourcesEnum, ResourcesProvider};
pub use system::{Executor, RawSystem};
pub use world::World;

pub use legion::filter::filter_fns::*;
pub use legion::query::{IntoQuery, Read, TryRead, TryWrite, Write};
