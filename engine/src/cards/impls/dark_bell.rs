//! Dark Bell (M5 / PBL 75): both Active non-[D] Pokémon are now Confused.
//!
//! Twinleaf: the player's own Active first, then the opponent's Active unless
//! a TrainerTargetEffect on it is blocked; each uses a CheckPokemonTypeEffect
//! and ADD_CONFUSION_TO_PLAYER_ACTIVE (AddSpecialConditionsPowerEffect, which
//! also resets poison/burn/sleep values to the defaults). `canPlay` is UI only.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "DarkBell", mask: mask(&[k::TRAINER]), reduce, resume: None, coin: None, can_play: None };

fn confuse_target(g: &mut Game, me: CardId, q: usize) -> R {
    let a = g.st.players[q].active;
    if g.st.slot(q, a).cards.is_empty() {
        return Ok(());
    }
    let target = SlotRef::new(q, a);
    let types = crate::engine::game_effect::pokemon_types(g, target);
    let (t, _) = g.run_fx(Effect::CheckPokemonType { target, card_types: types })?;
    let dark = matches!(t, Effect::CheckPokemonType { card_types, .. } if card_types.contains(&ct::DARK));
    if !dark {
        add_special_conditions_to_player_active(g, q, me, &[SpecialCondition::Confused])?;
    }
    Ok(())
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    let o = 1 - p;
    confuse_target(g, me, p)?;
    let target = SlotRef::new(o, g.st.players[o].active);
    let (t, prevented) = g.run_fx(Effect::TrainerTarget { p: p as u8, card: me, target: Some(target) })?;
    let blocked = prevented || matches!(t, Effect::TrainerTarget { target: None, .. });
    if !blocked {
        confuse_target(g, me, o)?;
    }
    Ok(())
}
