//! Black Kyurem ex (SSP): Ice Age — 90; if the opponent's Active Pokémon is a
//! [N] Pokémon (printed type), it is now Paralyzed. Black Frost — 250; this
//! Pokémon also does 30 damage to itself (a DealDamageEffect aimed at
//! `player.active`).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "BlackKyuremex", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let opp = match *g.e(e) {
            Effect::Attack { opp, .. } => opp as usize,
            _ => return Ok(()),
        };
        let a = g.st.players[opp].active;
        if let Some(c) = g.st.slot_pokemon(opp, a) {
            if g.st.cdef(c).card_type.contains(&ct::DRAGON) {
                add_special_conditions_to_opponent_active(g, e, &[SpecialCondition::Paralyzed])?;
            }
        }
    }
    if was_attack_used(g, e, 1, me) {
        if let Effect::Attack { p, opp, attack, source, .. } = *g.e(e) {
            let target = SlotRef::new(p as usize, g.st.players[p as usize].active);
            let b = AtkBase { attack_effect: e, player: p, opponent: opp, attack, source, target };
            g.run_fx(Effect::DealDamage { b, damage: 30 })?;
        }
    }
    Ok(())
}
