//! Re-exports types depending on crate enabled features.

#[cfg(feature = "no-std")]
type BaseHashMap<K, V, S> = hashbrown::HashMap<K, V, S>;
#[cfg(not(feature = "no-std"))]
type BaseHashMap<K, V, S = std::collections::hash_map::DefaultHasher> =
    std::collections::HashMap<K, V, S>;

#[cfg(feature = "fast-hasher")]
pub type HashMap<K, V> = BaseHashMap<K, V, rustc_hash::FxHasher>;
#[cfg(not(feature = "fast-hasher"))]
pub type HashMap<K, V> = BaseHashMap<K, V>;

#[cfg(feature = "no-std")]
pub type Mutex<T> = spinning_top::Spinlock<T>;
#[cfg(all(not(feature = "no-std"), not(feature = "fast-locks")))]
pub type Mutex<T> = std::sync::Mutex<T>;
#[cfg(all(not(feature = "no-std"), feature = "fast-locks"))]
pub type Mutex<T> = parking_lot::Mutex<T>;

#[cfg(not(loom))]
pub use core::{
    cell::UnsafeCell,
    sync::atomic::{AtomicBool, AtomicPtr, AtomicU32, AtomicUsize},
};

/*
#[cfg(loom)]
pub use loom::{
    cell::UnsafeCell,
    sync::atomic::{AtomicBool, AtomicPtr, AtomicU32, AtomicUsize},
};
*/
