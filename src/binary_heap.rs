//! A fixed-capacity priority queue implemented with a binary heap.
//!
//! Insertion and popping the largest element have O(log(n)) time complexity.
//! Checking the largest element is O(1).
//!
//! [`BinaryHeap<E, B, I>`](BinaryHeap) wraps a [`Vec<E, B, I>`](Vec) and
//! can therefore be converted into the underlying vector type at zero cost.
//! Converting a vector to a binary heap can be done in-place, and has O(n)
//! complexity. A binary heap can also be converted to a sorted vector in-place,
//! allowing it to be used for an O(n*log(n)) in-place heapsort.

use crate::storage::{Capacity, ContiguousStorage};
use crate::vec::{Drain, Vec};

use core::fmt;
#[allow(unused_imports)]
use core::mem::MaybeUninit;

/// A fixed-capacity priority queue implemented with a binary heap.
///
/// This will be a max-heap, i.e. [`heap.pop()`](BinaryHeap::pop) will return
/// the largest value in the queue. [`core::cmp::Reverse`] or a custom `Ord`
/// implementation can be used to make a min-heap instead.
///
/// It is a logic error for an item to be modified in such a way that the
/// item's ordering relative to any other item, as determined by the `Ord`
/// trait, changes while it is in the heap. This is normally only possible
/// through `Cell`, `RefCell`, global state, I/O, or unsafe code.
pub struct BinaryHeap<E, B, I = usize>
where
    E: Ord,
    B: ContiguousStorage<E>,
    I: Capacity,
{
    a: Vec<E, B, I>,
}

/// A binary heap using a mutable slice for storage.
///
/// # Examples
/// ```
/// use core::mem::MaybeUninit;
/// let mut backing_array = [MaybeUninit::<char>::uninit(); 32];
/// let (slice1, slice2) = (&mut backing_array[..]).split_at_mut(16);
/// let mut heap1 = coca::SliceHeap::<_>::from(slice1);
/// let mut heap2 = coca::SliceHeap::<_>::from(slice2);
/// assert_eq!(heap1.capacity(), 16);
/// assert_eq!(heap2.capacity(), 16);
/// ```
pub type SliceHeap<'a, E, I = usize> = BinaryHeap<E, crate::storage::SliceStorage<'a, E>, I>;
/// A binary heap using an arena-allocated slice for storage.
pub type ArenaHeap<'a, E, I = usize> = BinaryHeap<E, crate::storage::ArenaStorage<'a, E>, I>;

/// Structure wrapping a mutable reference to the greatest item on a `BinaryHeap`.
///
/// This `struct` is created by the [`BinaryHeap::peek_mut()`] method. See its
/// documentation for more.
pub struct PeekMut<'a, E, B, I = usize>
where
    E: 'a + Ord,
    B: ContiguousStorage<E>,
    I: Capacity,
{
    heap: &'a mut BinaryHeap<E, B, I>,
}

impl<E, B, I> fmt::Debug for PeekMut<'_, E, B, I>
where
    E: Ord + fmt::Debug,
    B: ContiguousStorage<E>,
    I: Capacity,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("PeekMut").field(&self.heap.peek()).finish()
    }
}

impl<E, B, I> Drop for PeekMut<'_, E, B, I>
where
    E: Ord,
    B: ContiguousStorage<E>,
    I: Capacity,
{
    fn drop(&mut self) {
        heapify(self.heap.a.as_mut_slice(), 0);
    }
}

impl<E, B, I> core::ops::Deref for PeekMut<'_, E, B, I>
where
    E: Ord,
    B: ContiguousStorage<E>,
    I: Capacity,
{
    type Target = E;

    fn deref(&self) -> &Self::Target {
        debug_assert!(!self.heap.is_empty());
        unsafe { self.heap.a.get_unchecked(0) }
    }
}

impl<E, B, I> core::ops::DerefMut for PeekMut<'_, E, B, I>
where
    E: Ord,
    B: ContiguousStorage<E>,
    I: Capacity,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        debug_assert!(!self.heap.is_empty());
        unsafe { self.heap.a.get_unchecked_mut(0) }
    }
}

