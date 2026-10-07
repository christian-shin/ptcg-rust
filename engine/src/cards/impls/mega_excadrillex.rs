//! Mega Excadrill ex (PBL / M5): Undermine — 90, discard the top 2 cards of
//! your opponent's deck. Maximum Drilling — 200+, 130 more if this Pokémon
//! has at least 2 extra Energy attached.
//!
//! Twinleaf: the mill is a plain MOVE_CARDS (not an attack effect). The extra
//! Energy count sums `provides` lengths of the player's Active provided
//! energy minus the checked cost length.
use crate::cards::prelude::*;
use crate::effects::Cost;

pub static IMPL: CardImpl = CardImpl { class: "MegaExcadrillex", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let opp = match *g.e(e) {
            Effect::Attack { opp, .. } => opp,
            _ => return Ok(()),
        };
        move_count_from(g, ListRef::Deck(opp), ListRef::Discard(opp), 2, me)?;
    }

    if was_attack_used(g, e, 1, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p,
            _ => return Ok(()),
        };
        let attack = my_attack(g, me, 1);
        let mut cost: Cost = SVec::new();
        for &c in crate::engine::attack::attack_def(g, attack).cost {
            cost.push(c);
        }
        let (ce, _) = g.run_fx(Effect::CheckAttackCost { p, attack, cost, set_cost: None, ignore_colorless: false, reduction: 0, any_reduction: false })?;
        let cost_len = match ce {
            Effect::CheckAttackCost { cost, .. } => cost.len() as i32,
            _ => 0,
        };
        let pu = p as usize;
        let src = SlotRef::new(pu, g.st.players[pu].active);
        let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p, source: src, energy_map: SVec::new() })?;
        let total: i32 = match pe {
            Effect::CheckProvidedEnergy { energy_map, .. } => energy_map.iter().map(|m| m.provides.len() as i32).sum(),
            _ => 0,
        };
        if total - cost_len >= 2 {
            if let Effect::Attack { damage, .. } = g.e_mut(e) {
                *damage += 130;
            }
        }
    }
    Ok(())
}
