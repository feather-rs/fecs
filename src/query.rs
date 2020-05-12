/// Trait implemented for tuples of components which
/// may be queried from a `World`.
///
/// Implemented for tuples of up to 8 components. If you
/// desire more, you are also able to nest `QueryTuple`s.
pub trait QueryTuple {}
