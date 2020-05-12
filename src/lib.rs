#![cfg_attr(feature = "no-std", no_std)]

extern crate alloc;

pub mod borrow;

mod component;
mod entity;
mod query;
mod util;
mod world;

pub use borrow::AtomicRefCell;
pub use component::Component;
pub use entity::Entity;
pub use world::{local::World, shared::SharedWorld};
