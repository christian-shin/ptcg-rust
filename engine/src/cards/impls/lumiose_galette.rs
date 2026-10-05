//! Lumiose Galette (POR): heal 20 damage and remove a Special Condition from
//! your Active Pokémon.
//!
//! Twinleaf: playable only if the Active has damage or a Special Condition;
//! a HealEffect for 20, then a Special Condition is removed directly.
//! Fixed (phase 4b, R3): the player chooses which one: the only condition is
//! removed without a prompt; with several, a SelectOptionPrompt lists all five
//! (Paralyzed, Confused, Asleep, Poisoned, Burned; absent ones disabled, not
//! cancellable) and the chosen one is removed (it used to remove the first
//! one listed).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "LumioseGalette", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

const CONDITIONS: &[&str] = &["Paralyzed", "Confused", "Asleep", "Poisoned", "Burned"];

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
    let conditions: Vec<u8> = g.st.slot(p, a).special_conditions.as_slice().to_vec();
    if conditions.len() == 1 {
        crate::engine::phase::remove_condition(g, p, a, SpecialCondition::from_u8(conditions[0]));
    } else if conditions.len() > 1 {
        let mut disabled = 0u16;
        for i in 0..CONDITIONS.len() {
            if !conditions.contains(&(i as u8)) {
                disabled |= 1 << i;
            }
        }
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        f.a[1] = conditions[0] as i32;
        let id = g.player_id(p);
        g.prompt(
            id,
            "CHOOSE_OPTION",
            PromptKind::SelectOption { values: CONDITIONS, allow_cancel: false, default_value: conditions[0] as i32, disabled: Some(disabled) },
            Cont::Card { card: me, frame: f },
        );
    }
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let choice = match results.first().copied().unwrap_or(Res::Null) {
        Res::Int(i) => i,
        // A missing answer (the oracle's bot answers null) stands in with the default (first listed).
        Res::Null => f.a[1],
        _ => bail!("INVALID_PROMPT_RESULT"),
    };
    let a = g.st.players[p].active;
    crate::engine::phase::remove_condition(g, p, a, SpecialCondition::from_u8(choice as u8));
    Ok(())
}
