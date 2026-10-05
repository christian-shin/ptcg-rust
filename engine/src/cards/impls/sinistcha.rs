//! Sinistcha (PBL / M5): Hide 'n' Sneak. Matcha Spin — if you have 6 or
//! more Pokémon with Hide 'n' Sneak in your discard pile, place 4 damage
//! counters on each of your opponent's Pokémon.
//!
//! Twinleaf: attack damage is zeroed first; then
//! PUT_X_DAMAGE_COUNTERS_ON_ALL_YOUR_OPPONENTS_POKEMON(4): one PutCountersEffect
//! (an effect of the attack) on the opponent's Active, then one per Benched
//! Pokémon. Fixed (phase 4b, R3): this used to be one PlaceDamageCountersEffect
//! (source = this card) per Pokémon, which Mist Energy and Spherical Shield
//! don't see.
use super::shuppet::{count_hide_n_sneak_in_discard, reduce_hide_n_sneak, HIDE_N_SNEAK_KINDS};
use crate::cards::prelude::*;
use crate::effects::KindMask;

const MASK: KindMask = mask(&HIDE_N_SNEAK_KINDS).or(mask(&[k::ATTACK]));

pub static IMPL: CardImpl = CardImpl { class: "Sinistcha", mask: MASK, reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    reduce_hide_n_sneak(g, me, e)?;
    if was_attack_used(g, e, 0, me) {
        let (p, opp, attack, source) = match *g.e(e) {
            Effect::Attack { p, opp, attack, source, .. } => (p as usize, opp as usize, attack, source),
            _ => return Ok(()),
        };
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage = 0;
        }
        if count_hide_n_sneak_in_discard(g, p) < 6 {
            return Ok(());
        }
        let active = g.st.players[opp].active;
        let b = AtkBase { attack_effect: e, player: p as u8, opponent: opp as u8, attack, source, target: SlotRef::new(opp, active) };
        g.run_fx(Effect::PutCounters { b, damage: 40 })?;
        let bench: Vec<SlotId> = g.st.players[opp].bench.iter().copied().collect();
        for s in bench {
            if g.st.slot(opp, s).cards.is_empty() {
                continue;
            }
            let b = AtkBase { attack_effect: e, player: p as u8, opponent: opp as u8, attack, source, target: SlotRef::new(opp, s) };
            g.run_fx(Effect::PutCounters { b, damage: 40 })?;
        }
    }
    Ok(())
}
