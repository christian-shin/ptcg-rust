//! Decidueye ex (POR / M3): Sniper's Eye — if your opponent has exactly 4
//! cards in their hand, ignore all [C] Energy in the costs of attacks used by
//! this Pokémon. Crushing Arrow — 240; discard an Energy from your
//! opponent's Active Pokémon.
//!
//! Fixed (phase 4b, R3): Sniper's Eye now checks Ability locks (IS_ABILITY_BLOCKED
//! for the owner before reading the opponent's hand), is not an activated
//! Ability (`use_when_in_play` false in the card DB), and Crushing Arrow's
//! discard is DISCARD_AN_ENERGY_FROM_OPPONENTS_ACTIVE_POKEMON on a fresh
//! AttackEffect (a DiscardCardsEffect, so Mist Energy can prevent it) instead
//! of a bare MOVE_CARDS.
//!
//! Twinleaf: Sniper's Eye strips every [C] from any CheckAttackCostEffect
//! while this card is the player's Active Pokémon (R7F-11: and sets
//! `ignoreColorless`, so a [C] added later by another effect is ignored too). Crushing Arrow
//! (AFTER_ATTACK) prompts (non-cancellable ChooseCardsPrompt, Energy, over the
//! opponent's Active) only when the Active holds an Energy card.
use crate::cards::prelude::*;
use super::trubbish::{discard_an_energy_from_opponents_active, discard_chosen};

pub static IMPL: CardImpl = CardImpl { class: "Decidueyeex@POR", mask: mask(&[k::CHECK_ATTACK_COST, k::AFTER_ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::CheckAttackCost { p, .. } = *g.e(e) {
        let p = p as usize;
        if g.st.slot_pokemon(p, g.st.players[p].active) == Some(me) {
            if is_ability_blocked(g, p, me, None) {
                return Ok(());
            }
            if g.st.players[1 - p].hand.len() == 4 {
                if let Effect::CheckAttackCost { cost, ignore_colorless, .. } = g.e_mut(e) {
                    cost.retain(|t| *t != ct::COLORLESS);
                    // ...also the [C] that other effects add (R7F-11, rulings 252, 1552).
                    *ignore_colorless = true;
                }
            }
        }
        return Ok(());
    }

    if after_attack_used(g, e, 0, me) {
        let (p, opp, attack) = match *g.e(e) {
            Effect::AfterAttack { p, opp, attack } => (p, opp, attack),
            _ => return Ok(()),
        };
        // `new AttackEffect(player, opponent, this.attacks[0])`
        let source = SlotRef::new(p as usize, g.st.players[p as usize].active);
        let atk = g.new_fx(Effect::Attack { p, opp, attack, damage: 0, ignore_weakness: false, ignore_resistance: false, source, barrage_used: false });
        return discard_an_energy_from_opponents_active(g, me, atk, 1);
    }
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage == 1 {
        return discard_chosen(g, f, results);
    }
    Ok(())
}
