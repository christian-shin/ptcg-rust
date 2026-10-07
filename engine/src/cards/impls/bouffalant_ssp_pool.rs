//! Bouffalant (SSP 151): Ready to Ram — 40; during your opponent's next turn,
//! if this Pokémon is damaged by an attack (even if Knocked Out), put 6 damage
//! counters on the Attacking Pokémon. Smashing Headbutt — 150; discard 2
//! Energy from this Pokémon.
//!
//! Twinleaf: Ready to Ram reduces a RetaliateOnDamageDuringOpponentsNextTurn
//! Effect (`{ damage: 60 }`); Smashing Headbutt is
//! DISCARD_X_ENERGY_FROM_THIS_POKEMON(2) (ruling 1652: Energy units, never more cards than 2; no prompt
//! without Energy on the Active).
use super::slither_wing::{discard_energy_chosen, discard_x_energy_from_this_pokemon, energy_on_active};
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "BouffalantSSPPool", mask: mask(&[k::ATTACK, k::AFTER_ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        if let Effect::Attack { p, opp, attack, source, .. } = *g.e(e) {
            let b = AtkBase { attack_effect: e, player: p, opponent: opp, attack, source, target: source };
            g.run_fx(Effect::RetaliateOnDamage { b, damage: 60, source_card: me })?;
        }
        return Ok(());
    }
    if after_attack_used(g, e, 1, me) {
        let e = real_attack(g, e);
        if energy_on_active(g, e) {
            discard_x_energy_from_this_pokemon(g, me, e, 2, 1)?;
        }
    }
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage == 1 {
        return discard_energy_chosen(g, f, results);
    }
    Ok(())
}
