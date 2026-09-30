//! Zoroark (SV11W / WHT 62): Mind Jack — 30 damage for each of your
//! opponent's Benched Pokémon. Foul Play — choose 1 of your opponent's Active
//! Pokémon's attacks and use it as this attack (COPY_OPPONENT_ACTIVE_ATTACK,
//! see `copy_attack.rs`).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Zoroark", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        if let Effect::Attack { opp, .. } = *g.e(e) {
            let o = opp as usize;
            let a = g.st.players[o].active;
            let benched = for_each_pokemon(g, o, PlayerType::BottomPlayer).iter().filter(|(s, _, _)| *s != a).count() as i32;
            if let Effect::Attack { damage, .. } = g.e_mut(e) {
                *damage = benched * 30;
            }
        }
    }
    if was_attack_used(g, e, 1, me) {
        return crate::copy_attack::copy_opponent_active_attack(g, e);
    }
    Ok(())
}
