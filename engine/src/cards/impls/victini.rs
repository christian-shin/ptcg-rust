//! Victini (SSP): Victory Cheer — attacks used by your Evolution [R] Pokémon
//! do 10 more damage to your opponent's Active Pokémon. Flare — 30.
//!
//! Twinleaf: on every DealDamageEffect (either player's): bail out if the
//! ability is blocked for the effect's player; count Victini copies among
//! that player's Pokémon; a CheckPokemonTypeEffect on THAT player's Active
//! must contain [R] (not the attacker's slot); the target must be the
//! opponent's Active; the attacking Pokémon's `evolvesFrom !== ''` (an
//! empty source slot counts as an Evolution); then damage += 10 * count.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Victini", mask: mask(&[k::DEAL_DAMAGE]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::DealDamage { b, .. } = *g.e(e) {
        let p = b.player as usize;
        let o = 1 - p;
        if is_ability_blocked(g, p, me, None) {
            return Ok(());
        }
        let mut count = 0;
        let in_play = {
            let pl = &g.st.players[p];
            pl.in_play().iter().any(|s| pl.slots[*s as usize].cards.contains(me))
        };
        if in_play {
            let pl = &g.st.players[p];
            for s in pl.in_play().iter() {
                if g.st.slot_pokemon(p, *s).is_some() && pl.slots[*s as usize].cards.contains(me) {
                    count += 1;
                }
            }
        }
        let active = SlotRef::new(p, g.st.players[p].active);
        let (te, _) = g.run_fx(Effect::CheckPokemonType { target: active, card_types: crate::engine::game_effect::pokemon_types(g, active) })?;
        let is_fire = match te {
            Effect::CheckPokemonType { card_types, .. } => card_types.contains(&ct::FIRE),
            _ => false,
        };
        if is_fire && b.target.p as usize == o && b.target.s == g.st.players[o].active {
            let evo = match g.st.slot_pokemon(b.source.p as usize, b.source.s) {
                Some(c) => !g.st.cdef(c).evolves_from.is_empty(),
                None => true,
            };
            if evo {
                if let Effect::DealDamage { damage, .. } = g.e_mut(e) {
                    *damage += 10 * count;
                }
            }
        }
    }
    Ok(())
}
