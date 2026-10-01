//! Perilous Jungle (TEF): during Pokémon Checkup, put 2 more damage counters
//! on each Poisoned non-[D] Pokémon.
//!
//! Twinleaf: for each BetweenTurnsEffect only the effect player's Active
//! Pokémon is checked (block probe, then CheckPokemonTypeEffect) and the
//! bonus is added to the effect's shared `poisonDamage`.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "PerilousJungle", mask: mask(&[k::BETWEEN_TURNS, k::USE_STADIUM]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    match *g.e(e) {
        Effect::BetweenTurns { p, .. } if g.st.stadium_card() == Some(me) => {
            let p = p as usize;
            let active = SlotRef::new(p, g.st.players[p].active);
            let types = crate::engine::game_effect::pokemon_types(g, active);
            if is_stadium_effect_blocked(g, p, active, me) {
                return Ok(());
            }
            let (te, _) = g.run_fx(Effect::CheckPokemonType { target: active, card_types: types })?;
            let dark = match te {
                Effect::CheckPokemonType { card_types, .. } => card_types.contains(&ct::DARK),
                _ => false,
            };
            if dark {
                return Ok(());
            }
            if let Effect::BetweenTurns { poison_damage, .. } = g.e_mut(e) {
                *poison_damage += 20;
            }
            Ok(())
        }
        Effect::UseStadium { .. } if g.st.stadium_card() == Some(me) => bail!("CANNOT_USE_STADIUM"),
        _ => Ok(()),
    }
}