impl<E, B, I> PeekMut<'_, E, B, I>
where
    E: Ord,
    B: ContiguousStorage<E>,
    I: Capacity,
{
    /// Removes the peeked value from the heap and returns it.
    pub fn pop(this: PeekMut<'_, E, B, I>) -> E {
        debug_assert!(!this.heap.is_empty());
        let value = this.heap.pop().unwrap();
        core::mem::forget(this);
        value
    }
}

impl<E, B, I> From<B> for BinaryHeap<E, B, I>
where
    E: Ord,
    B: ContiguousStorage<E>,
    I: Capacity,
{
    /// Converts a contiguous block of memory into an empty binary heap.
    ///
    /// # Panics
    /// This may panic if the index type I cannot represent `buf.capacity()`.
    fn from(buf: B) -> Self {
        BinaryHeap { a: Vec::from(buf) }
    }
}

// This implementatin is largely based on the pseudocode given in
// CLRS - Introduction to Algorithms (third edition), Chapter 6

// These utility functions for binary tree traversal differ from the reference
// because we're using 0-based indexing, i.e. these are equivalent to
// `PARENT(i + 1) - 1`, `LEFT(i + 1) - 1`, and `RIGHT(i + 1) - 1`, respectively.
#[inline(always)]
fn parent(i: usize) -> usize {
    (i + 1) / 2 - 1
}

#[inline(always)]
fn left(i: usize) -> usize {
    2 * (i + 1) - 1
}

#[inline(always)]
fn right(i: usize) -> usize {
    2 * (i + 1)
}

fn heapify<T: Ord>(a: &mut [T], i: usize) {
    let l = left(i);
    let r = right(i);
    let mut largest = if l < a.len() && a[l] > a[i] { l } else { i };
    if r < a.len() && a[r] > a[largest] {
        largest = r;
    }
    if largest != i {
        a.swap(i, largest);
        heapify(a, largest);
    }
}

impl<E, B, I> fmt::Debug for BinaryHeap<E, B, I>
where
    E: Ord + fmt::Debug,
    B: ContiguousStorage<E>,
    I: Capacity,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<E, B, I> From<Vec<E, B, I>> for BinaryHeap<E, B, I>
where
    E: Ord,
    B: ContiguousStorage<E>,
    I: Capacity,
{
    /// Converts a [`Vec`] into a binary heap.
    ///
    /// This conversion happens in-place, and has O(n) time complexity.
    fn from(mut vec: Vec<E, B, I>) -> Self {
        let a = vec.as_mut_slice();
        for i in (0..(a.len() / 2)).rev() {
            heapify(a, i);
        }
        BinaryHeap { a: vec }
    }
}

