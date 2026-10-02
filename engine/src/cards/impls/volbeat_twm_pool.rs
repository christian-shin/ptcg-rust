//! Volbeat (TWM 9): Quick Sign (usable on the first turn when going first) —
//! search your deck for up to 2 Basic Pokémon and put them onto your Bench,
//! then shuffle. Coordinated Strike — 20+; 60 more damage if Illumise is on
//! your Bench.
//!
//! Twinleaf: Quick Sign does nothing (no shuffle) with an empty deck or a full
//! Bench; else SEARCH_YOUR_DECK_FOR_POKEMON_AND_PUT_ONTO_BENCH({ stage: BASIC },
//! { min: 0, max: min(2, open slots) }) (cancellable).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "VolbeatTWMPool", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let open = empty_bench_slots(g, p).len();
        if g.st.players[p].deck.is_empty() || open == 0 {
            return Ok(());
        }
        let filter = Filter { stage: Some(Stage::Basic as u8), ..Filter::none() };
        return search_deck_for_pokemon_to_bench(g, p, filter, ChooseCardsOpts::new(0, open.min(2) as u8, true));
    }
    if was_attack_used(g, e, 1, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let bench: Vec<SlotId> = g.st.players[p].bench.iter().copied().collect();
        let has = bench.iter().any(|b| g.st.slot_pokemon(p, *b).map(|c| g.st.cdef(c).name == "Illumise").unwrap_or(false));
        if has {
            if let Effect::Attack { damage, .. } = g.e_mut(e) {
                *damage += 60;
            }
        }
    }
    Ok(())
}
