//! Salamence ex (JTG): Wide Blast — 50 damage to each of the opponent's
//! Benched Pokémon (no Weakness/Resistance). Dragon Impact — 300; discard 2
//! Energy from this Pokémon (ChooseEnergyPrompt for [C][C], no cancel).
//!
//! Twinleaf: Dragon Impact returns early when the Active has no Energy cards
//! attached; otherwise the prompt/discard is the same as
//! DISCARD_X_ENERGY_FROM_THIS_POKEMON(2).
use crate::cards::prelude::*;
use crate::cards::registry::slither_wing::{discard_energy_chosen, discard_x_energy_from_this_pokemon};

pub static IMPL: CardImpl = CardImpl { class: "Salamenceex", mask: mask(&[k::ATTACK, k::AFTER_ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let opp = match *g.e(e) {
            Effect::Attack { opp, .. } => opp as usize,
            _ => return Ok(()),
        };
        let active = g.st.players[opp].active;
        let targets: Vec<SlotId> = for_each_pokemon(g, opp, PlayerType::TopPlayer).iter().filter(|(s, _, _)| *s != active).map(|(s, _, _)| *s).collect();
        for s in targets {
            put_damage(g, e, 50, SlotRef::new(opp, s))?;
        }
    }
    if after_attack_used(g, e, 1, me) {
        let e = real_attack(g, e);
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let a = g.st.players[p].active;
        if !g.st.slot(p, a).cards.iter().any(|c| g.st.cdef(c).is_energy()) {
            return Ok(());
        }
        discard_x_energy_from_this_pokemon(g, me, e, 2, 1)?;
    }
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage == 1 {
        return discard_energy_chosen(g, f, results);
    }
    Ok(())
}
