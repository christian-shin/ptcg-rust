//! Haban Berry (PRE, tool): if the Pokémon this card is attached to is
//! damaged by an attack from your opponent's [N] Pokémon, it takes 60 less
//! damage (after applying Weakness and Resistance), and discard this card.
//!
//! Twinleaf: same shape as Payapa Berry (see `payapa_berry_scr_pool`), keyed
//! on the attacker's type being Dragon (the printed [N]).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "HabanBerry", mask: mask(&[k::PUT_DAMAGE]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    super::payapa_berry_scr_pool::berry_reduce(g, me, e, ct::DRAGON)
}
