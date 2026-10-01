//! Full Metal Lab (TEF, stadium): [M] Pokémon (both yours and your
//! opponent's) take 30 less damage from attacks from the opponent's Pokémon
//! (after applying Weakness and Resistance).
//!
//! Twinleaf: every PutDamageEffect (any source, including a player's own
//! attack on its own Pokémon) on a [M] Pokémon (CheckPokemonTypeEffect) is
//! reduced by 30 (floored at 0) unless the stadium effect is blocked for the
//! target's owner. The stadium can't be used.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "FullMetalLab",
    mask: mask(&[k::PUT_DAMAGE, k::USE_STADIUM]),
    reduce,
    resume: None,
    coin: None,
    can_play: None,
};

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if g.st.stadium_card() != Some(me) {
        return Ok(());
    }
    match *g.e(e) {
        Effect::PutDamage { b, .. } => {
            let t = b.target;
            if is_stadium_effect_blocked(g, t.p as usize, t, me) {
                return Ok(());
            }
            let types = crate::engine::game_effect::pokemon_types(g, t);
            let (ct_e, _) = g.run_fx(Effect::CheckPokemonType { target: t, card_types: types })?;
            if matches!(ct_e, Effect::CheckPokemonType { card_types, .. } if card_types.contains(&ct::METAL)) {
                if let Effect::PutDamage { damage, .. } = g.e_mut(e) {
                    *damage = (*damage - 30).max(0);
                }
            }
            Ok(())
        }
        Effect::UseStadium { .. } => bail!("CANNOT_USE_STADIUM"),
        _ => Ok(()),
    }
}
