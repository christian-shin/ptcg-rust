//! Fixed-capacity inline lists (no heap, `Copy`).

use std::fmt;

pub type CardId = u8;
pub const NO_CARD: CardId = u8::MAX;

/// What the card layout tracking keeps per thread: a generation counter bumped by every change, the
/// generation of the last change of the whole layout (the Active/Bench slots, a list replaced wholesale),
/// and per card id the generation of the last change of that card's place (it entered or left a zone
/// list, or the cards of its list were reordered). A value computed from the places of some cards (the
/// dispatch index: the cards with a handler for an effect kind, in zone order) is still good while none
/// of those cards and not the whole layout changed since it was made ([`unchanged_since`]).
pub struct LayoutTrack {
    gen: std::cell::Cell<u64>,
    layout: std::cell::Cell<u64>,
    thread: std::cell::Cell<u64>,
    moved: [std::cell::Cell<u64>; 256],
}

thread_local! {
    static TRACK: LayoutTrack = const {
        LayoutTrack {
            gen: std::cell::Cell::new(1),
            layout: std::cell::Cell::new(1),
            thread: std::cell::Cell::new(0),
            moved: [const { std::cell::Cell::new(0) }; 256],
        }
    };
}

/// Records that the card layout of the board changed as a whole (the Active/Bench slot ids, a whole
/// slot or list assigned, a list's cards rewritten in place). Zone lists record their own changes card
/// by card; code that changes the layout in another way calls this by hand.
#[inline]
pub fn touch() {
    TRACK.with(|t| {
        let g = t.gen.get() + 1;
        t.gen.set(g);
        t.layout.set(g);
    });
}

/// Records that card `c` entered or left a zone list (zone lists do this themselves; code that puts a
/// card back by assigning a saved list calls it by hand).
#[inline]
pub fn mark(c: CardId) {
    TRACK.with(|t| {
        let g = t.gen.get() + 1;
        t.gen.set(g);
        t.moved[c as usize].set(g);
    });
}

/// Records that each of `cards` changed its place (one generation for all).
#[inline]
pub fn mark_all(cards: &[CardId]) {
    TRACK.with(|t| {
        let g = t.gen.get() + 1;
        t.gen.set(g);
        for &c in cards {
            t.moved[c as usize].set(g);
        }
    });
}

/// A counter that changes whenever the card layout of any game on this thread changes.
#[inline]
pub fn zone_gen() -> u64 {
    TRACK.with(|t| t.gen.get())
}

/// This thread's id for [`unchanged_since`] stamps (generations of different threads don't compare).
#[inline]
pub fn layout_thread() -> u64 {
    TRACK.with(|t| match t.thread.get() {
        0 => {
            static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
            let id = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            t.thread.set(id);
            id
        }
        id => id,
    })
}

/// Have the places of the cards in `cards` (a bit per card id) and the whole layout stayed the same since
/// generation `since` of thread `thread` (a [`zone_gen`] read on that thread)?
#[inline]
pub fn unchanged_since(thread: u64, since: u64, cards: u128) -> bool {
    TRACK.with(|t| {
        if t.thread.get() != thread || t.layout.get() > since {
            return false;
        }
        let mut m = cards;
        while m != 0 {
            let c = m.trailing_zeros() as usize;
            if t.moved[c].get() > since {
                return false;
            }
            m &= m - 1;
        }
        true
    })
}

/// Marks what `set_from` changes: the cards that left or entered the list, and all the cards that stay if
/// their order changed.
fn mark_replaced(old: &[CardId], new: &[CardId]) {
    let bits = |l: &[CardId]| {
        let mut m = [0u64; 4];
        for &c in l {
            m[(c >> 6) as usize] |= 1 << (c & 63);
        }
        m
    };
    let (mo, mn) = (bits(old), bits(new));
    let has = |m: &[u64; 4], c: CardId| (m[(c >> 6) as usize] >> (c & 63)) & 1 != 0;
    TRACK.with(|t| {
        let g = t.gen.get() + 1;
        t.gen.set(g);
        for &c in old {
            if !has(&mn, c) {
                t.moved[c as usize].set(g);
            }
        }
        for &c in new {
            if !has(&mo, c) {
                t.moved[c as usize].set(g);
            }
        }
        // The cards in both lists: same relative order?
        let mut a = old.iter().copied().filter(|&c| has(&mn, c));
        let mut b = new.iter().copied().filter(|&c| has(&mo, c));
        let same = loop {
            match (a.next(), b.next()) {
                (None, None) => break true,
                (x, y) if x == y => continue,
                _ => break false,
            }
        };
        if !same {
            for &c in old {
                if has(&mn, c) {
                    t.moved[c as usize].set(g);
                }
            }
        }
    });
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
    /// The cards to permute in place (the same cards come back, in another order).
    fn as_permutable_slice(&mut self) -> &mut [CardId];

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
    fn as_permutable_slice(&mut self) -> &mut [CardId] {
        if Z {
            mark_all(&self.items[..self.len as usize]);
        }
        &mut self.items[..self.len as usize]
    }
    #[inline]
    fn push(&mut self, c: CardId) {
        assert!((self.len as usize) < N, "List<{}> overflow", N);
        if Z {
            mark(c);
        }
        self.items[self.len as usize] = c;
        self.len += 1;
    }
    fn insert(&mut self, i: usize, c: CardId) {
        assert!((self.len as usize) < N, "List<{}> overflow", N);
        if Z {
            mark(c);
        }
        let len = self.len as usize;
        self.items.copy_within(i..len, i + 1);
        self.items[i] = c;
        self.len += 1;
    }
    fn remove_at(&mut self, i: usize) -> CardId {
        let len = self.len as usize;
        let c = self.items[i];
        if Z {
            mark(c);
        }
        self.items.copy_within(i + 1..len, i);
        self.len -= 1;
        self.items[self.len as usize] = NO_CARD;
        c
    }
    #[inline]
    fn clear(&mut self) {
        if Z {
            mark_all(&self.items[..self.len as usize]);
        }
        self.len = 0;
    }
    fn set_from(&mut self, cards: &[CardId]) {
        assert!(cards.len() <= N, "List<{}> overflow", N);
        if Z {
            mark_replaced(&self.items[..self.len as usize], cards);
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

impl<T: Copy, const N: usize> std::ops::Index<usize> for SVec<T, N> {
    type Output = T;
    #[inline]
    fn index(&self, i: usize) -> &T {
        &self.as_slice()[i]
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
