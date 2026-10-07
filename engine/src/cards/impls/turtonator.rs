//! Turtonator (SSP): Fully Singe — discard an Energy from your opponent's
//! Active Pokémon ex. Steaming Stomp — 100.
//!
//! Fully Singe: when the opponent's Active is a Pokémon ex with an Energy
//! card attached, a non-cancellable ChooseCardsPrompt (min 1, max 1) over
//! that Active's Energy, then a DiscardCardsEffect on the chosen card.
//!
//! Fixed (phase 4b, W4): Twinleaf tested `activeCard.cardTag.includes(
//! CardTag.POKEMON_ex)` — the deprecated `cardTag` array, which is empty for
//! every pool Pokémon — so the attack never had an effect. It now uses
//! `hasTag(POKEMON_ex)`. Steaming Stomp costs [F][C][C] (printed data, was [R]).
use super::trubbish::{discard_an_energy_from_opponents_active, discard_chosen};
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Turtonator", mask: mask(&[k::AFTER_ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if after_attack_used(g, e, 0, me) {
        let e = real_attack(g, e);
        let o = match *g.e(e) {
            Effect::Attack { opp, .. } => opp as usize,
            _ => return Ok(()),
        };
        let a = g.st.players[o].active;
        let is_ex = g.st.slot_pokemon(o, a).map(|c| g.st.cdef(c).has_tag(tag::POKEMON_EX_LOWER)).unwrap_or(false);
        if !is_ex {
            return Ok(());
        }
        g.retain_fx(e);
        return discard_an_energy_from_opponents_active(g, me, e, 1);
    }
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage == 1 {
        return discard_chosen(g, f, results);
    }
    Ok(())
}
