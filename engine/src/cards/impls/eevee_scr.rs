//! Eevee (SCR): Call for Family — search your deck for a Basic Pokémon and
//! put it onto your Bench, then shuffle. Gnaw — 20.
//!
//! Twinleaf has several `Eevee` classes; this port is bound to SCR. With no
//! empty Bench slot the attack does nothing (no prompt, no shuffle); the
//! prompt is cancellable.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Eevee@SCR", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        if empty_bench_slots(g, p).is_empty() {
            return Ok(());
        }
        super::chatot::bench_search(g, me, p, 1, true);
    }
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    super::chatot::bench_search_resume(g, f, results)
}
