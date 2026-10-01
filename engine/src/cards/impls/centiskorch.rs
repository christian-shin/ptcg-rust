//! Centiskorch (SSP): Billowing Heat Wave — 130; also 30 damage to each of
//! your Benched Pokémon (PutDamageEffect, no Weakness/Resistance). Heat Blast — 80.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Centiskorch", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let active = g.st.players[p].active;
        for (s, _, _) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
            if s != active {
                put_damage(g, e, 30, SlotRef::new(p, s))?;
            }
        }
    }
    Ok(())
}
