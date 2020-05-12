use crate::util::HashMap;
use crate::world::allocator::EntityIndexAllocator;
use crate::world::shared::SharedWorld;
use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::any::{Any, TypeId};

/// A _shard_ of a world. Each world shard owns its own entities
/// and their components, but it also broadcasts any shared components
/// to the backing `SharedWorld`.
///
/// A shard allows mutable access to all components and allows for the creation
/// and deletion of entities. These created entities are only accessible inside this
/// shard, but a `SharedEntity` handle can be created to access shared components
/// through the `SharedWorld`.
pub struct World {
    /// Index allocator for local entity indices.
    indices: EntityIndexAllocator,
    /// Vector of entity versions indexed by an entity's
    /// local index.
    versions: Vec<u32>,
    /// Component storages.
    components: HashMap<TypeId, Box<dyn Any>>,
    /// The `SharedWorld` backing this `World`.
    shared: Arc<SharedWorld>,
}

impl Default for World {
    fn default() -> Self {
        let shared = Arc::new(SharedWorld::new());

        Self {
            indices: EntityIndexAllocator::new(),
            versions: Vec::new(),
            components: HashMap::default(),
            shared,
        }
    }
}

impl World {
    /// Creates a new, empty `World`.
    ///
    /// If you want to use the sharded-world functionality
    /// provided by this create, we recommend you use `SharedWorld::create_world()`
    /// instead.
    pub fn new() -> Self {
        Self::default()
    }
}

/// A component storage based on a sparse set.
///
/// Stores all components of one type within a `World`.
/// Note that shared components are stored in `shared::ComponentStore`
/// instead.
struct ComponentStore<T> {
    sparse: Vec<u32>,
    dense: Vec<u32>,
    data: Vec<T>,
}
