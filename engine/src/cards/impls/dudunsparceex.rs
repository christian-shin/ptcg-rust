//! Dudunsparce ex (JTG): Tenacious Tail — 60× your opponent's Pokémon ex in
//! play. Destructive Drill — 150, not affected by effects on your opponent's
//! Active Pokémon (own ApplyWeaknessEffect, direct damage, AfterDamageEffect).
use super::mega_lopunnyex::shred;
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Dudunsparceex", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        if let Effect::Attack { p, .. } = *g.e(e) {
            let o = 1 - p as usize;
            let n = for_each_pokemon(g, o, PlayerType::TopPlayer).iter().filter(|(_, c, _)| g.st.cdef(*c).has_tag(tag::POKEMON_EX_LOWER)).count() as i32;
            if let Effect::Attack { damage, .. } = g.e_mut(e) {
                *damage = n * 60;
            }
        }
    }
    if was_attack_used(g, e, 1, me) {
        shred(g, e, 150)?;
    }
    Ok(())
}
