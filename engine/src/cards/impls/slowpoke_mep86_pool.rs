//! Slowpoke (MEP 86): Dopey Face — this Pokémon can't be Confused. Super Psy
//! Bolt — 50.
//!
//! Twinleaf: on an AddSpecialConditionsEffect or AddSpecialConditionsPowerEffect
//! whose conditions include Confused and whose target's top Pokémon is this
//! card, the effect is prevented unless the ability is blocked for the owner.
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
            g.set_prevent(e, true);
        }
    }
    Ok(())
}
