//! Poltchageist (PBL / M5): Hide 'n' Sneak. Furtive Drop — place 1 damage
//! counter on your opponent's Active Pokémon (a PutCountersEffect).
use super::shuppet::{reduce_hide_n_sneak, HIDE_N_SNEAK_KINDS};
use crate::cards::prelude::*;
use crate::effects::KindMask;

const MASK: KindMask = mask(&HIDE_N_SNEAK_KINDS).or(mask(&[k::ATTACK]));

pub static IMPL: CardImpl = CardImpl { class: "Poltchageist", mask: MASK, reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    reduce_hide_n_sneak(g, me, e)?;
    if was_attack_used(g, e, 0, me) {
        if let Effect::Attack { p, opp, attack, source, .. } = *g.e(e) {
            let target = SlotRef::new(opp as usize, g.st.players[opp as usize].active);
            let b = AtkBase { attack_effect: e, player: p, opponent: opp, attack, source, target };
            g.run_fx(Effect::PutCounters { b, damage: 10 })?;
        }
    }
    Ok(())
}
