//! Implements an `AtomicRefCell`

use core::cell::UnsafeCell;
use core::fmt::{self, Display, Formatter};
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicU32, Ordering};

/// Error returned when an `AtomicRefCell`
/// is already mutably borrowed.
#[derive(Debug)]
pub struct MutablyBorrowed;

impl Display for MutablyBorrowed {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "value already mutably borrowed")
    }
}

#[cfg(feature = "std")]
impl std::error::Error for MutablyBorrowed {}

/// Error returned when an `AtomicRefCell` has
/// existing immutable borrows.
#[derive(Debug)]
pub struct ImmutablyBorrowed;

impl Display for ImmutablyBorrowed {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "value already immutably borrowed")
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ImmutablyBorrowed {}

/// Like `RefCell`, but atomic. Akin to a `RwLock`
/// which never blocks.
///
/// This is used in this crate for interior mutability
/// of component storages. Note that all components of the
/// same type are wrapped in a single `AtomicRefCell`. This
/// means that you cannot have a mutable reference to two
/// components of the same type at once, even if they belong
/// to different entities.
#[derive(Debug, Default)]
pub struct AtomicRefCell<T> {
    value: UnsafeCell<T>,
    /// Flag has MSB set if currently
    /// mutably borrowed. The other bits indicate
    /// the number of current immutable borrows.
    flag: AtomicU32,
}

const NO_BORROWS: u32 = 0;
const MUTABLE_MASK: u32 = 1 << 31;

impl<T> AtomicRefCell<T> {
    /// Creates a new `AtomicRefCell` initialized with the given value.
    pub fn new(value: T) -> Self {
        Self {
            value: UnsafeCell::new(value),
            flag: AtomicU32::new(NO_BORROWS),
        }
    }

    /// Immutably borrows the value.
    ///
    /// # Panics
    /// Panics if the value is already borrowed mutably.
    /// If you wish to handle this gracefully, use `try_borrow()`.
    pub fn borrow(&self) -> AtomicRef<T> {
        self.try_borrow().unwrap_or_else(|e| {
            panic!(
                "failed to immutably borrow `AtomicRefCell` of type {}: {}",
                core::any::type_name::<T>(),
                e
            )
        })
    }

    /// Mutably borrows the value.
    ///
    /// # Panics
    /// Panics if the value has existing immutable borrows.
    /// If you wish to handle this gracefully, use `try_borrow_mut()`.
    pub fn borrow_mut(&self) -> AtomicRefMut<T> {
        self.try_borrow_mut().unwrap_or_else(|e| {
            panic!(
                "failed to mutably borrow `AtomicRefCell` of type {}: {}",
                core::any::type_name::<T>(),
                e
            )
        })
    }

    /// Attempts to immutably borrow the value.
    ///
    /// Returns an error if the value is already borrowed
    /// mutably.
    pub fn try_borrow(&self) -> Result<AtomicRef<T>, MutablyBorrowed> {
        // Increment the borrow count.
        // Note that the count will even be incremented
        // if there is an existing mutable borrow. However,
        // `release_mut()` stores 0 into the flag, so
        // these "phantom borrows" are cleared out.
        let flag = self.flag.fetch_add(1, Ordering::AcqRel);

        if flag & MUTABLE_MASK != 0 {
            Err(MutablyBorrowed)
        } else {
            Ok(AtomicRef { cell: self })
        }
    }

    /// Attempts to mutably borrow the value.
    ///
    /// Returns an error if the value has one or more
    /// existing immutable borrows.
    pub fn try_borrow_mut(&self) -> Result<AtomicRefMut<T>, ImmutablyBorrowed> {
        // Compare and swap the borrow flag; if the old
        // value is NO_BORROWS, then we have unique access
        // to the value.
        if self
            .flag
            .compare_and_swap(NO_BORROWS, MUTABLE_MASK, Ordering::AcqRel)
            == NO_BORROWS
        {
            Ok(AtomicRefMut { cell: self })
        } else {
            Err(ImmutablyBorrowed)
        }
    }

    fn release_ref(&self) {
        // Decrement the flag value.
        // This causes the immutable reference
        // count to drop by one.
        debug_assert_eq!(self.flag.load(Ordering::Acquire) & MUTABLE_MASK, 0);
        debug_assert!(self.flag.load(Ordering::Acquire) > 0);
        self.flag.fetch_sub(1, Ordering::AcqRel);
    }

    fn release_mut(&self) {
        // Set the flag value to NO_BORROWS.
        // Since a mutable borrow requires exclusivity,
        // releasing it causes no borrows to exist.
        self.flag.store(NO_BORROWS, Ordering::Release);
    }
}

/// RAII guard for an immutably borrowed value
/// from an `AtomicRefCell`.
pub struct AtomicRef<'a, T> {
    cell: &'a AtomicRefCell<T>,
}

impl<'a, T> Deref for AtomicRef<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // Safety: an `AtomicRef` can only be created
        // through `AtomicRefCell::borrow()`, which
        // marks the flag to ensure the borrow is correct.
        unsafe { &*self.cell.value.get() }
    }
}

impl<'a, T> Drop for AtomicRef<'a, T> {
    fn drop(&mut self) {
        self.cell.release_ref();
    }
}

/// RAII guard for a mutably borrowed value
/// from an `AtomicRefCell`.
pub struct AtomicRefMut<'a, T> {
    cell: &'a AtomicRefCell<T>,
}

impl<'a, T> Deref for AtomicRefMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // Safety: see `AtomicRef::deref()`
        unsafe { &*self.cell.value.get() }
    }
}

impl<'a, T> DerefMut for AtomicRefMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // Safety: see `deref()`
        unsafe { &mut *self.cell.value.get() }
    }
}

impl<'a, T> Drop for AtomicRefMut<'a, T> {
    fn drop(&mut self) {
        self.cell.release_mut();
    }
}