impl<E, B, I> BinaryHeap<E, B, I>
where
    E: Ord,
    B: ContiguousStorage<E>,
    I: Capacity,
{
    /// Returns a reference to the greatest item in the binary heap, or [`None`] if it is empty.
    #[inline]
    pub fn peek(&self) -> Option<&E> {
        self.a.first()
    }

    /// Returns a mutable reference to the greatest item in the binary heap, or
    /// [`None`] if it is empty.
    ///
    /// Note: If the `PeekMut` value is leaked, the heap may be left in an
    /// inconsistent state.
    ///
    /// # Examples
    /// ```
    /// let mut backing_region = [core::mem::MaybeUninit::<u32>::uninit(); 3];
    /// let mut heap = coca::SliceHeap::<_>::from(&mut backing_region[..]);
    /// heap.try_push(3);
    /// heap.try_push(5);
    /// heap.try_push(1);
    ///
    /// {
    ///     let mut val = heap.peek_mut().unwrap();
    ///     *val = 0;
    /// }
    ///
    /// assert_eq!(heap.pop(), Some(3));
    /// assert_eq!(heap.pop(), Some(1));
    /// assert_eq!(heap.pop(), Some(0));
    /// ```
    #[inline]
    pub fn peek_mut(&mut self) -> Option<PeekMut<E, B, I>> {
        if self.is_empty() {
            None
        } else {
            Some(PeekMut { heap: self })
        }
    }

    /// Removes the greatest element from the binary heap and returns it, or [`None`] if it is empty.
    ///
    /// # Examples
    /// ```
    /// use coca::{SliceHeap, SliceVec};
    /// let mut backing_region = [core::mem::MaybeUninit::<u32>::uninit(); 3];
    /// let mut vec = SliceVec::<u32>::from(&mut backing_region[..]);
    /// vec.push(1); vec.push(3);
    ///
    /// let mut heap = SliceHeap::from(vec);
    ///
    /// assert_eq!(heap.pop(), Some(3));
    /// assert_eq!(heap.pop(), Some(1));
    /// assert_eq!(heap.pop(), None);
    /// ```
    pub fn pop(&mut self) -> Option<E> {
        if self.is_empty() {
            return None;
        }

        let result = self.a.swap_remove(I::from_usize(0));
        heapify(self.a.as_mut_slice(), 0);
        Some(result)
    }

    /// Pushes an item onto the binary heap.
    ///
    /// # Panics
    /// Panics if the heap is already at capacity. See [`try_push`](BinaryHeap::try_push)
    /// for a checked version that never panics.
    #[inline]
    pub fn push(&mut self, item: E) {
        #[cold]
        #[inline(never)]
        fn assert_failed() -> ! {
            panic!("binary heap is already at capacity")
        }

        if self.try_push(item).is_err() {
            assert_failed();
        }
    }

    /// Pushes an item onto the binary heap, returning `Err(item)` if it is full.
    ///
    /// # Examples
    /// ```
    /// let mut backing_region = [core::mem::MaybeUninit::<u32>::uninit(); 3];
    /// let mut heap = coca::SliceHeap::<_>::from(&mut backing_region[..]);
    /// heap.try_push(3);
    /// heap.try_push(5);
    /// heap.try_push(1);
    ///
    /// assert_eq!(heap.len(), 3);
    /// assert_eq!(heap.peek(), Some(&5));
    /// ```
    pub fn try_push(&mut self, item: E) -> Result<(), E> {
        self.a.try_push(item)?;
        let a = self.a.as_mut_slice();
        let mut i = a.len() - 1;
        while i > 0 && a[parent(i)] < a[i] {
            a.swap(i, parent(i));
            i = parent(i);
        }
        Ok(())
    }

    /// Returns the number of elements the binary heap can hold.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.a.capacity()
    }

    /// Returns the number of elements in the binary heap, also referred to as its 'length'.
    #[inline]
    pub fn len(&self) -> usize {
        self.a.len()
    }

    /// Returns `true` if the binary heap contains no elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.a.is_empty()
    }

    /// Returns `true` if the binary heap contains the maximum number of elements.
    #[inline]
    pub fn is_full(&self) -> bool {
        self.a.is_full()
    }

    /// Returns an iterator visiting all values in the underlying vector in arbitrary order.
    pub fn iter(&self) -> impl core::iter::Iterator<Item = &E> {
        self.a.iter()
    }

    /// Clears the binary heap, returning an iterator over the removed elements.
    /// The elements are removed in arbitrary order.
    ///
    /// # Examples
    /// ```
    /// let mut backing_region = [core::mem::MaybeUninit::<u32>::uninit(); 3];
    /// let mut heap = coca::SliceHeap::<_>::from(&mut backing_region[..]);
    /// heap.push(1); heap.push(3);
    /// assert!(!heap.is_empty());
    ///
    /// let mut iter = heap.drain();
    /// assert!(iter.next().is_some());
    /// assert!(iter.next().is_some());
    /// assert!(iter.next().is_none());
    /// drop(iter);
    ///
    /// assert!(heap.is_empty());
    /// ```
    #[inline]
    pub fn drain(&mut self) -> Drain<'_, E, B, I> {
        self.a.drain(..)
    }

    /// Returns an iterator which retrieves elements in heap order. The retrieved
    /// elements are removed from the original heap. The remaining elements will
    /// be removed on drop in heap order.
    ///
    /// # Remarks
    /// `.drain_sorted()` is O(n * log(n)), much slower than [`.drain()`](BinaryHeap::drain).
    /// The latter is preferable in most cases.
    ///
    /// # Examples
    /// ```
    /// let mut backing_region = [core::mem::MaybeUninit::<u32>::uninit(); 3];
    /// let mut heap = coca::SliceHeap::<_>::from(&mut backing_region[..]);
    /// heap.push(1); heap.push(3); heap.push(5);
    ///
    /// let mut iter = heap.drain_sorted();
    /// assert_eq!(iter.next(), Some(5));
    /// drop(iter);
    /// assert!(heap.is_empty());
    /// ```
    #[inline]
    pub fn drain_sorted(&mut self) -> DrainSorted<'_, E, B, I> {
        DrainSorted { heap: self }
    }

    /// Drops all items from the binary heap.
    #[inline]
    pub fn clear(&mut self) {
        self.a.clear();
    }

    /// Consumes the `BinaryHeap` and returns the underlying vector in arbitrary order.
    #[inline]
    pub fn into_vec(self) -> Vec<E, B, I> {
        self.a
    }

    /// Consumes the `BinaryHeap` and returns a vector in sorted (ascending) order.
    ///
    /// # Examples
    /// ```
    /// let mut backing_region = [core::mem::MaybeUninit::<u32>::uninit(); 5];
    /// let mut heap = coca::SliceHeap::<_>::from(&mut backing_region[..]);
    /// heap.push(1); heap.push(5); heap.push(3); heap.push(2); heap.push(4);
    /// let vec = heap.into_sorted_vec();
    /// assert_eq!(vec, &[1, 2, 3, 4, 5][..]);
    /// ```
    pub fn into_sorted_vec(self) -> Vec<E, B, I> {
        let mut result = self.into_vec();
        let a = result.as_mut_slice();
        for i in (1..a.len()).rev() {
            a.swap(0, i);
            heapify(&mut a[..i], 0);
        }
        result
    }

    /// Consumes the `BinaryHeap` and returns an iterator which yields elements
    /// in heap order.
    ///
    /// When dropped, the remaining elements will be dropped in heap order.
    ///
    /// # Remarks
    /// `.into_iter_sorted()` is O(n * log(n)), much slower than [`.into_iter()`](BinaryHeap::into_iter).
    /// The latter is preferable in most cases.
    ///
    /// # Examples
    /// ```
    /// let mut backing_region = [core::mem::MaybeUninit::<u32>::uninit(); 3];
    /// let mut heap = coca::SliceHeap::<_>::from(&mut backing_region[..]);
    /// heap.push(1); heap.push(3); heap.push(5);
    ///
    /// let mut iter = heap.into_iter_sorted();
    /// assert_eq!(iter.next(), Some(5));
    /// assert_eq!(iter.next(), Some(3));
    /// assert_eq!(iter.next(), Some(1));
    /// ```
    pub fn into_iter_sorted(self) -> IntoIterSorted<E, B, I> {
        IntoIterSorted { heap: self }
    }
}

