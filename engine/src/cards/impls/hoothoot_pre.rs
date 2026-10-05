//! Hoothoot (PRE): Insomnia — this Pokémon can't be Asleep. Tackle — 20.
//!
//! Twinleaf has several `Hoothoot` classes; this port is bound to PRE.
//! Fixed in phase 4b (R4): on an AddSpecialConditionsEffect or
//! AddSpecialConditionsPowerEffect that includes Asleep and whose target's top
//! Pokémon is this card, Asleep is removed from the effect (the effect is
//! prevented when it was the only condition) unless the Ability is blocked
//! for the owner. It used to prevent every such effect, whatever its target,
//! from any zone and without an Ability-lock check.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "Hoothoot@PRE",
    mask: mask(&[k::ADD_SPECIAL_CONDITIONS, k::ADD_SPECIAL_CONDITIONS_POWER]),
    reduce,
    resume: None,
    coin: None,
    can_play: None,
};

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let (target, conditions) = match *g.e(e) {
        Effect::AddSpecialConditions { b, conditions, .. } => (b.target, conditions),
        Effect::AddSpecialConditionsPower { target, conditions, .. } => (target, conditions),
        _ => return Ok(()),
    };
    let asleep = SpecialCondition::Asleep as u8;
    if conditions.contains(&asleep) && g.st.slot_pokemon(target.p as usize, target.s) == Some(me) {
        let owner = target.p as usize;
        if !is_ability_blocked(g, owner, me, None) {
            let mut remaining = conditions;
            remaining.retain(|c| *c != asleep);
            if remaining.is_empty() {
                g.set_prevent(e, true);
            } else {
                match g.e_mut(e) {
                    Effect::AddSpecialConditions { conditions, .. } | Effect::AddSpecialConditionsPower { conditions, .. } => *conditions = remaining,
                    _ => {}
                }
            }
        }
    }
    Ok(())
}
