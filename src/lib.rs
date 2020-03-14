mod builder;
mod erased_vec;
mod query;
mod system;
mod world;

pub use builder::{BuiltEntity, EntityBuilder};
pub(crate) use erased_vec::ErasedVec;
pub use fecs_macros::system;
pub use system::{EventHandler, Executor, RawSystem, SystemCtx};
pub use world::World;

pub use resources::Resources;
