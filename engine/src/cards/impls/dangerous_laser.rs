//! Dangerous Laser (SFA, ACE SPEC): your opponent's Active Pokémon is now
//! Burned and Confused.
//!
//! Twinleaf: unless a TrainerTargetEffect on the Active is blocked, the
//! conditions are added directly (`addSpecialCondition`, no effect).
use crate::cards::prelude::*;
use crate::engine::phase::{add_condition, would_change_special_conditions};

pub static IMPL: CardImpl = CardImpl { class: "DangerousLaser", mask: mask(&[k::TRAINER]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    let o = 1 - p;
    let target = SlotRef::new(o, g.st.players[o].active);
    // Can't be played when the Active Pokémon is already Burned and Confused (Advanced Rulebook B-01, ruling 962).
    if !would_change_special_conditions(g.st.slot(o, target.s), &[SpecialCondition::Burned, SpecialCondition::Confused]) {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    let (t, prevented) = g.run_fx(Effect::TrainerTarget { p: p as u8, card: me, target: Some(target) })?;
    let blocked = prevented || matches!(t, Effect::TrainerTarget { target: None, .. });
    if !blocked {
        let slot = &mut g.st.players[o].slots[target.s as usize];
        add_condition(slot, SpecialCondition::Burned);
        add_condition(slot, SpecialCondition::Confused);
    }
    Ok(())
}
