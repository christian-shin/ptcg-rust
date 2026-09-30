//! Hoothoot (PRE): Insomnia — this Pokémon can't be Asleep. Tackle — 20.
//!
//! Twinleaf has several `Hoothoot` classes; this port is bound to PRE.
//! Twinleaf quirk kept: every AddSpecialConditionsEffect that includes
//! Asleep is prevented, whatever its target, from any zone (deck, hand,
//! discard too) and without an ability-lock check.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Hoothoot@PRE", mask: mask(&[k::ADD_SPECIAL_CONDITIONS]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, _me: CardId, e: EffId) -> R {
    if let Effect::AddSpecialConditions { conditions, .. } = g.e(e) {
        if conditions.contains(&(SpecialCondition::Asleep as u8)) {
            g.set_prevent(e, true);
        }
    }
    Ok(())
}
