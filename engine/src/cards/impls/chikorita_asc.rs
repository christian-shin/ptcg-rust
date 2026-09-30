//! Chikorita (MC / ASC 8): Growl — during your opponent's next turn, attacks
//! used by the Defending Pokémon do 20 less damage (before W/R). Seed Bomb — 30.
//!
//! DEFENDING_POKEMON_DOES_LESS_DAMAGE: a ReduceDamageEffect setting the
//! opponent Active's `attackDamageReductionNextTurn`.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Chikorita", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        return defending_pokemon_does_less_damage(g, e, 20);
    }
    Ok(())
}

/// `DEFENDING_POKEMON_DOES_LESS_DAMAGE(store, state, effect, source, reduction)`.
pub fn defending_pokemon_does_less_damage(g: &mut Game, atk: EffId, reduction: i32) -> R {
    let b = match *g.e(atk) {
        Effect::Attack { p, opp, attack, source, .. } => {
            let target = SlotRef::new(opp as usize, g.st.players[opp as usize].active);
            AtkBase { attack_effect: atk, player: p, opponent: opp, attack, source, target }
        }
        _ => return Ok(()),
    };
    g.run_fx(Effect::ReduceDamage { b, reduction })?;
    Ok(())
}
