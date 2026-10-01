//! Alolan Exeggutor (30C 2): Scale Up — if this Pokémon has 6 or more [G]
//! Energy attached, it gets +250 HP. Mega Drain — 150; heal 50 damage from
//! this Pokémon.
//!
//! Twinleaf: on CheckHpEffect for the slot whose top Pokémon is this card,
//! unless the Ability is blocked, counts GRASS / ANY `provides` entries of a
//! CheckProvidedEnergyEffect on that slot.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "AlolanExeggutor", mask: mask(&[k::CHECK_HP, k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::CheckHp { target, card, .. } = *g.e(e) {
        let (tp, ts) = (target.p as usize, target.s);
        if !g.st.slot(tp, ts).cards.contains(me) || g.st.slot_pokemon(tp, ts) != Some(me) {
            return Ok(());
        }
        if is_ability_blocked(g, tp, me, None) {
            return Ok(());
        }
        let n = super::mega_meganiumex::grass_energy_count(g, tp, ts)?;
        if n >= 6 && card.is_some() {
            g.st.players[tp].slots[ts as usize].hp_bonus += 250;
        }
        return Ok(());
    }
    if was_attack_used(g, e, 0, me) {
        super::applin_dri::heal_this_pokemon(g, e, 50)?;
    }
    Ok(())
}
