//! Slowpoke (MEP 86): Dopey Face — this Pokémon can't be Confused. Super Psy
//! Bolt — 50.
//!
//! Twinleaf: on an AddSpecialConditionsEffect or AddSpecialConditionsPowerEffect
//! whose conditions include Confused and whose target's top Pokémon is this
//! card, Confused is removed from the effect (the effect is prevented when it
//! was the only condition) unless the ability is blocked for the owner. Fixed
//! in phase 4b (R4): the whole effect used to be prevented, so a Burned and
//! Confused attack applied neither.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "SlowpokeMEP86Pool",
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
    if conditions.contains(&(SpecialCondition::Confused as u8)) && g.st.slot_pokemon(target.p as usize, target.s) == Some(me) {
        let owner = target.p as usize;
        if !is_ability_blocked(g, owner, me, None) {
            let confused = SpecialCondition::Confused as u8;
            let mut remaining = conditions;
            remaining.retain(|c| *c != confused);
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
