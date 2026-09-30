//! Lilligant (JTG): Sunny Day — attacks used by your [G] and [R] Pokémon do
//! 20 more damage to your opponent's Active Pokémon. Spinning Attack — 60.
//!
//! Twinleaf: reacts to every DealDamageEffect. The lock probe runs for the
//! attacking player first (even when this Lilligant is on the other side),
//! then a CheckPokemonTypeEffect on the attacker's Active is always reduced;
//! +20 when that Active is [G]/[R], the target is the opponent's Active and
//! this Lilligant is in play on the attacker's side.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Lilligant@JTG", mask: mask(&[k::DEAL_DAMAGE]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let b = match *g.e(e) {
        Effect::DealDamage { b, .. } => b,
        _ => return Ok(()),
    };
    let p = b.player as usize;
    let o = 1 - p;
    if is_ability_blocked(g, p, me, None) {
        return Ok(());
    }
    let in_play = g.st.players[p].in_play().iter().any(|s| g.st.slot(p, *s).cards.contains(me));
    let mut n = 0;
    if in_play {
        for (s, _, _) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter() {
            if g.st.slot(p, *s).cards.contains(me) {
                n += 1;
            }
        }
    }
    let target = SlotRef::new(p, g.st.players[p].active);
    let types = crate::engine::game_effect::pokemon_types(g, target);
    let (t, _) = g.run_fx(Effect::CheckPokemonType { target, card_types: types })?;
    let ok = matches!(t, Effect::CheckPokemonType { card_types, .. } if card_types.contains(&ct::GRASS) || card_types.contains(&ct::FIRE));
    if ok && b.target.p as usize == o && b.target.s == g.st.players[o].active {
        if let Effect::DealDamage { damage, .. } = g.e_mut(e) {
            *damage += 20 * n;
        }
    }
    Ok(())
}
