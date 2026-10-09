//! The per-kind dispatch index (events design section 9): for each effect kind, the cards of the game
//! with a handler for it, in propagation order. `reduce_effect` visits the index's list instead of
//! computing the order from the whole board for every effect.
//!
//! The order is the broadcast's ([`Game::propagation_order_slow`]: zone order over stadium, supporter,
//! Active, Bench, prizes, hand, deck and discard of each player, then a stable sort by rank). It depends
//! on the places of the handler cards only, so a kind's list is rebuilt only when one of its handler
//! cards entered or left a zone, the cards of a zone holding one were reordered, or the Active/Bench
//! layout changed ([`crate::list::unchanged_since`]). Ability locks, the turn and other state don't
//! change the list: the handlers read them when they run.
//!
//! The broadcast stays as the reference: with `PTCG_VERIFY_CACHE=1` (or `PTCG_VERIFY_LEGAL=1`) every
//! lookup is checked against it (same cards, same order).
//!
//! Today's handlers also listen from outside play (a Trainer handles its own TRAINER effect from the
//! hand, cards in the deck and discard are visited too), so the lists hold those cards and change with
//! draws and shuffles that move one of them. Batch 8 narrows each list to the declarations of the kind
//! with their zones.

use crate::effects::Effect;
use crate::game::{prop_class, verify_cache, Game};
use crate::list::*;

/// Entries of the index, and the most cards one entry holds (longer lists are built on each lookup).
const SLOTS: usize = 32;
const CARDS: usize = 16;

/// The index: entries by (effect kind, rank class) in `kind % SLOTS`, each with the layout generation it
/// was built at; and per effect kind the set of cards of the game with a handler for it (a bit per card
/// id), which depends on the cards of the game only.
#[derive(Clone, Copy)]
pub struct DispatchIndex {
    /// The thread whose layout generations the entries carry ([`crate::list::layout_thread`]).
    thread: u64,
    tags: [u16; SLOTS],
    gens: [u64; SLOTS],
    lens: [u8; SLOTS],
    items: [[CardId; CARDS]; SLOTS],
    /// `n_cards` the handler sets were made for.
    cand_cards: u8,
    cand_tags: [u16; SLOTS],
    cands: [u128; SLOTS],
}

impl DispatchIndex {
    const EMPTY_TAG: u16 = u16::MAX;

    pub fn new() -> DispatchIndex {
        DispatchIndex {
            thread: 0,
            tags: [Self::EMPTY_TAG; SLOTS],
            gens: [0; SLOTS],
            lens: [0; SLOTS],
            items: [[NO_CARD; CARDS]; SLOTS],
            cand_cards: 0,
            cand_tags: [Self::EMPTY_TAG; SLOTS],
            cands: [0; SLOTS],
        }
    }
}

impl Default for DispatchIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl Game {
    /// The cards an effect is dispatched to, in propagation order (from the index).
    pub(crate) fn propagation_order(&mut self, e: &Effect, kind: u32) -> SVec<CardId, 120> {
        self.listeners(prop_class(e), kind)
    }

    /// The index's list for `kind` (ranked by `class`); rebuilt when it is stale.
    pub(crate) fn listeners(&mut self, class: u16, kind: u32) -> SVec<CardId, 120> {
        let tag = (kind as u16) << 2 | class;
        let slot = kind as usize % SLOTS;
        let handlers = self.handlers_of(kind);
        let ix = &self.dispatch;
        let out = if ix.tags[slot] == tag && unchanged_since(ix.thread, ix.gens[slot], handlers) {
            let mut out: SVec<CardId, 120> = SVec::new();
            for &c in &ix.items[slot][..ix.lens[slot] as usize] {
                out.push(c);
            }
            out
        } else {
            let thread = layout_thread();
            if self.dispatch.thread != thread {
                // Generations of another thread don't compare with this one's.
                self.dispatch.tags = [DispatchIndex::EMPTY_TAG; SLOTS];
                self.dispatch.thread = thread;
            }
            let out = self.propagation_order_fresh(class, handlers);
            let ix = &mut self.dispatch;
            if out.len() <= CARDS {
                ix.tags[slot] = tag;
                ix.gens[slot] = zone_gen();
                ix.lens[slot] = out.len() as u8;
                ix.items[slot][..out.len()].copy_from_slice(out.as_slice());
            } else if ix.tags[slot] == tag {
                ix.tags[slot] = DispatchIndex::EMPTY_TAG;
            }
            out
        };
        if verify_cache() {
            let slow = self.propagation_order_slow(class, kind);
            assert_eq!(out.as_slice(), slow.as_slice(), "dispatch index differs from the broadcast for kind {} (class {})", kind, class);
        }
        out
    }

    /// The cards of the game with a handler for `kind`, one bit per card id.
    pub(crate) fn handlers_of(&mut self, kind: u32) -> u128 {
        let n = self.st.n_cards;
        let ix = &mut self.dispatch;
        if ix.cand_cards != n {
            ix.cand_tags = [DispatchIndex::EMPTY_TAG; SLOTS];
            ix.cand_cards = n;
        }
        let slot = kind as usize % SLOTS;
        if ix.cand_tags[slot] != kind as u16 {
            let mut set = 0u128;
            for c in 0..n {
                if crate::cards::impl_for(self.st.cards[c as usize].def).is_some_and(|imp| imp.mask.has(kind)) {
                    set |= 1u128 << c;
                }
            }
            let ix = &mut self.dispatch;
            ix.cand_tags[slot] = kind as u16;
            ix.cands[slot] = set;
        }
        self.dispatch.cands[slot]
    }
}
