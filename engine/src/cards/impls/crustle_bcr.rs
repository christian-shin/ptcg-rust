//! Crustle (BCR): Sturdy — if this Pokémon has full HP and would be Knocked
//! Out by damage from an attack, it is not Knocked Out and its remaining HP
//! becomes 10. Stone Edge — 70+; flip a coin, if heads 20 more damage.
//!
//! SURVIVE_ON_TEN_IF_FULL_HP: on any PutDamageEffect whose target slot holds
//! this card (not necessarily on top), unless the ability is blocked for the
//! slot's owner, when the slot has no damage and `effect.damage >=` its HP
//! (CheckHpEffect by the owner), sets `surviveOnTenHPReason`. The core then
//! caps the damage at HP - 10 only when it went strictly over HP (exactly
//! lethal damage still Knocks Out).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Crustle@BCR", mask: mask(&[k::PUT_DAMAGE, k::ATTACK]), reduce, resume: None, coin: Some(coin), can_play: None };

/// SURVIVE_ON_TEN_IF_FULL_HP(store, state, effect, { source: me }) for a
/// Pokémon source (checkBlocked).
pub fn survive_on_ten_if_full_hp(g: &mut Game, me: CardId, e: EffId) -> R {
    let (t, damage) = match *g.e(e) {
        Effect::PutDamage { b, damage, .. } => (b.target, damage),
        _ => return Ok(()),
    };
    let owner = t.p as usize;
    if !g.st.slot(owner, t.s).cards.contains(me) {
        return Ok(());
    }
    if is_ability_blocked(g, owner, me, None) {
        return Ok(());
    }
    if g.st.slot(owner, t.s).damage != 0 {
        return Ok(());
    }
    let hp = crate::engine::check::check_hp(g, owner, t.s)?;
    if damage >= hp {
        if let Effect::PutDamage { survive_on_ten_hp, .. } = g.e_mut(e) {
            *survive_on_ten_hp = true;
        }
    }
    Ok(())
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    survive_on_ten_if_full_hp(g, me, e)?;
    if was_attack_used(g, e, 0, me) {
        super::riolu_pre::flip_more_damage(g, me, e, 20)?;
    }
    Ok(())
}

fn coin(g: &mut Game, _me: CardId, f: CardFrame, heads: bool) -> R {
    super::riolu_pre::coin_more_damage(g, f, heads)
}
