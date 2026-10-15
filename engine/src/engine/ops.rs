//! Card-list operations: moving cards between lists and slots, and sorting a list.

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

    pub fn lst_mut(&mut self, r: ListRef) -> ListMut<'_> {
        match r {
            ListRef::Temp(i) => ListMut::Plain(&mut self.temps[i as usize]),
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
        self.st.put_at(c, dst);
        if let Some((p, s)) = Self::slot_of(dst) {
            if energy_check && self.is_energy(c) {
                let mut edit = self.st.slot_edit(p, s);
                if !edit.energies().contains(c) {
                    edit.energies().push(c);
                }
            }
        }
    }

    /// Moves the first `count` cards of `src` (all when `None`) to `dst`, for either list kind.
    ///
    /// A slot's `energies` is a view of the Energy
    /// cards that are also in `cards`, so moving `cards` moves them once. Only an entry that is in
    /// `energies` alone (a Pokémon acting as Energy) is moved separately, and
    /// only by a full move. Energy moved into another slot is recorded in that
    /// slot's `energies`.
    pub fn move_to(&mut self, src: ListRef, dst: ListRef, count: Option<usize>) {
        let len = self.lst(src).len();
        let n = count.unwrap_or(len).min(len);
        if self.record.on {
            crate::record::moving_first(self, src, n, dst);
        }
        let mut en_moved: SVec<CardId, 48> = SVec::new();
        if let Some((p, s)) = Self::slot_of(src) {
            let slot = &self.st.players[p].slots[s as usize];
            let full = n >= len;
            let mut orphans: SVec<CardId, 48> = SVec::new();
            for c in slot.energies().iter() {
                if full && !slot.cards().contains(c) {
                    orphans.push(c);
                    en_moved.push(c);
                } else if slot.cards().as_slice()[..n].contains(&c) {
                    en_moved.push(c);
                }
            }
            for &c in orphans.iter() {
                self.lst_mut(dst).push(c);
                self.st.put_at(c, dst);
            }
            for &c in en_moved.iter() {
                self.st.slot_edit(p, s).energies().remove(c);
            }
        }
        let mut moved: SVec<CardId, 120> = SVec::new();
        for &c in &self.lst(src)[..n] {
            moved.push(c);
        }
        for &c in moved.iter() {
            self.note_staged(src, c, dst);
        }
        {
            let mut l = self.lst_mut(src);
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
            self.st.put_at(c, dst);
        }
        if let Some((dp, ds)) = Self::slot_of(dst) {
            for &c in en_moved.iter() {
                let mut edit = self.st.slot_edit(dp, ds);
                if !edit.energies().contains(c) {
                    edit.energies().push(c);
                }
            }
        }
    }

    /// A card moved into a staging list (`Game::temps`) remembers the rules zone it came from
    /// (`CardInst::staged_from`): an event that takes it from there reports that zone as its source.
    #[inline]
    fn note_staged(&mut self, src: ListRef, c: CardId, dst: ListRef) {
        if let ListRef::Temp(i) = dst {
            let inst = &mut self.st.cards[c as usize];
            // From one staging list to another the card keeps the zone it was staged from.
            if let Some(z) = crate::engine::enter::rules_zone_of(src) {
                inst.staged_from = Some(z);
            }
            inst.staged_in = Some(i);
        }
    }

    /// Moves the given cards from `src` to `dst`, for either list kind.
    pub fn move_cards_to(&mut self, src: ListRef, cards: &[CardId], dst: ListRef) {
        for &c in cards {
            self.move_card_to(src, c, dst);
        }
    }

    pub fn move_card_to(&mut self, src: ListRef, c: CardId, dst: ListRef) {
        if self.record.on {
            crate::record::moving(self, src, c, dst);
        }
        self.note_staged(src, c, dst);
        // Entering, leaving or changing slot ends the Abilities the card used (ruling 317).
        if (Self::slot_of(src).is_some() || Self::slot_of(dst).is_some()) && src != dst {
            crate::engine::usage::reset_card(&mut self.st, c);
        }
        if let Some((p, s)) = Self::slot_of(src) {
            // PokemonCardList.moveCardsTo
            if let Some(i) = self.st.slot(p, s).cards().index_of(c) {
                let mut edit = self.st.slot_edit(p, s);
                edit.cards().remove_at(i);
                edit.energies().remove(c);
                drop(edit);
                self.push_to(dst, c, true);
            } else if let Some(i) = self.st.slot(p, s).energies().index_of(c) {
                self.st.slot_edit(p, s).energies().remove_at(i);
                self.lst_mut(dst).push(c);
                self.st.put_at(c, dst);
                if let Some((dp, ds)) = Self::slot_of(dst) {
                    let mut edit = self.st.slot_edit(dp, ds);
                    if !edit.energies().contains(c) {
                        edit.energies().push(c);
                    }
                }
            } else if let Some(i) = self.st.slot(p, s).tools().index_of(c) {
                self.st.slot_edit(p, s).tools().remove_at(i);
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
}
