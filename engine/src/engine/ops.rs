//! Card-list operations with Twinleaf's exact semantics (`CardList.moveTo`,
//! `moveCardsTo`, `PokemonCardList` overrides, `sort`), including quirks
//! such as energies being pushed twice by a full `PokemonCardList.moveTo`.

use crate::game::Game;
use crate::list::*;
use crate::state::*;
use crate::types::*;

impl Game {
    pub fn lst(&self, r: ListRef) -> &[CardId] {
        match r {
            ListRef::Temp(i) => self.temps[i as usize].as_slice(),
            _ => self.st.list(r).as_slice(),
        }
    }

    pub fn lst_mut(&mut self, r: ListRef) -> &mut dyn CardList {
        match r {
            ListRef::Temp(i) => &mut self.temps[i as usize],
            _ => self.st.list_mut(r),
        }
    }

    fn slot_of(r: ListRef) -> Option<(usize, SlotId)> {
        match r {
            ListRef::Slot(p, s) => Some((p as usize, s)),
            _ => None,
        }
    }

    fn is_energy(&self, c: CardId) -> bool {
        self.st.cdef(c).super_type == SuperType::Energy as u8
    }

    /// Push onto a destination list; if it is a slot and the card is an
    /// Energy card, also record it in the slot's energies.
    fn push_to(&mut self, dst: ListRef, c: CardId, energy_check: bool) {
        self.lst_mut(dst).push(c);
        if let Some((p, s)) = Self::slot_of(dst) {
            if energy_check && self.is_energy(c) {
                let e = &mut self.st.players[p].slots[s as usize].energies;
                if !e.contains(c) {
                    e.push(c);
                }
            }
        }
    }

    /// `CardList.moveTo(destination, count)` / `PokemonCardList.moveTo`.
    pub fn move_to(&mut self, src: ListRef, dst: ListRef, count: Option<usize>) {
        if let Some((p, s)) = Self::slot_of(src) {
            // Energies CardList is moved first (plain moveTo into dst.cards).
            let en: SVec<CardId, 48> = {
                let mut v = SVec::new();
                for c in self.st.players[p].slots[s as usize].energies.iter() {
                    v.push(c);
                }
                v
            };
            if !en.is_empty() {
                self.st.players[p].slots[s as usize].energies.clear();
                for &c in en.iter() {
                    self.lst_mut(dst).push(c);
                }
            }
        }
        let len = self.lst(src).len();
        let n = count.unwrap_or(len).min(len);
        let mut moved: SVec<CardId, 120> = SVec::new();
        for &c in &self.lst(src)[..n] {
            moved.push(c);
        }
        {
            let l = self.lst_mut(src);
            let rest: SVec<CardId, 120> = {
                let mut v = SVec::new();
                for &c in &l.as_slice()[n..] {
                    v.push(c);
                }
                v
            };
            l.set_from(rest.as_slice());
        }
        for &c in moved.iter() {
            self.lst_mut(dst).push(c);
        }
    }

    /// `moveToTopOfDestination`: prepends a copy; the source keeps its cards.
    pub fn move_to_top_of_destination(&mut self, src: ListRef, dst: ListRef) {
        let mut v: SVec<CardId, 120> = SVec::new();
        for &c in self.lst(src) {
            v.push(c);
        }
        for &c in self.lst(dst) {
            v.push(c);
        }
        self.lst_mut(dst).set_from(v.as_slice());
    }

    /// `moveCardsTo(cards, destination)` for either list kind.
    pub fn move_cards_to(&mut self, src: ListRef, cards: &[CardId], dst: ListRef) {
        for &c in cards {
            self.move_card_to(src, c, dst);
        }
    }

    pub fn move_card_to(&mut self, src: ListRef, c: CardId, dst: ListRef) {
        if let Some((p, s)) = Self::slot_of(src) {
            // PokemonCardList.moveCardsTo
            if let Some(i) = self.st.players[p].slots[s as usize].cards.index_of(c) {
                self.st.players[p].slots[s as usize].cards.remove_at(i);
                self.st.players[p].slots[s as usize].energies.remove(c);
                self.push_to(dst, c, true);
            } else if let Some(i) = self.st.players[p].slots[s as usize].energies.index_of(c) {
                self.st.players[p].slots[s as usize].energies.remove_at(i);
                self.lst_mut(dst).push(c);
                if let Some((dp, ds)) = Self::slot_of(dst) {
                    let e = &mut self.st.players[dp].slots[ds as usize].energies;
                    if !e.contains(c) {
                        e.push(c);
                    }
                }
            } else if let Some(i) = self.st.players[p].slots[s as usize].tools.index_of(c) {
                self.st.players[p].slots[s as usize].tools.remove_at(i);
                self.push_to(dst, c, true);
            }
        } else {
            // CardList.moveCardsTo (source is never a PokemonCardList here)
            if let Some(i) = self.lst(src).iter().position(|x| *x == c) {
                self.lst_mut(src).remove_at(i);
                self.push_to(dst, c, true);
            }
        }
    }

