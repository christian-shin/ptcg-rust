//! Fixed-capacity inline lists (no heap, `Copy`).

use std::fmt;

pub type CardId = u8;
pub const NO_CARD: CardId = u8::MAX;

thread_local! {
    /// Bumped by every mutation of a zone list (`List<N, true>`) and by [`touch`]; see [`zone_gen`].
    static ZONE_GEN: std::cell::Cell<u64> = const { std::cell::Cell::new(1) };
}

/// Records that the card layout of the board changed (a card moved, a zone was shuffled, the Active
/// Spot or the Bench changed). Zone lists call this themselves; code that changes the layout in another
/// way (the Active/Bench slot ids, a whole slot or list assigned) calls it by hand.
#[inline]
pub fn touch() {
    ZONE_GEN.with(|g| g.set(g.get() + 1));
}

/// A counter that changes whenever the card layout of any game on this thread changes: a value computed
/// from the layout (the propagation order) is still good while this is the same.
#[inline]
pub fn zone_gen() -> u64 {
    ZONE_GEN.with(|g| g.get())
}

/// A card list. `Z` marks a zone (hand, deck, discard, prizes, a slot's cards): its mutations [`touch`].
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct List<const N: usize, const Z: bool = false> {
    len: u8,
    items: [CardId; N],
}

/// A card list that is part of the board layout.
pub type ZoneList<const N: usize> = List<N, true>;

impl<const N: usize, const Z: bool> Default for List<N, Z> {
    fn default() -> Self {
        List { len: 0, items: [NO_CARD; N] }
    }
}

impl<const N: usize, const Z: bool> fmt::Debug for List<N, Z> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.as_slice()).finish()
    }
}

/// Object-safe access to any card list regardless of capacity.
pub trait CardList {
    fn as_slice(&self) -> &[CardId];
    fn as_mut_slice(&mut self) -> &mut [CardId];
    fn push(&mut self, c: CardId);
    fn insert(&mut self, i: usize, c: CardId);
    fn remove_at(&mut self, i: usize) -> CardId;
    fn clear(&mut self);
    fn set_from(&mut self, cards: &[CardId]);
    fn capacity(&self) -> usize;

    fn len(&self) -> usize {
        self.as_slice().len()
    }
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn contains(&self, c: CardId) -> bool {
        self.as_slice().contains(&c)
    }
    fn index_of(&self, c: CardId) -> Option<usize> {
        self.as_slice().iter().position(|x| *x == c)
    }
    fn get(&self, i: usize) -> Option<CardId> {
        self.as_slice().get(i).copied()
    }
    /// Remove the first occurrence; returns whether it was present.
    fn remove(&mut self, c: CardId) -> bool {
        match self.index_of(c) {
            Some(i) => {
                self.remove_at(i);
                true
            }
            None => false,
        }
    }
    fn last(&self) -> Option<CardId> {
        self.as_slice().last().copied()
    }
}

impl<const N: usize, const Z: bool> CardList for List<N, Z> {
    #[inline]
    fn as_slice(&self) -> &[CardId] {
        &self.items[..self.len as usize]
    }
    #[inline]
    fn as_mut_slice(&mut self) -> &mut [CardId] {
        if Z {
            touch();
        }
        &mut self.items[..self.len as usize]
    }
    #[inline]
    fn push(&mut self, c: CardId) {
        assert!((self.len as usize) < N, "List<{}> overflow", N);
        if Z {
            touch();
        }
        self.items[self.len as usize] = c;
        self.len += 1;
    }
    fn insert(&mut self, i: usize, c: CardId) {
        assert!((self.len as usize) < N, "List<{}> overflow", N);
        if Z {
            touch();
        }
        let len = self.len as usize;
        self.items.copy_within(i..len, i + 1);
        self.items[i] = c;
        self.len += 1;
    }
    fn remove_at(&mut self, i: usize) -> CardId {
        if Z {
            touch();
        }
        let len = self.len as usize;
        let c = self.items[i];
        self.items.copy_within(i + 1..len, i);
        self.len -= 1;
        self.items[self.len as usize] = NO_CARD;
        c
    }
    #[inline]
    fn clear(&mut self) {
        if Z {
            touch();
        }
        self.len = 0;
    }
    fn set_from(&mut self, cards: &[CardId]) {
        assert!(cards.len() <= N, "List<{}> overflow", N);
        if Z {
            touch();
        }
        self.items[..cards.len()].copy_from_slice(cards);
        self.len = cards.len() as u8;
    }
    fn capacity(&self) -> usize {
        N
    }
}

