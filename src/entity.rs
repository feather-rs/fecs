/// A handle to an entity in a `World`. This
/// can be used to retrieve components, either
/// through `World` or `SharedWorld`.
///
/// # Representation
/// `Entity` is guaranteed to have the same representation
/// as the following C struct:
/// ```c
/// struct Entity {
///     uint32_t a;
///     uint32_t b;
///     uint32_t c;
/// }
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(C)]
pub struct Entity {
    // Based on the standard generational
    // indices design. However, we store
    // two indices—one which is unique across
    // all worlds belonging to the same `SharedWorld`,
    // and one which is only unique within its `World`.
    /// The index into the `World` component storages.
    pub(crate) local_index: u32,
    /// The index into the `SharedWorld` component storages.
    pub(crate) global_index: u32,
    /// The entity's version, used to avoid the ABA problem.
    pub(crate) version: u32,
}
