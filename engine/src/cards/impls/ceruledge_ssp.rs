//! Ceruledge (SSP): Blaze Curse — discard all Special Energy from each of your
//! opponent's Pokémon. Amethyst Rage — 160; during your next turn, this
//! Pokémon can't attack.
//!
//! Twinleaf: for the opponent's Active, then each Bench slot, one MOVE_CARDS
//! (no source card) of the attached Special Energy to their discard pile.
//! R7A (ruling 1620 and the attack flow chart): the Special Energy is discarded after the damage (`move_cards_after_damage`).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Ceruledge@SSP", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn discard_special(g: &mut Game, atk: EffId, o: usize, s: SlotId) -> R {
    let cards: Vec<CardId> = g
        .st
        .slot(o, s)
        .cards
        .iter()
        .filter(|c| {
            let d = g.st.cdef(*c);
            d.is_energy() && d.energy_type == EnergyType::Special as u8
        })
        .collect();
    if !cards.is_empty() {
        move_cards_after_damage(g, atk, ListRef::Slot(o as u8, s), ListRef::Discard(o as u8), &cards, NO_CARD)?;
    }
    Ok(())
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let o = match *g.e(e) {
            Effect::Attack { opp, .. } => opp as usize,
            _ => return Ok(()),
        };
        let a = g.st.players[o].active;
        discard_special(g, e, o, a)?;
        let bench: Vec<SlotId> = g.st.players[o].bench.iter().copied().collect();
        for s in bench {
            discard_special(g, e, o, s)?;
        }
    }
    if was_attack_used(g, e, 1, me) {
        if let Effect::Attack { p, .. } = *g.e(e) {
            let p = p as usize;
            let a = g.st.players[p].active;
            g.st.players[p].slots[a as usize].cannot_attack_next_turn_pending = true;
        }
    }
    Ok(())
}
