mod builder;
mod entity_ref;
mod events;
mod query;
mod registry;
mod resources;
mod system;
mod world;

pub use builder::{BuiltEntity, EntityBuilder};
pub use fecs_macros::{event_handler, system};
pub use legion::entity::Entity;
// pub use query::{Query, QueryBorrow, QueryElement};
pub use entity_ref::EntityRef;
pub use events::{Event, EventHandlers, RawEventHandler};
pub use registry::{build_executor, SystemRegistration};
pub use resources::{Ref, RefMut, Resources};
pub use system::{Executor, RawSystem};
pub use world::World;

pub use legion::filter::filter_fns::*;
pub use legion::query::{IntoQuery, Read, TryRead, TryWrite, Write};

pub extern crate inventory;

#[macro_export]
macro_rules! query {
    ($q:expr) => {{
        $crate::panic();
        $q
    }};
}

#[doc(hidden)]
pub fn panic() {
    unreachable!()
}