    /// `CardList.sort()` with V8's TimSort (binary insertion for < 64 items)
    /// and Twinleaf's comparator (which never returns 0).
    pub fn sort_list(&mut self, r: ListRef) {
        let mut v: SVec<CardId, 120> = SVec::new();
        for &c in self.lst(r) {
            v.push(c);
        }
        let st = &self.st;
        let cmp = |a: CardId, b: CardId| -> i32 {
            let da = st.cdef(a);
            let db = st.cdef(b);
            let sup = |t: u8| match t {
                1 => 1,
                2 => 2,
                3 => 3,
                _ => 100,
            };
            let r = sup(da.super_type) - sup(db.super_type);
            if r != 0 {
                return r;
            }
            if da.is_trainer() && db.is_trainer() {
                let tt = |t: u8| match t {
                    1 => 1, // supporter
                    0 => 2, // item
                    3 => 3, // tool
                    2 => 4, // stadium
                    _ => 100,
                };
                let s = tt(da.trainer_type) - tt(db.trainer_type);
                if s != 0 {
                    return s;
                }
            } else if da.is_energy() && db.is_energy() {
                let et = |t: u8| if t == 0 { 1 } else { 2 };
                let s = et(da.energy_type) - et(db.energy_type);
                if s != 0 {
                    return s;
                }
            }
            if js_str_lt(da.name, db.name) {
                -1
            } else {
                1
            }
        };
        v8_sort(v.as_mut_slice(), &cmp);
        self.lst_mut(r).set_from(v.as_slice());
    }
}

/// JavaScript string `<` (UTF-16 code unit order).
pub fn js_str_lt(a: &str, b: &str) -> bool {
    let mut ai = a.encode_utf16();
    let mut bi = b.encode_utf16();
    loop {
        match (ai.next(), bi.next()) {
            (Some(x), Some(y)) => {
                if x != y {
                    return x < y;
                }
            }
            (None, Some(_)) => return true,
            _ => return false,
        }
    }
}

/// V8 `Array.prototype.sort` for arrays shorter than 64 elements:
/// CountAndMakeRun then BinaryInsertionSort, with the comparator as given.
pub fn v8_sort<T: Copy>(a: &mut [T], cmp: &dyn Fn(T, T) -> i32) {
    let n = a.len();
    if n < 2 {
        return;
    }
    if n >= 64 {
        // V8's merge phase (with galloping) is not reproduced; only reachable
        // when Twinleaf's card duplication grows a deck past 63 cards.
        let mut v: Vec<T> = a.to_vec();
        merge_sort(&mut v, cmp);
        a.copy_from_slice(&v);
        return;
    }
    // CountAndMakeRun(0, n)
    let run = {
        let low = 1;
        if low == n {
            1
        } else {
            let mut run_length = 2;
            let mut prev = a[low];
            let order = cmp(a[low], a[low - 1]);
            let descending = order < 0;
            for idx in low + 1..n {
                let cur = a[idx];
                let o = cmp(cur, prev);
                if descending {
                    if o >= 0 {
                        break;
                    }
                } else if o < 0 {
                    break;
                }
                prev = cur;
                run_length += 1;
            }
            if descending {
                a[..run_length].reverse();
            }
            run_length
        }
    };
    // BinaryInsertionSort(0, run, n)
    let mut start = if run == 0 { 1 } else { run };
    while start < n {
        let pivot = a[start];
        let mut left = 0;
        let mut right = start;
        while left < right {
            let mid = left + ((right - left) >> 1);
            if cmp(pivot, a[mid]) < 0 {
                right = mid;
            } else {
                left = mid + 1;
            }
        }
        let mut p = start;
        while p > left {
            a[p] = a[p - 1];
            p -= 1;
        }
        a[left] = pivot;
        start += 1;
    }
}

fn merge_sort<T: Copy>(v: &mut Vec<T>, cmp: &dyn Fn(T, T) -> i32) {
    if v.len() < 2 {
        return;
    }
    let mut right = v.split_off(v.len() / 2);
    merge_sort(v, cmp);
    merge_sort(&mut right, cmp);
    let left = std::mem::take(v);
    let (mut i, mut j) = (0, 0);
    while i < left.len() && j < right.len() {
        if cmp(right[j], left[i]) < 0 {
            v.push(right[j]);
            j += 1;
        } else {
            v.push(left[i]);
            i += 1;
        }
    }
    v.extend_from_slice(&left[i..]);
    v.extend_from_slice(&right[j..]);
}
