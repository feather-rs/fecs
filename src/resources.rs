use arrayvec::ArrayVec;
use fxhash::{FxBuildHasher, FxHashMap};
use std::any::{Any, TypeId};
use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Debug, thiserror::Error)]
pub enum ResourceError {
    #[error("resource {0} not found")]
    NotFound(&'static str),
    #[error("resource already borrowed")]
    AlreadyBorrowed,
}

type Result<T> = std::result::Result<T, ResourceError>;

pub trait Resource: Send + Sync + Any + 'static {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T> Resource for T
where
    T: Send + Sync + Any + 'static,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Default, Debug)]
#[doc(hidden)]
pub struct BorrowFlag(
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

pub trait ResourcesProvider {
    /// Immutably borrows a resource from this container.
    ///
    /// # Panics
    /// Panics if the resource does not exist or if it
    /// is already mutably borrowed.
    fn get<T>(&self) -> Ref<T>
    where
        T: Resource;

    /// Immutably borrows a resource from this container.
    ///
    /// Returns `None` if the resource does not exist
    /// or if it is already mutably borrowed.
    fn try_get<T>(&self) -> Result<Ref<T>>
    where
        T: Resource;

    /// Mutably borrows a resource from this container.
    ///
    /// # Panics
    /// Panics of the resource does not exist or it is already borrowed.
    fn get_mut<T>(&self) -> RefMut<T>
    where
        T: Resource;

    /// Mutably borrows a resource from this container.
    ///
    /// Returns `None` if the resource does not exist
    /// or it is already borrowed.
    fn try_get_mut<T>(&self) -> Result<RefMut<T>>
    where
        T: Resource;

    /// Converts this `ResourcesProvider` into a `ResourcesRef`
    /// suitable for passing to dynamically-dispatched functions.
    fn as_resources_ref(&self) -> ResourcesEnum;
}

pub enum ResourcesEnum<'a> {
    Owned(&'a OwnedResources),
    Ref(&'a RefResources<'a, OwnedResources>),
    DoubleRef(&'a ResourcesEnum<'a>),
}

impl<'a> ResourcesProvider for ResourcesEnum<'a> {
    fn get<T>(&self) -> Ref<T>
    where
        T: Resource,
    {
        match self {
            ResourcesEnum::Owned(res) => res.get(),
            ResourcesEnum::Ref(res) => res.get(),
            ResourcesEnum::DoubleRef(res) => res.get(),
        }
    }

    fn try_get<T>(&self) -> Result<Ref<T>>
    where
        T: Resource,
    {
        match self {
            ResourcesEnum::Owned(res) => res.try_get(),
            ResourcesEnum::Ref(res) => res.try_get(),
            ResourcesEnum::DoubleRef(res) => res.try_get(),
        }
    }

    fn get_mut<T>(&self) -> RefMut<T>
    where
        T: Resource,
    {
        match self {
            ResourcesEnum::Owned(res) => res.get_mut(),
            ResourcesEnum::Ref(res) => res.get_mut(),
            ResourcesEnum::DoubleRef(res) => res.get_mut(),
        }
    }

    fn try_get_mut<T>(&self) -> Result<RefMut<T>>
    where
        T: Resource,
    {
        match self {
            ResourcesEnum::Owned(res) => res.try_get_mut(),
            ResourcesEnum::Ref(res) => res.try_get_mut(),
            ResourcesEnum::DoubleRef(res) => res.try_get_mut(),
        }
    }

    fn as_resources_ref(&self) -> ResourcesEnum {
        ResourcesEnum::DoubleRef(self)
    }
}

/// Stores a set of owned values, each with a distinct type.
///
/// Resources are borrow checked at runtime.
pub struct OwnedResources {
    /// Mapping from resource types to their structs.
    types: FxHashMap<TypeId, (BorrowFlag, UnsafeCell<Box<dyn Resource>>)>,
}

// Safety: we ensure correct resource borrows through the atomic `BorrowFlag`.
unsafe impl Send for OwnedResources {}
unsafe impl Sync for OwnedResources {}

impl Default for OwnedResources {
    fn default() -> Self {
        Self::new()
    }
}

impl OwnedResources {
    /// Creates a new `Resources` with no stored values.
    pub fn new() -> Self {
        Self {
            types: FxHashMap::with_hasher(FxBuildHasher::default()),
        }
    }

    /// Inserts a new resource into this `Resources`.
    ///
    /// Replaces an existing value of the same type.
    pub fn insert<T>(&mut self, resource: T)
    where
        T: Resource,
    {
        self.types.insert(
            TypeId::of::<T>(),
            (BorrowFlag::default(), UnsafeCell::new(Box::new(resource))),
        );
    }

    /// Method chaining alias for `insert`.
    pub fn with<T>(mut self, resource: T) -> Self
    where
        T: Resource,
    {
        self.insert(resource);
        self
    }
}

impl ResourcesProvider for OwnedResources {
    /// Immutably borrows a resource from this container.
    ///
    /// # Panics
    /// Panics if the resource does not exist or if it
    /// is already mutably borrowed.
    fn get<T>(&self) -> Ref<T>
    where
        T: Resource,
    {
        self.try_get().unwrap()
    }

    /// Immutably borrows a resource from this container.
    ///
    /// Returns `None` if the resource does not exist
    /// or if it is already mutably borrowed.
    fn try_get<T>(&self) -> Result<Ref<T>>
    where
        T: Resource,
    {
        self.types
            .get(&TypeId::of::<T>())
            .ok_or_else(|| ResourceError::NotFound(std::any::type_name::<T>()))
            .and_then(|(flag, resource)| {
                if flag.obtain_immutable() {
                    Ok(Ref {
                        flag,
                        value: Box::deref(unsafe { &*resource.get() })
                            .as_any()
                            .downcast_ref()
                            .unwrap(),
                    })
                } else {
                    Err(ResourceError::AlreadyBorrowed)
                }
            })
    }

    /// Mutably borrows a resource from this container.
    ///
    /// # Panics
    /// Panics of the resource does not exist or it is already borrowed.
    fn get_mut<T>(&self) -> RefMut<T>
    where
        T: Resource,
    {
        self.try_get_mut().unwrap()
    }

    /// Mutably borrows a resource from this container.
    ///
    /// Returns `None` if the resource does not exist
    /// or it is already borrowed.
    fn try_get_mut<T>(&self) -> Result<RefMut<T>>
    where
        T: Resource,
    {
        self.types
            .get(&TypeId::of::<T>())
            .ok_or_else(|| ResourceError::NotFound(std::any::type_name::<T>()))
            .and_then(|(flag, resource)| {
                if flag.obtain_mutable() {
                    Ok(RefMut {
                        flag,
                        value: Box::deref_mut(unsafe { &mut *resource.get() })
                            .as_any_mut()
                            .downcast_mut()
                            .unwrap(),
                    })
                } else {
                    Err(ResourceError::AlreadyBorrowed)
                }
            })
    }

    fn as_resources_ref(&self) -> ResourcesEnum {
        ResourcesEnum::Owned(self)
    }
}

type RefEntry = (BorrowFlag, UnsafeCell<*mut dyn Resource>);

pub unsafe trait ResourceTuple<'a> {
    fn into_vec(self) -> ArrayVec<[(TypeId, RefEntry); 4]>;
}

macro_rules! impl_resource_tuple {
    ($($ty:ident, $idx:tt),*) => {
        unsafe impl <'a, $($ty,)*> ResourceTuple<'a> for ($(&'a mut $ty,)*) where $($ty: Resource,)* {
            fn into_vec(self) -> ArrayVec<[(TypeId, RefEntry); 4]> {
                let mut vec = ArrayVec::new();

                $(
                    vec.push((TypeId::of::<$ty>(), (BorrowFlag::default(), UnsafeCell::new(self.$idx as *mut _))));
                )*

                vec
            }
        }
    }
}

impl_resource_tuple!(A, 0);
impl_resource_tuple!(A, 0, B, 1);
impl_resource_tuple!(A, 0, B, 1, C, 2);
impl_resource_tuple!(A, 0, B, 1, C, 2, D, 3);

/// A wrapper over `OwnedResources` which allows insertion of temporary
/// borrows.
pub struct RefResources<'a, R> {
    inner: &'a R,
    refs: ArrayVec<[(TypeId, RefEntry); 4]>,
    _lifetime: PhantomData<&'a mut dyn Resource>,
}

impl<'a, R> RefResources<'a, R> {
    /// Creates a new `RefResources` wrapping the given resources.
    pub fn new(inner: &'a R, refs: impl ResourceTuple<'a>) -> Self {
        Self {
            inner,
            refs: refs.into_vec(),
            _lifetime: PhantomData::default(),
        }
    }
}

impl<'b> ResourcesProvider for RefResources<'b, OwnedResources> {
    fn get<T>(&self) -> Ref<T>
    where
        T: Resource,
    {
        self.try_get::<T>().unwrap()
    }