impl<E, B, I> IntoIterator for BinaryHeap<E, B, I>
where
    E: Ord,
    B: ContiguousStorage<E>,
    I: Capacity,
{
    type Item = E;
    type IntoIter = <Vec<E, B, I> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.a.into_iter()
    }
}

impl<E1, E2, B, I> core::iter::Extend<E1> for BinaryHeap<E2, B, I>
where
    Vec<E2, B, I>: core::iter::Extend<E1>,
    E2: Ord,
    B: ContiguousStorage<E2>,
    I: Capacity,
{
    fn extend<T: IntoIterator<Item = E1>>(&mut self, iter: T) {
        self.a.extend(iter);
        for i in (0..(self.a.len() / 2)).rev() {
            heapify(self.a.as_mut_slice(), i);
        }
    }
}

impl<E, B, I> core::iter::FromIterator<E> for BinaryHeap<E, B, I>
where
    Vec<E, B, I>: core::iter::FromIterator<E>,
    E: Ord,
    B: ContiguousStorage<E>,
    I: Capacity,
{
    /// Creates a binary heap from an iterator.
    ///
    /// # Panics
    /// Panics if the iterator yields more elements than the binary heap can hold.
    fn from_iter<It: IntoIterator<Item = E>>(iter: It) -> Self {
        let a = Vec::<E, B, I>::from_iter(iter);
        Self::from(a)
    }
}

