//! Dusknoir (SFA): Cursed Blast — put 13 damage counters on 1 of your
//! opponent's Pokémon, then this Pokémon is Knocked Out. Shadow Bind — 150,
//! the Defending Pokémon can't retreat during your opponent's next turn.
//!
//! Same Twinleaf structure as Dusclops (no once-per-turn marker, `damage += 999`).
use super::dusclops::{cursed_blast_reduce, cursed_blast_resume};
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Dusknoir", mask: mask(&[k::POWER, k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_power_used(g, e, 0, me) {
        return cursed_blast_reduce(g, me, e);
    }
    if was_attack_used(g, e, 0, me) {
        return block_retreat(g, e);
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    cursed_blast_resume(g, me, f, results, 130)
}