    fn try_get<T>(&self) -> Result<Ref<T>>
    where
        T: Resource,
    {
        self.refs
            .iter()
            .find(|(id, _)| *id == TypeId::of::<T>())
            .ok_or_else(|| ResourceError::NotFound(std::any::type_name::<T>()))
            .and_then(|(_, (flag, cell))| {
                if flag.obtain_immutable() {
                    Ok(Ref {
                        flag,
                        value: unsafe { &**cell.get() }.as_any().downcast_ref().unwrap(),
                    })
                } else {
                    Err(ResourceError::AlreadyBorrowed)
                }
            })
            .or_else(|_| self.inner.try_get())
    }

    fn get_mut<T>(&self) -> RefMut<T>
    where
        T: Resource,
    {
        self.try_get_mut().unwrap()
    }

    fn try_get_mut<T>(&self) -> Result<RefMut<T>>
    where
        T: Resource,
    {
        self.refs
            .iter()
            .find(|(id, _)| *id == TypeId::of::<T>())
            .ok_or_else(|| ResourceError::NotFound(std::any::type_name::<T>()))
            .and_then(|(_, (flag, cell))| {
                if flag.obtain_mutable() {
                    Ok(RefMut {
                        flag,
                        value: unsafe { &mut **cell.get() }
                            .as_any_mut()
                            .downcast_mut()
                            .unwrap(),
                    })
                } else {
                    Err(ResourceError::AlreadyBorrowed)
                }
            })
            .or_else(|_| self.inner.try_get_mut())
    }

    fn as_resources_ref(&self) -> ResourcesEnum {
        ResourcesEnum::Ref(self)
    }
}

static_assertions::assert_impl_all!(OwnedResources: Send, Sync);