impl<const N: usize, const Z: bool> List<N, Z> {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn from_slice(cards: &[CardId]) -> Self {
        let mut l = Self::default();
        l.set_from(cards);
        l
    }
    pub fn iter(&self) -> impl Iterator<Item = CardId> + '_ {
        self.as_slice().iter().copied()
    }
}

/// Small fixed-capacity vector of `Copy` values (markers, conditions, ...).
#[derive(Clone, Copy)]
pub struct SVec<T: Copy, const N: usize> {
    len: u8,
    items: [std::mem::MaybeUninit<T>; N],
}

impl<T: Copy, const N: usize> Default for SVec<T, N> {
    fn default() -> Self {
        SVec { len: 0, items: [std::mem::MaybeUninit::uninit(); N] }
    }
}

impl<T: Copy + fmt::Debug, const N: usize> fmt::Debug for SVec<T, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.as_slice()).finish()
    }
}

impl<T: Copy + PartialEq, const N: usize> PartialEq for SVec<T, N> {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<T: Copy, const N: usize> SVec<T, N> {
    /// Write a copy of `self` to `dst`, copying only the live items.
    ///
    /// # Safety
    /// `dst` must be valid for writes and properly aligned.
    #[inline]
    pub unsafe fn copy_live_to(&self, dst: *mut Self) {
        std::ptr::addr_of_mut!((*dst).len).write(self.len);
        let items = std::ptr::addr_of_mut!((*dst).items) as *mut std::mem::MaybeUninit<T>;
        std::ptr::copy_nonoverlapping(self.items.as_ptr(), items, self.len as usize);
    }

    pub fn new() -> Self {
        Self::default()
    }
    #[inline]
    pub const fn capacity(&self) -> usize {
        N
    }
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        // SAFETY: the first `len` items are initialized.
        unsafe { std::slice::from_raw_parts(self.items.as_ptr() as *const T, self.len as usize) }
    }
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        // SAFETY: the first `len` items are initialized.
        unsafe { std::slice::from_raw_parts_mut(self.items.as_mut_ptr() as *mut T, self.len as usize) }
    }
    pub fn len(&self) -> usize {
        self.len as usize
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    pub fn push(&mut self, v: T) {
        assert!((self.len as usize) < N, "SVec<{}> overflow", N);
        self.items[self.len as usize] = std::mem::MaybeUninit::new(v);
        self.len += 1;
    }
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            Some(unsafe { self.items[self.len as usize].assume_init() })
        }
    }
    pub fn remove_at(&mut self, i: usize) -> T {
        let len = self.len as usize;
        assert!(i < len);
        let v = unsafe { self.items[i].assume_init() };
        self.items.copy_within(i + 1..len, i);
        self.len -= 1;
        v
    }
    pub fn insert(&mut self, i: usize, v: T) {
        assert!((self.len as usize) < N, "SVec<{}> overflow", N);
        let len = self.len as usize;
        self.items.copy_within(i..len, i + 1);
        self.items[i] = std::mem::MaybeUninit::new(v);
        self.len += 1;
    }
    pub fn clear(&mut self) {
        self.len = 0;
    }
    pub fn retain(&mut self, mut f: impl FnMut(&T) -> bool) {
        let mut w = 0;
        for r in 0..self.len as usize {
            let v = unsafe { self.items[r].assume_init() };
            if f(&v) {
                self.items[w] = std::mem::MaybeUninit::new(v);
                w += 1;
            }
        }
        self.len = w as u8;
    }
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.as_slice().iter()
    }
    pub fn get(&self, i: usize) -> Option<&T> {
        self.as_slice().get(i)
    }
    pub fn last(&self) -> Option<&T> {
        self.as_slice().last()
    }
}

impl<T: Copy + PartialEq, const N: usize> SVec<T, N> {
    pub fn contains(&self, v: &T) -> bool {
        self.as_slice().contains(v)
    }
    pub fn position(&self, v: &T) -> Option<usize> {
        self.as_slice().iter().position(|x| x == v)
    }
}
