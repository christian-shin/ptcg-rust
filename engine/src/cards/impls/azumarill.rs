//! Azumarill (SSP): Glistening Bubbles - with a Tera Pokémon in play,
//! Double-Edge costs less. Double-Edge - 230; 50 damage to itself.
//!
//! Twinleaf: on CheckAttackCostEffect for attack 0, when any of the player's
//! in-play Pokémon has the Tera tag and an Ability probe passes, the first
//! [P] of the cost is removed together with the two entries after it
//! (`splice(index, 3)`), so [P][P][P][P] becomes [P] (fixed in phase 4b: the
//! old loop repeated the splice and left an empty cost). The self-damage
//! is a DealDamageEffect targeting the player's Active (not necessarily this
//! Pokémon).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Azumarill", mask: mask(&[k::CHECK_ATTACK_COST, k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::CheckAttackCost { p, attack, .. } = *g.e(e) {
        if attack != my_attack(g, me, 0) {
            return Ok(());
        }
        let p = p as usize;
        let tera = for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().any(|(_, c, _)| g.st.cdef(*c).has_tag(tag::POKEMON_TERA));
        if tera {
            if is_ability_blocked(g, p, me, None) {
                return Ok(());
            }
            if let Effect::CheckAttackCost { cost, .. } = g.e_mut(e) {
                let mut v: Vec<CardType> = cost.iter().copied().collect();
                if !v.contains(&ct::PSYCHIC) {
                    return Ok(());
                }
                if let Some(i) = v.iter().position(|t| *t == ct::PSYCHIC) {
                    let end = (i + 3).min(v.len());
                    v.drain(i..end);
                }
                cost.clear();
                for t in v {
                    cost.push(t);
                }
            }
        }
        return Ok(());
    }
    if was_attack_used(g, e, 0, me) {
        let (p, opp, attack, source) = match *g.e(e) {
            Effect::Attack { p, opp, attack, source, .. } => (p, opp, attack, source),
            _ => return Ok(()),
        };
        let pu = p as usize;
        let target = SlotRef::new(pu, g.st.players[pu].active);
        let b = AtkBase { attack_effect: e, player: p, opponent: opp, attack, source, target };
        g.run_fx(Effect::DealDamage { b, damage: 50 })?;
    }
    Ok(())
}
