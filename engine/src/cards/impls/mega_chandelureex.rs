//! Mega Chandelure ex (PBL / M5): Binding Flame - your opponent's Active
//! Pokémon's Retreat Cost is [C] more. Phantom Maze - 130+; 50 more damage for
//! each [C] in your opponent's Active Pokémon's Retreat Cost.
//!
//! Twinleaf: on any CheckRetreatCostEffect the ability owner is the retreating
//! player's opponent (this card must be the top card of one of their
//! Pokémon, and not ability-blocked); Phantom Maze sets (not adds) the damage
//! from a fresh CheckRetreatCostEffect for the opponent.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "MegaChandelureex", mask: mask(&[k::CHECK_RETREAT_COST, k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::CheckRetreatCost { p, .. } = *g.e(e) {
        let owner_side = 1 - p as usize;
        let in_play = for_each_pokemon(g, owner_side, PlayerType::BottomPlayer).iter().any(|x| x.1 == me);
        if !in_play {
            return Ok(());
        }
        if is_ability_blocked(g, owner_side, me, None) {
            return Ok(());
        }
        if let Effect::CheckRetreatCost { cost, .. } = g.e_mut(e) {
            cost.push(ct::COLORLESS);
        }
        return Ok(());
    }
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let o = 1 - p;
        let cost = crate::engine::retreat::check_retreat_cost_base(g, o);
        let (re, _) = g.run_fx(Effect::CheckRetreatCost { p: o as u8, cost, no_cost: false })?;
        let colorless = match re {
            Effect::CheckRetreatCost { cost, .. } => cost.iter().filter(|t| **t == ct::COLORLESS).count() as i32,
            _ => 0,
        };
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage = 130 + 50 * colorless;
        }
    }
    Ok(())
}
