//! Kadabra (M1S): Psychic Draw — when you play this Pokémon from your hand to
//! evolve, you may draw 2 cards. Super Psy Bolt — 30.
//!
//! Twinleaf: fires on any EvolveEffect for this card (Rare Candy included).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Kadabra", mask: mask(&[k::EVOLVE]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    psychic_draw_reduce(g, me, e)
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    psychic_draw_resume(g, f, results, 2)
}

/// `JUST_EVOLVED(effect, this)` → CONFIRMATION_PROMPT unless the ability is blocked.
pub fn psychic_draw_reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match *g.e(e) {
        Effect::Evolve { p, card, .. } if card == me => p as usize,
        _ => return Ok(()),
    };
    if is_ability_blocked(g, p, me, None) {
        return Ok(());
    }
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    confirmation_prompt(g, p, "WANT_TO_USE_ABILITY", Cont::Card { card: me, frame: f });
    Ok(())
}

pub fn psychic_draw_resume(g: &mut Game, f: CardFrame, results: &[Res], n: usize) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    if results.first().map(|r| r.as_bool()).unwrap_or(false) {
        draw_cards(g, f.a[0] as usize, n)?;
    }
    Ok(())
}
