/// A type which can be used as a component.
///
/// Components can be any `Send + Sync + 'static` type.
/// To easily implement `Component` for your type,
/// we provide the `Component` derive macro.
pub trait Component: Send + Sync + 'static {
    /// Whether this component is a _shared_ component,
    /// i.e. one shared between all shards of a world.
    const SHARED: bool;
}