/// A draining iterator over the elements of a `BinaryHeap`.
///
/// This `struct` is created by [`BinaryHeap::drain_sorted()`].
/// See its documentation for more.
pub struct DrainSorted<'a, E, B, I>
where
    E: Ord,
    B: ContiguousStorage<E>,
    I: Capacity,
{
    heap: &'a mut BinaryHeap<E, B, I>,
}

impl<E, B, I> Iterator for DrainSorted<'_, E, B, I>
where
    E: Ord,
    B: ContiguousStorage<E>,
    I: Capacity,
{
    type Item = E;

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.len();
        (size, Some(size))
    }

    fn next(&mut self) -> Option<Self::Item> {
        self.heap.pop()
    }
}

impl<E, B, I> core::iter::ExactSizeIterator for DrainSorted<'_, E, B, I>
where
    E: Ord,
    B: ContiguousStorage<E>,
    I: Capacity,
{
}
impl<E, B, I> core::iter::FusedIterator for DrainSorted<'_, E, B, I>
where
    E: Ord,
    B: ContiguousStorage<E>,
    I: Capacity,
{
}

impl<E, B, I> Drop for DrainSorted<'_, E, B, I>
where
    E: Ord,
    B: ContiguousStorage<E>,
    I: Capacity,
{
    fn drop(&mut self) {
        self.for_each(drop);
    }
}

/// A consuming iterator that moves out of a `BinaryHeap`.
///
/// This `struct` is created by [`BinaryHeap::into_iter_sorted()`].
/// See its documentation for more.
#[derive(Debug)]
pub struct IntoIterSorted<E, B, I>
where
    E: Ord,
    B: ContiguousStorage<E>,
    I: Capacity,
{
    heap: BinaryHeap<E, B, I>,
}

impl<E, B, I> Iterator for IntoIterSorted<E, B, I>
where
    E: Ord,
    B: ContiguousStorage<E>,
    I: Capacity,
{
    type Item = E;

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.heap.len();
        (size, Some(size))
    }

    #[inline]
    fn next(&mut self) -> Option<E> {
        self.heap.pop()
    }
}

impl<E, B, I> core::iter::ExactSizeIterator for IntoIterSorted<E, B, I>
where
    E: Ord,
    B: ContiguousStorage<E>,
    I: Capacity,
{
}
impl<E, B, I> core::iter::FusedIterator for IntoIterSorted<E, B, I>
where
    E: Ord,
    B: ContiguousStorage<E>,
    I: Capacity,
{
}

impl<E, B, I> Clone for IntoIterSorted<E, B, I>
where
    BinaryHeap<E, B, I>: Clone,
    E: Clone + Ord,
    B: ContiguousStorage<E>,
    I: Capacity,
{
    fn clone(&self) -> Self {
        self.heap.clone().into_iter_sorted()
    }
}

impl<E, B, I> Drop for IntoIterSorted<E, B, I>
where
    E: Ord,
    B: ContiguousStorage<E>,
    I: Capacity,
{
    fn drop(&mut self) {
        self.for_each(drop);
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docs_rs, doc(cfg(feature = "alloc")))]
/// A binary heap using a heap-allocated slice for storage.
///
/// Note this still has a fixed capacity, and will never reallocate.
///
/// # Examples
/// ```
/// let mut heap = coca::AllocHeap::<char>::with_capacity(3);
/// heap.push('a');
/// heap.push('b');
/// heap.push('c');
/// assert!(heap.try_push('d').is_err());
/// ```
pub type AllocHeap<E, I = usize> = BinaryHeap<E, crate::storage::HeapStorage<E>, I>;

