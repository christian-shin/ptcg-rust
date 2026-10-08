//! Tapu Bulu (SFA): Wood Hammer — 220. This Pokémon also does 30 damage to itself.
//!
//! Twinleaf: a DealDamageEffect (Weakness/Resistance path) on the player's
//! Active, reduced from the AttackEffect handler (before the main damage).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TapuBulu",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(self_damage(30)),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();

// Still called by Hariyama and Walrein until they are converted.
pub fn this_pokemon_does_damage_to_itself(g: &mut crate::game::Game, e: crate::effects::EffId, amount: i32) -> crate::game::R {
    use crate::effects::{AtkBase, Effect};
    let b = match *g.e(e) {
        Effect::Attack { p, opp, attack, source, .. } => AtkBase { attack_effect: e, player: p, opponent: opp, attack, source, target: source },
        _ => return Ok(()),
    };
    g.run_fx(Effect::DealDamage { b, damage: amount })?;
    Ok(())
}
