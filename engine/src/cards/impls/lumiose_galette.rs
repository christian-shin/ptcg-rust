//! Lumiose Galette (POR): heal 20 damage and remove a Special Condition from
//! your Active Pokémon.
//!
//! Twinleaf: playable only if the Active has damage or a Special Condition;
//! a HealEffect for 20, then the first listed Special Condition is removed
//! directly.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "LumioseGalette", mask: mask(&[k::TRAINER]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    let a = g.st.players[p].active;
    let slot = g.st.slot(p, a);
    if slot.damage <= 0 && slot.special_conditions.is_empty() {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    g.set_prevent(e, true);
    g.run_fx(Effect::Heal { p: p as u8, target: SlotRef::new(p, a), damage: 20 })?;
    let a = g.st.players[p].active;
    if let Some(&sc) = g.st.slot(p, a).special_conditions.as_slice().first() {
        crate::engine::phase::remove_condition(g, p, a, SpecialCondition::from_u8(sc));
    }
    Ok(())
}
