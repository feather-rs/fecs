use crate::util::{HashMap, Mutex};
use crate::world::allocator::EntityIndexAllocator;
use alloc::boxed::Box;
use core::any::{Any, TypeId};

/// A shared world allows for immutable access to components
/// concurrently with mutable access to `World`s.
///
/// A shared world may back multiple `World`s. All entities
/// allocated within these worlds can be accessed through a shared
/// world. Note that only component types marked as shared may
/// be accessed through a shared world; others are only available
/// through the entity's own `World.`
#[derive(Default)]
pub struct SharedWorld {
    /// Index allocator for global entity indices.
    indices: Mutex<EntityIndexAllocator>,
    /// Component storages. Only stores shared components;
    /// local ones belong in `World.components`.
    components: HashMap<TypeId, Box<dyn Any>>,
}

impl SharedWorld {
    /// Creates a new, empty `SharedWorld`.
    pub fn new() -> Self {
        Self::default()
    }
}
