//! Babiri Berry (SSP 163, Pokémon Tool): like Payapa Berry but against
//! attacks from your opponent's [M] Pokémon (60 less damage, discard this
//! card).
use crate::cards::prelude::*;
use super::payapa_berry_scr_pool::berry_reduce;

pub static IMPL: CardImpl = CardImpl { class: "BabiriBerrySSPPool", mask: mask(&[k::PUT_DAMAGE]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    berry_reduce(g, me, e, ct::METAL)
}
