//! Bronzor (TEF): Mirror Attack — 10+, 30 more if your opponent's Active
//! Pokémon is a [P] Pokémon (via CheckPokemonTypeEffect).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Bronzor", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let opp = match *g.e(e) {
            Effect::Attack { opp, .. } => opp as usize,
            _ => return Ok(()),
        };
        let target = SlotRef::new(opp, g.st.players[opp].active);
        let types = crate::engine::game_effect::pokemon_types(g, target);
        let (t, _) = g.run_fx(Effect::CheckPokemonType { target, card_types: types })?;
        if matches!(t, Effect::CheckPokemonType { card_types, .. } if card_types.contains(&ct::PSYCHIC)) {
            if let Effect::Attack { damage, .. } = g.e_mut(e) {
                *damage += 30;
            }
        }
    }
    Ok(())
}