#[cfg(feature = "alloc")]
#[cfg_attr(docs_rs, doc(cfg(feature = "alloc")))]
impl<E, I> AllocHeap<E, I>
where
    E: Copy + Ord,
    I: Capacity,
{
    /// Constructs a new, empty `AllocHeap<E, I>` with the specified capacity.
    ///
    /// # Panics
    /// Panics if the specified capacity cannot be represented by a `usize`.
    pub fn with_capacity(capacity: I) -> Self {
        BinaryHeap {
            a: Vec::with_capacity(capacity),
        }
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docs_rs, doc(cfg(feature = "alloc")))]
impl<E, I> Clone for AllocHeap<E, I>
where
    E: Copy + Ord,
    I: Capacity,
{
    fn clone(&self) -> Self {
        BinaryHeap { a: self.a.clone() }
    }
}

/// A binary heap using an inline array for storage.
///
/// # Examples
/// ```
/// let mut heap = coca::ArrayHeap::<char, 3>::new();
/// heap.push('a');
/// heap.push('b');
/// heap.push('c');
/// assert_eq!(heap.peek(), Some(&'c'));
/// ```
#[cfg(feature = "nightly")]
#[cfg_attr(docs_rs, doc(cfg(feature = "nightly")))]
pub type ArrayHeap<E, const C: usize> = BinaryHeap<E, crate::storage::InlineStorage<E, C>, usize>;

/// A binary heap using an inline array for storage, generic over the index type.
///
/// # Examples
/// ```
/// let mut heap = coca::TiArrayHeap::<char, u8, 3>::new();
/// heap.push('a');
/// let vec = heap.into_vec();
/// assert_eq!(vec[0u8], 'a');
/// ```
#[cfg(feature = "nightly")]
#[cfg_attr(docs_rs, doc(cfg(feature = "nightly")))]
pub type TiArrayHeap<E, Index, const C: usize> =
    BinaryHeap<E, crate::storage::InlineStorage<E, C>, Index>;

#[cfg(feature = "nightly")]
#[cfg_attr(docs_rs, doc(cfg(feature = "nightly")))]
impl<E, I, const C: usize> BinaryHeap<E, [MaybeUninit<E>; C], I>
where
    E: Ord,
    I: Capacity,
{
    /// Constructs a new, empty `BinaryHeap` backed by an inline array.
    ///
    /// # Panics
    /// Panics if `C` cannot be represented as a value of type `I`.
    ///
    /// # Examples
    /// ```
    /// let heap = coca::ArrayHeap::<char, 4>::new();
    /// assert_eq!(heap.capacity(), 4);
    /// assert!(heap.is_empty());
    /// ```
    pub fn new() -> Self {
        let a = Vec::new();
        BinaryHeap { a }
    }
}

#[cfg(feature = "nightly")]
#[cfg_attr(docs_rs, doc(cfg(feature = "nightly")))]
impl<E, I, const C: usize> Default for BinaryHeap<E, [MaybeUninit<E>; C], I>
where
    E: Ord,
    I: Capacity,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "nightly")]
