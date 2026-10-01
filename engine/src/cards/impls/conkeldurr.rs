//! Conkeldurr (TWM 105): Tantrum — 80; this Pokémon is now Confused. Gutsy
//! Swing — 250; if this Pokémon is affected by a Special Condition, ignore
//! all Energy in this attack's cost.
//!
//! Twinleaf: Tantrum reduces an AddSpecialConditionsEffect on
//! `player.active`; Gutsy Swing empties the CheckAttackCostEffect cost when
//! this card is the player's Active and it has any Special Condition.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Conkeldurr", mask: mask(&[k::ATTACK, k::CHECK_ATTACK_COST]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let (p, opp, attack, source) = match *g.e(e) {
            Effect::Attack { p, opp, attack, source, .. } => (p, opp, attack, source),
            _ => return Ok(()),
        };
        let a = g.st.players[p as usize].active;
        let b = AtkBase { attack_effect: e, player: p, opponent: opp, attack, source, target: SlotRef::new(p as usize, a) };
        let mut cs = SVec::new();
        cs.push(SpecialCondition::Confused as u8);
        g.run_fx(Effect::AddSpecialConditions { b, conditions: cs, poison_damage: None, burn_damage: None, confusion_damage: None })?;
        return Ok(());
    }
    if let Effect::CheckAttackCost { p, attack, .. } = *g.e(e) {
        if attack != my_attack(g, me, 1) {
            return Ok(());
        }
        let p = p as usize;
        let a = g.st.players[p].active;
        if g.st.slot_pokemon(p, a) != Some(me) {
            return Ok(());
        }
        if !g.st.slot(p, a).special_conditions.is_empty() {
            if let Effect::CheckAttackCost { cost, .. } = g.e_mut(e) {
                cost.clear();
            }
        }
    }
    Ok(())
}
