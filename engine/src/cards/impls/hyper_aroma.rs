//! Hyper Aroma (TWM, ACE SPEC): search your deck for up to 3 Stage 1
//! Pokémon, reveal them, and put them into your hand. Then, shuffle.
//!
//! Same flow as Master Ball (reveal before MOVE_CARDS).
use super::master_ball::{resume, search};
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "HyperAroma", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    let mut filter = Filter::super_type(SuperType::Pokemon);
    filter.stage = Some(Stage::Stage1 as u8);
    search(g, me, e, p, filter, 3)
}