#[cfg_attr(docs_rs, doc(cfg(feature = "nightly")))]
impl<E, I, const C: usize> Clone for BinaryHeap<E, [MaybeUninit<E>; C], I>
where
    E: Clone + Ord,
    I: Capacity,
{
    fn clone(&self) -> Self {
        BinaryHeap { a: self.a.clone() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tree_traversal_utilities() {
        assert_eq!(left(0), 1);
        assert_eq!(right(0), 2);
        assert_eq!(parent(1), 0);
        assert_eq!(parent(2), 0);

        for i in 1..=1000 {
            let l = left(i);
            let r = right(i);
            assert_eq!(l + 1, r);
            assert_eq!(parent(l), i);
            assert_eq!(parent(r), i);

            let ll = left(l);
            let lr = right(l);
            let rl = left(r);
            let rr = right(r);

            assert_eq!(ll + 1, lr);
            assert_eq!(rl + 1, rr);
            assert_eq!(parent(parent(ll)), i);
            assert_eq!(parent(parent(lr)), i);
            assert_eq!(parent(parent(rl)), i);
            assert_eq!(parent(parent(rr)), i);
        }
    }

    #[test]
    fn push_and_pop_randomized_inputs() {
        use rand::{rngs::SmallRng, RngCore, SeedableRng};

        let mut backing_region = [core::mem::MaybeUninit::<u32>::uninit(); 32];
        let mut heap = SliceHeap::<_>::from(&mut backing_region[..]);

        let mut rng = SmallRng::from_seed([
            0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc,
            0xde, 0xf0, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x12, 0x34, 0x56, 0x78,
            0x9a, 0xbc, 0xde, 0xf0,
        ]);

        let mut newest = 0;
        for _ in 0..32 {
            newest = rng.next_u32();
            heap.push(newest);
        }

        let mut prev = u32::max_value();
        for _ in 0..1000 {
            let x = heap.pop().unwrap();
            assert!(x <= prev || x == newest);
            prev = x;

            newest = rng.next_u32();
            heap.push(newest);
        }
    }

    #[test]
    fn iterators_take_and_drop_correctly() {
        use core::cell::RefCell;

        #[derive(Clone)]
        struct Droppable<'a, 'b> {
            value: usize,
            log: &'a RefCell<crate::SliceVec<'b, usize>>,
        }

        impl PartialEq for Droppable<'_, '_> {
            fn eq(&self, rhs: &Self) -> bool {
                self.value == rhs.value
            }
        }

        impl Eq for Droppable<'_, '_> {}

        impl PartialOrd for Droppable<'_, '_> {
            fn partial_cmp(&self, rhs: &Self) -> Option<core::cmp::Ordering> {
                Some(self.cmp(rhs))
            }
        }

        impl Ord for Droppable<'_, '_> {
            fn cmp(&self, rhs: &Self) -> core::cmp::Ordering {
                self.value.cmp(&rhs.value)
            }
        }

        impl Drop for Droppable<'_, '_> {
            fn drop(&mut self) {
                self.log.borrow_mut().push(self.value);
            }
        }

        let mut backing_array = [MaybeUninit::<usize>::uninit(); 16];
        let drop_log = RefCell::new(crate::SliceVec::<_>::from(&mut backing_array[..]));

        let mut backing_region = [
            core::mem::MaybeUninit::<Droppable>::uninit(),
            core::mem::MaybeUninit::<Droppable>::uninit(),
            core::mem::MaybeUninit::<Droppable>::uninit(),
            core::mem::MaybeUninit::<Droppable>::uninit(),
            core::mem::MaybeUninit::<Droppable>::uninit(),
            core::mem::MaybeUninit::<Droppable>::uninit(),
            core::mem::MaybeUninit::<Droppable>::uninit(),
            core::mem::MaybeUninit::<Droppable>::uninit(),
        ];

        let mut heap = SliceHeap::<Droppable>::from(&mut backing_region[..]);
        for i in 1..=8 {
            heap.push(Droppable {
                value: i,
                log: &drop_log,
            });
        }

        let mut drain_iter = heap.drain_sorted();
        assert_eq!(drain_iter.next().unwrap().value, 8);
        assert_eq!(drain_iter.next().unwrap().value, 7);
        assert_eq!(drop_log.borrow().len(), 2);

        drop(drain_iter);
        assert_eq!(drop_log.borrow().len(), 8);
        assert_eq!(heap.len(), 0);

        for i in 1..=8 {
            heap.push(Droppable {
                value: i,
                log: &drop_log,
            });
        }

        let mut into_iter = heap.into_iter_sorted();
        assert_eq!(into_iter.next().unwrap().value, 8);
        assert_eq!(into_iter.next().unwrap().value, 7);
        assert_eq!(into_iter.next().unwrap().value, 6);
        assert_eq!(drop_log.borrow().len(), 11);

        drop(into_iter);
        assert_eq!(drop_log.borrow().len(), 16);

        assert_eq!(
            drop_log.borrow().as_slice(),
            &[8, 7, 6, 5, 4, 3, 2, 1, 8, 7, 6, 5, 4, 3, 2, 1]
        );
    }
}
