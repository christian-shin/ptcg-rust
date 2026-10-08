//! Tynamo (SV11B): Hold Still — heal 10 damage from this Pokémon.
//!
//! Twinleaf: a HealTargetEffect(effect, 10) targeting the player's Active.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Tynamo@BLK|ASC",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(heal_active(10, HealVia::Attack)),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();

// Still called by Zarude (SSP) until it is converted.
pub fn heal_own_active(g: &mut crate::game::Game, e: crate::effects::EffId, amount: i32) -> crate::game::R {
    use crate::effects::{AtkBase, Effect, SlotRef};
    let b = match *g.e(e) {
        Effect::Attack { p, opp, attack, source, .. } => {
            let target = SlotRef::new(p as usize, g.st.players[p as usize].active);
            AtkBase { attack_effect: e, player: p, opponent: opp, attack, source, target }
        }
        _ => return Ok(()),
    };
    g.run_fx(Effect::HealTarget { b, damage: amount })?;
    Ok(())
}
