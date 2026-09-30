//! Sinistcha (PBL / M5): Hide 'n' Sneak. Matcha Spin — if you have 6 or
//! more Pokémon with Hide 'n' Sneak in your discard pile, place 4 damage
//! counters on each of your opponent's Pokémon.
//!
//! Twinleaf: attack damage is zeroed first; one PlaceDamageCountersEffect
//! (source = this card) per opponent Pokémon, Active first.
use super::shuppet::{count_hide_n_sneak_in_discard, reduce_hide_n_sneak, HIDE_N_SNEAK_KINDS};
use crate::cards::prelude::*;
use crate::effects::KindMask;

const MASK: KindMask = mask(&HIDE_N_SNEAK_KINDS).or(mask(&[k::ATTACK]));

pub static IMPL: CardImpl = CardImpl { class: "Sinistcha", mask: MASK, reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    reduce_hide_n_sneak(g, me, e)?;
    if was_attack_used(g, e, 0, me) {
        let (p, opp) = match *g.e(e) {
            Effect::Attack { p, opp, .. } => (p as usize, opp as usize),
            _ => return Ok(()),
        };
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage = 0;
        }
        if count_hide_n_sneak_in_discard(g, p) < 6 {
            return Ok(());
        }
        for (s, _, _) in for_each_pokemon(g, opp, PlayerType::TopPlayer).iter().copied() {
            if g.st.slot(opp, s).cards.is_empty() {
                continue;
            }
            g.run_fx(Effect::PlaceDamageCounters { p: p as u8, target: SlotRef::new(opp, s), damage: 40, source: me })?;
        }
    }
    Ok(())
}
