use erasable::{Erasable, ErasedPtr};
use std::alloc::Layout;
use std::ptr::NonNull;

/// A type-erased vector.
pub struct ErasedVec {
    ptr: ErasedPtr,
    capacity: usize,
    len: usize,
    layout: Layout,
}

#[allow(unused)]
impl ErasedVec {
    /// Creates a new, empty type-erased vector of type T.
    pub fn new<T>() -> Self {
        let mut inner = Vec::<T>::new();
        let res = Self {
            ptr: T::erase(NonNull::new(inner.as_mut_ptr()).unwrap()),
            len: inner.len(),
            capacity: inner.capacity(),
            layout: Layout::new::<T>(),
        };
        std::mem::forget(inner);
        res
    }

    /// Pushes a value to this vector.
    ///
    /// # Safety
    /// * `T` must be the same type this vector was initialized with.
    pub unsafe fn push<T>(&mut self, value: T) {
        let mut vec = self.as_vec();
        vec.push(value);
        self.reapply(vec);
    }

    /// Clears this vector.
    pub fn clear(&mut self) {
        self.len = 0;
    }

    /// Extends this vector.
    ///
    /// # Safety
    /// * `T` must be the same type this vector was initialized with.
    pub unsafe fn extend<T>(&mut self, iter: impl IntoIterator<Item = T>) {
        let mut vec = self.as_vec();
        vec.extend(iter);
        self.reapply(vec);
    }

    /// Returns the length of this vector.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns the capacity of this vector.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns a pointer to the values of this vector.
    ///
    /// # Safety
    /// * `T` must be the same type this vector was initialized with.
    pub unsafe fn as_ptr<T>(&self) -> NonNull<T> {
        T::unerase(self.ptr)
    }

    /// Returns an erased pointer to the values of this vector.
    pub fn ptr(&self) -> ErasedPtr {
        self.ptr
    }

    /// Drops this vector as a vector of `T`.
    ///
    /// # Safety
    /// * `T` must be the same type this vector was initialized with.
    pub unsafe fn drop_as<T>(self) {
        let vec = self.as_vec::<T>();
        drop(vec);
        std::mem::forget(self);
    }

    /// Returns this `ErasedVec` as a `Vec<T>`.
    ///
    /// # Safety
    /// * `T` must be the same type this vector was initialized with.
    /// * The returned `Vec` may never be dropped, and it must be reapplied
    /// to this type-erased vector using `reapply` if it is mutated.
    unsafe fn as_vec<T>(&self) -> Vec<T> {
        debug_assert_eq!(self.layout, Layout::new::<T>());
        Vec::from_raw_parts(T::unerase(self.ptr).as_ptr(), self.len, self.capacity)
    }

    /// Reapplies changes to this vector.
    unsafe fn reapply<T>(&mut self, mut vec: Vec<T>) {
        debug_assert_eq!(self.layout, Layout::new::<T>());
        self.ptr = T::erase(NonNull::new_unchecked(vec.as_mut_ptr()));
        self.capacity = vec.capacity();
        self.len = vec.len();
        std::mem::forget(vec);
    }
}

impl Drop for ErasedVec {
    fn drop(&mut self) {
        let layout = LayoutExt::repeat(&self.layout, self.capacity)
            .map(|(k, offs)| {
                debug_assert_eq!(self.layout.size(), offs);
                k
            })
            .expect("arithmetic overflow");

        unsafe {
            std::alloc::dealloc(self.ptr.as_ptr().cast(), layout);
        }
    }
}

trait LayoutExt: Sized {
    fn repeat(&self, n: usize) -> Result<(Self, usize), ()>;
    fn padding_needed_for(&self, align: usize) -> usize;
}

impl LayoutExt for Layout {
    fn repeat(&self, n: usize) -> Result<(Self, usize), ()> {
        // This cannot overflow. Quoting from the invariant of Layout:
        // > `size`, when rounded up to the nearest multiple of `align`,
        // > must not overflow (i.e., the rounded value must be less than
        // > `usize::MAX`)
        let padded_size = self.size() + LayoutExt::padding_needed_for(self, self.align());
        let alloc_size = padded_size.checked_mul(n).ok_or(())?;

        unsafe {
            // self.align is already known to be valid and alloc_size has been
            // padded already.
            Ok((
                Layout::from_size_align_unchecked(alloc_size, self.align()),
                padded_size,
            ))
        }
    }

    fn padding_needed_for(&self, align: usize) -> usize {
        let len = self.size();

        // Rounded up value is:
        //   len_rounded_up = (len + align - 1) & !(align - 1);
        // and then we return the padding difference: `len_rounded_up - len`.
        //
        // We use modular arithmetic throughout:
        //
        // 1. align is guaranteed to be > 0, so align - 1 is always
        //    valid.
        //
        // 2. `len + align - 1` can overflow by at most `align - 1`,
        //    so the &-mask with `!(align - 1)` will ensure that in the
        //    case of overflow, `len_rounded_up` will itself be 0.
        //    Thus the returned padding, when added to `len`, yields 0,
        //    which trivially satisfies the alignment `align`.
        //
        // (Of course, attempts to allocate blocks of memory whose
        // size and padding overflow in the above manner should cause
        // the allocator to yield an error anyway.)

        let len_rounded_up = len.wrapping_add(align).wrapping_sub(1) & !align.wrapping_sub(1);
        len_rounded_up.wrapping_sub(len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn erased_vec() {
        unsafe {
            let mut vec = ErasedVec::new::<Vec<i32>>();
            vec.push(vec![0i32, 10i32]);

            assert_eq!(vec.len(), 1);

            let ptr = vec.as_ptr::<Vec<i32>>().as_ptr();
            let slice = std::slice::from_raw_parts(ptr, vec.len());

            let v = &slice[0];

            assert_eq!(v[0], 0);
            assert_eq!(v[1], 10);
            assert_eq!(v.len(), 2);

            vec.drop_as::<Vec<i32>>();
        }
    }
}
