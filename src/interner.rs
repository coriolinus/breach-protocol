use std::{borrow::Borrow, ops::Deref};

/// An interner keeps track of several possibly-non-`Copy` values, and can
/// produce [`Interned`] versions of those values which implement `Copy`.
///
/// Only a single instance of each unique value is stored.
///
/// As each [`Interned`] reference keeps a shared reference to the interner, it
/// is not possible to modify the interner's set of values while any such
/// references exist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Interner<T>(
    // We maintain the invariant that this vector is sorted
    Vec<T>,
);

impl<T> Default for Interner<T> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<T> Interner<T> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<T> Interner<T>
where
    T: Ord,
{
    fn interned(&self, idx: usize) -> Interned<T> {
        Interned {
            interner: self,
            idx,
        }
    }

    /// Get the interned version of the provided value, if it is available in the interner.
    pub fn get<Q>(&self, value: &Q) -> Option<Interned<T>>
    where
        T: Borrow<Q>,
        Q: Ord,
    {
        self.0
            .binary_search_by_key(&value.borrow(), Borrow::borrow)
            .ok()
            .map(|idx| self.interned(idx))
    }

    /// Insert the provided value into this interner.
    ///
    /// If there are many values to insert, [`extend`][self.extend] will likely be more efficient.
    pub fn insert(&mut self, value: T) -> Interned<T> {
        match self.0.binary_search(&value) {
            Ok(idx) => self.interned(idx),
            Err(idx) => {
                self.0.insert(idx, value);
                self.interned(idx)
            }
        }
    }

    /// Insert all provided values into this interner.
    ///
    /// Note that provided values must be pre-collected into a vector. This avoids
    /// hiding the collection costs which would otherwise happen under the hood.
    ///
    /// If `T` is such that items can compare equal without being precisely equal,
    /// and there is a conflict between an existing item and an incoming item, the
    /// existing item is preserved.
    pub fn extend(&mut self, mut values: Vec<T>) {
        // output vector is large enough to contain both input vectors
        let mut merged = Vec::with_capacity(self.0.len() + values.len());

        // take complete ownership of `self.0` and swap in the output vector
        // this is free because `Vec::default()` doesn't allocate
        let mut left = std::mem::take(&mut self.0).into_iter().peekable();

        // sort the input values, then turn them into an iterator
        values.sort_unstable();
        let mut right = values.into_iter().peekable();

        // merge sort
        while let (Some(left_item), Some(right_item)) = (left.peek(), right.peek()) {
            let next = match left_item.cmp(right_item) {
                std::cmp::Ordering::Less => left.next().unwrap(),
                std::cmp::Ordering::Greater => right.next().unwrap(),
                std::cmp::Ordering::Equal => {
                    // we have to advance both iterators
                    right.next();
                    left.next().unwrap()
                }
            };

            // don't duplicate any elements though
            if merged.last() == Some(&next) {
                continue;
            }
            merged.push(next);
        }

        // at least one of the input iterators is exhausted, so it doesn't matter in which order
        // we extend the rest of the items.
        merged.extend(left);
        merged.extend(right);

        // move the new merged list back into `self`.
        self.0 = merged;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Interned<'a, T> {
    interner: &'a Interner<T>,
    idx: usize,
}

impl<'a, T> Deref for Interned<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.interner.0[self.idx]
    }
}

// Interned values can be compared, but can't implement `Ord` because in the event
// that the interners don't match, there simply is no ordering between them.
impl<'a, T> PartialOrd for Interned<'a, T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        (self.interner == other.interner).then(|| self.idx.cmp(&other.idx))
    }
}
