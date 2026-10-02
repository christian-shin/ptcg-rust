//! Mega Froslass ex (M2a): Resentful Refrain — 50x per card in your
//! opponent's hand (`effect.damage = 50 * handCount`). Absolute Snow — 150;
//! your opponent's Active Pokémon is now Asleep, via AFTER_ATTACK and an
//! AddSpecialConditionsPowerEffect (ADD_SLEEP_TO_PLAYER_ACTIVE) sourced from
//! this card.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "MegaFroslassex", mask: mask(&[k::ATTACK, k::AFTER_ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let o = match *g.e(e) {
            Effect::Attack { opp, .. } => opp as usize,
            _ => return Ok(()),
        };
        let n = g.st.players[o].hand.len() as i32;
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage = 50 * n;
        }
    }
    if after_attack_used(g, e, 1, me) {
        let o = match *g.e(e) {
            Effect::AfterAttack { opp, .. } => opp as usize,
            _ => return Ok(()),
        };
        add_special_conditions_to_player_active(g, o, me, &[SpecialCondition::Asleep])?;
    }
    Ok(())
}
