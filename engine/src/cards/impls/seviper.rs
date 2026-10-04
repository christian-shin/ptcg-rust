//! Seviper (M2 / PFL 62): Excite Power — if you have a [D] Mega Evolution
//! Pokémon ex in play, this Pokémon's attacks do 120 more damage. Jet Black
//! Fang — 120.
//!
//! Fixed (phase 4b, W4): Twinleaf accepted any [D] Pokémon ex; the Mega ex tag
//! is now required.
//!
//! Twinleaf: any AttackEffect whose source slot holds this card gets +120
//! when its damage is above 0 (checked after the ability-lock probe).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Seviper", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let (p, source) = match *g.e(e) {
        Effect::Attack { p, source, .. } => (p as usize, source),
        _ => return Ok(()),
    };
    if !g.st.slot(source.p as usize, source.s).cards.contains(me) {
        return Ok(());
    }
    if is_ability_blocked(g, p, me, None) {
        return Ok(());
    }
    let has = for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().any(|(_, c, _)| {
        let d = g.st.cdef(*c);
        d.card_type.contains(&ct::DARK) && d.has_tag(tag::POKEMON_EX_LOWER) && d.has_tag(tag::POKEMON_SV_MEGA)
    });
    if let Effect::Attack { damage, .. } = g.e_mut(e) {
        if has && *damage > 0 {
            *damage += 120;
        }
    }
    Ok(())
}
