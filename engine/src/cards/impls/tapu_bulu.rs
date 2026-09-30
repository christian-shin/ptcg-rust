//! Tapu Bulu (SFA): Wood Hammer — 220. This Pokémon also does 30 damage to itself.
//!
//! Twinleaf: a DealDamageEffect (Weakness/Resistance path) on the player's
//! Active, reduced from the AttackEffect handler (before the main damage).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "TapuBulu", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let b = match *g.e(e) {
            Effect::Attack { p, opp, attack, source, .. } => {
                let target = SlotRef::new(p as usize, g.st.players[p as usize].active);
                AtkBase { attack_effect: e, player: p, opponent: opp, attack, source, target }
            }
            _ => return Ok(()),
        };
        g.run_fx(Effect::DealDamage { b, damage: 30 })?;
    }
    Ok(())
}

/// `THIS_POKEMON_DOES_DAMAGE_TO_ITSELF(store, state, effect, amount)`: target = effect.source.
pub fn this_pokemon_does_damage_to_itself(g: &mut Game, e: EffId, amount: i32) -> R {
    let b = match *g.e(e) {
        Effect::Attack { p, opp, attack, source, .. } => AtkBase { attack_effect: e, player: p, opponent: opp, attack, source, target: source },
        _ => return Ok(()),
    };
    g.run_fx(Effect::DealDamage { b, damage: amount })?;
    Ok(())
}
