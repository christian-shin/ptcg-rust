//! Applin (DRI 16): Mini Drain — 10; heal 10 damage from this Pokémon.
//!
//! Twinleaf: a HealTargetEffect on `player.active`. Two `Applin` classes
//! exist; this port is bound to DRI.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Applin@DRI",
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

// Still called by Alolan Exeggutor until it is converted.
pub fn heal_this_pokemon(g: &mut crate::game::Game, e: crate::effects::EffId, damage: i32) -> crate::game::R {
    use crate::effects::{AtkBase, Effect, SlotRef};
    let (p, opp, attack, source) = match *g.e(e) {
        Effect::Attack { p, opp, attack, source, .. } => (p, opp, attack, source),
        _ => return Ok(()),
    };
    let a = g.st.players[p as usize].active;
    let b = AtkBase { attack_effect: e, player: p, opponent: opp, attack, source, target: SlotRef::new(p as usize, a) };
    g.run_fx(Effect::HealTarget { b, damage })?;
    Ok(())
}
