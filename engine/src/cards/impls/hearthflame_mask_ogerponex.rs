//! Hearthflame Mask Ogerpon ex (TWM / PRE, Tera): Wrathful Hearth - 20 damage
//! for each damage counter on this Pokémon. Dynamic Blaze - 140+; if the
//! opposing Active isn't a Basic, 140 more and discard all Energy from this
//! Pokémon. Tera: no attack damage while on the Bench.
//!
//! Twinleaf: Wrathful Hearth is `damage = player.active.damage * 2`; Dynamic
//! Blaze adds 140 when `!opponent.active.isStage(BASIC)` (true as well for
//! an empty Active slot), then discards every card of the Active's
//! CheckProvidedEnergyEffect in one DiscardCardsEffect. The Tera rule is
//! TERA_RULE spelled out (see `tera_rule`).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "HearthflameMaskOgerponex", mask: mask(&[k::ATTACK, k::PUT_DAMAGE]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let a = g.st.players[p].active;
        let d = g.st.players[p].slots[a as usize].damage * 2;
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage = d;
        }
        return Ok(());
    }
    if was_attack_used(g, e, 1, me) {
        let (p, opp, attack, source) = match *g.e(e) {
            Effect::Attack { p, opp, attack, source, .. } => (p, opp, attack, source),
            _ => return Ok(()),
        };
        let pu = p as usize;
        let o = opp as usize;
        let oa = g.st.players[o].active;
        let basic = g.st.slot_pokemon(o, oa).map(|c| g.st.cdef(c).stage == Stage::Basic as u8).unwrap_or(false);
        if !basic {
            if let Effect::Attack { damage, .. } = g.e_mut(e) {
                *damage += 140;
            }
        }
        let active = SlotRef::new(pu, g.st.players[pu].active);
        let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p, source: active, energy_map: SVec::new() })?;
        let mut cards: SVec<CardId, 16> = SVec::new();
        if let Effect::CheckProvidedEnergy { energy_map, .. } = pe {
            for m in energy_map.iter() {
                cards.push(m.card);
            }
        }
        let b = AtkBase { attack_effect: e, player: p, opponent: opp, attack, source, target: active };
        g.run_fx(Effect::DiscardCards { b, cards })?;
    }
    tera_rule(g, e, me);
    Ok(())
}
