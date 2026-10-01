//! Iron Boulder ex (TEF 99): Repulsor Axe — 60; during your opponent's next
//! turn, if this Pokémon is damaged by an attack (even if it is Knocked Out),
//! put 8 damage counters on the Attacking Pokémon. Power Stomp — 200; discard
//! 2 Energy from this Pokémon.
//!
//! Twinleaf: Repulsor Axe reduces a RetaliateOnDamageDuringOpponentsNextTurn
//! Effect (`{ damage: 80 }`, target = the attacker's slot) arming
//! `retaliateOnDamageNextTurnPending` on the attacker's Active; Power Stomp is
//! DISCARD_UP_TO_X_ENERGY_FROM_THIS_POKEMON(2, {}, 2). The revenge itself
//! lives in the core AfterDamage reducer (see `attack.rs`).
use crate::cards::prelude::*;
use super::team_rockets_houndoom_dri_pool::{discard_up_to_chosen, discard_up_to_x_energy_from_this_pokemon};

pub static IMPL: CardImpl = CardImpl { class: "IronBoulderexTEFPool", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        if let Effect::Attack { p, opp, attack, source, .. } = *g.e(e) {
            let b = AtkBase { attack_effect: e, player: p, opponent: opp, attack, source, target: source };
            g.run_fx(Effect::RetaliateOnDamage { b, damage: 80, source_card: me })?;
        }
    }
    if was_attack_used(g, e, 1, me) {
        discard_up_to_x_energy_from_this_pokemon(g, me, e, 2, 2, 1)?;
    }
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage == 1 {
        return discard_up_to_chosen(g, f, results);
    }
    Ok(())
}
