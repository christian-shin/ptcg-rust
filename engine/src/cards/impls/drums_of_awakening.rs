//! Awakening Drum (TEF, ACE SPEC): draw a card for each of your Ancient
//! Pokémon in play.
//!
//! Twinleaf: one MOVE_CARDS deck→hand with `count` (also run with count 0
//! or an empty deck).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "DrumsOfAwakening", mask: mask(&[k::TRAINER]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    let pl = &g.st.players[p];
    let mut n = 0;
    for s in pl.all_slots().iter() {
        if let Some(c) = g.st.slot_pokemon(p, *s) {
            if g.st.cdef(c).has_tag(tag::ANCIENT) {
                n += 1;
            }
        }
    }
    move_count_from(g, ListRef::Deck(p as u8), ListRef::Hand(p as u8), n, me)
}
