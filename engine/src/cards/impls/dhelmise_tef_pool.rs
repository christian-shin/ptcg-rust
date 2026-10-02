//! Dhelmise (TEF 19): Spinning Attack — 30. Steel Anchor — 80+; 80 more
//! damage if you have any [M] Pokémon on your Bench.
//!
//! Twinleaf: each non-empty Bench slot with a Pokémon card gets a
//! CheckPokemonTypeEffect (`some` stops at the first [M]).
use crate::cards::prelude::*;
use crate::engine::game_effect::pokemon_types;

pub static IMPL: CardImpl = CardImpl { class: "DhelmiseTEFPool", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 1, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let mut has_metal = false;
        let bench: Vec<SlotId> = g.st.players[p].bench.iter().copied().collect();
        for b in bench {
            if g.st.slot(p, b).cards.is_empty() || g.st.slot_pokemon(p, b).is_none() {
                continue;
            }
            let target = SlotRef::new(p, b);
            let types = pokemon_types(g, target);
            let (t, _) = g.run_fx(Effect::CheckPokemonType { target, card_types: types })?;
            if matches!(t, Effect::CheckPokemonType { card_types, .. } if card_types.contains(&ct::METAL)) {
                has_metal = true;
                break;
            }
        }
        if has_metal {
            if let Effect::Attack { damage, .. } = g.e_mut(e) {
                *damage += 80;
            }
        }
    }
    Ok(())
}
