use fxhash::{FxBuildHasher, FxHashMap};
use std::any::{Any, TypeId};
use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Default)]
struct BorrowFlag(
    /// If set to `u32::max_value()`, the resource
    /// is borrowed mutably; otherwise, it is set to the number of immutable
    /// borrows currently existing.
    AtomicU32,
);

impl BorrowFlag {
    /// Attempts to flag this value as mutably borrowed, returning
    /// `true` if successful and `false` otherwise.
    fn obtain_mutable(&self) -> bool {
        self.0
            .compare_and_swap(0, u32::max_value(), Ordering::AcqRel)
            == 0
    }

    /// Marks this resource as not mutably borrowed.
    fn release_mutable(&self) {
        debug_assert_eq!(self.0.load(Ordering::Acquire), u32::max_value());
        self.0.store(0, Ordering::Release);
    }

    /// Attempts to obtain an immutable borrow, returning `true` if successful
    /// and `false` otherwise.
    fn obtain_immutable(&self) -> bool {
        loop {
            let val = self.0.load(Ordering::Acquire);

            if val == u32::max_value() {
                return false;
            }

            if self.0.compare_and_swap(val, val + 1, Ordering::AcqRel) == val {
                return true;
            }
        }
    }

    /// Releases an immutable borrow.
    fn release_immutable(&self) {
        self.0.fetch_sub(1, Ordering::AcqRel);
    }
}

pub struct Ref<'a, T> {
    flag: &'a BorrowFlag,
    value: &'a T,
}

impl<'a, T> Deref for Ref<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<'a, T> Drop for Ref<'a, T> {
    fn drop(&mut self) {
        self.flag.release_immutable();
    }
}

pub struct RefMut<'a, T> {
    flag: &'a BorrowFlag,
    value: &'a mut T,
}

impl<'a, T> Deref for RefMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<'a, T> DerefMut for RefMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value
    }
}

impl<'a, T> Drop for RefMut<'a, T> {
    fn drop(&mut self) {
        self.flag.release_mutable();
    }
}

type RefEntry = (BorrowFlag, UnsafeCell<*mut dyn Any>);

/// Stores a set of values, each with a distinct type.
///
/// Resources are borrow checked at runtime.
///
/// In contrast to many resources crates, this type allows
/// temporary binding of non-owned resources through the `with_ref` method.
/// The lifetime parameter indicates the lifetime of these references;
/// if you are only adding owned resources through `insert`, this
/// can be `'static`.
pub struct Resources<'a> {
    /// Mapping from resource types to their structs.
    types: FxHashMap<TypeId, (BorrowFlag, UnsafeCell<Box<dyn Any>>)>,
    /// Additional shared values. (Linear search map)
    refs: Vec<(TypeId, RefEntry)>,
    _lifetime: PhantomData<&'a dyn Any>,
}

impl Default for Resources<'static> {
    fn default() -> Self {
        Self::new()
    }
}

impl Resources<'static> {
    /// Creates a new `Resources` with no stored values.
    pub fn new() -> Self {
        Self {
            types: FxHashMap::with_hasher(FxBuildHasher::default()),
            refs: Vec::new(),
            _lifetime: Default::default(),
        }
    }
}

impl<'a> Resources<'a> {
    /// Inserts a new resource into this `Resources`.
    ///
    /// Replaces an existing value of the same type.
    pub fn insert<T>(&mut self, resource: T)
    where
        T: 'static,
    {
        self.types.insert(
            TypeId::of::<T>(),
            (BorrowFlag::default(), UnsafeCell::new(Box::new(resource))),
        );
    }

    /// Method chaining alias for `insert`.
    pub fn with<T>(mut self, resource: T) -> Self
    where
        T: 'static,
    {
        self.insert(resource);
        self
    }

    /// Inserts a reference to a resource into this `Resources`.
    pub fn with_ref<'b, T>(mut self, resource: &'a mut T) -> Resources<'b>
    where
        T: 'static,
        'a: 'b,
    {
        self.refs.push((
            TypeId::of::<T>(),
            (BorrowFlag::default(), UnsafeCell::new(resource as *mut _)),
        ));

        Resources {
            types: self.types,
            refs: self.refs,
            _lifetime: self._lifetime,
        }
    }

    /// Removes all references from this `Resources`, returning
    /// a `Resources<'static>`.
    pub fn without_refs(self) -> Resources<'static> {
        let refs: Vec<(TypeId, RefEntry)> = Vec::new();
        Resources {
            types: self.types,
            refs,
            _lifetime: Default::default(),
        }
    }

    /// Immutably borrows a resource from this container.
    ///
    /// # Panics
    /// Panics if the resource does not exist or if it
    /// is already mutably borrowed.
    pub fn get<T>(&self) -> Ref<T>
    where
        T: 'static,
    {
        self.try_get().unwrap()
    }

    /// Immutably borrows a resource from this container.
    ///
    /// Returns `None` if the resource does not exist
    /// or if it is already mutably borrowed.
    pub fn try_get<T>(&self) -> Option<Ref<T>>
    where
        T: 'static,
    {
        self.types
            .get(&TypeId::of::<T>())
            .map(|(flag, resource)| {
                if flag.obtain_immutable() {
                    Some(Ref {
                        flag,
                        value: (unsafe { &*resource.get() }).downcast_ref().unwrap(),
                    })
                } else {
                    None
                }
            })
            .flatten()
            .or_else(|| {
                self.refs
                    .iter()
                    .find(|(id, _)| *id == TypeId::of::<T>())
                    .map(|(_, (flag, resource))| {
                        if flag.obtain_immutable() {
                            Some(Ref {
                                flag,
                                value: unsafe { &*(*resource.get()) }.downcast_ref().unwrap(),
                            })
                        } else {
                            None
                        }
                    })
                    .flatten()
            })
    }

    /// Mutably borrows a resource from this container.
    ///
    /// # Panics
    /// Panics of the resource does not exist or it is already borrowed.
    pub fn get_mut<T>(&self) -> RefMut<T>
    where
        T: 'static,
    {
        self.try_get_mut().unwrap()
    }

    /// Mutably borrows a resource from this container.
    ///
    /// Returns `None` if the resource does not exist
    /// or it is already borrowed.
    pub fn try_get_mut<T>(&self) -> Option<RefMut<T>>
    where
        T: 'static,
    {
        self.types
            .get(&TypeId::of::<T>())
            .map(|(flag, resource)| {
                if flag.obtain_mutable() {
                    Some(RefMut {
                        flag,
                        value: (unsafe { &mut *resource.get() }).downcast_mut().unwrap(),
                    })
                } else {
                    None
                }
            })
            .flatten()
            .or_else(|| {
                self.refs
                    .iter()
                    .find(|(id, _)| *id == TypeId::of::<T>())
                    .map(|(_, (flag, resource))| {
                        if flag.obtain_mutable() {
                            Some(RefMut {
                                flag,
                                value: unsafe { &mut *(*resource.get()) }.downcast_mut().unwrap(),
                            })
                        } else {
                            None
                        }
                    })
                    .flatten()
            })
    }
}
