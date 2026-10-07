//! Archaludon (SCR 107 / PRE 70): Metal Bridge — all of your Pokémon that have
//! [M] Energy attached have no Retreat Cost. Iron Blaster — 160; during your
//! next turn this Pokémon can't attack.
//!
//! Twinleaf (set-stellar-crown/archaludon.ts): on any CheckRetreatCostEffect of
//! the owner (findCardList throws if the card is nowhere), unless blocked, a
//! CheckProvidedEnergy on the Active always runs; if this card is in play and
//! the Active provides [M], `cost = []`. Iron Blaster sets
//! `cannotAttackNextTurnPending` on the Active.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "Archaludon@Archaludon SCR|Archaludon PRE",
    mask: mask(&[k::CHECK_RETREAT_COST, k::ATTACK]),
    reduce,
    resume: None,
    coin: None,
    can_play: None,
};

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::CheckRetreatCost { p, .. } = *g.e(e) {
        let p = p as usize;
        let owner = match g.st.locate(me).and_then(|l| l.owner()) {
            Some(o) => o,
            None => bail!("INVALID_GAME_STATE"),
        };
        if owner == p && !is_ability_blocked(g, p, me, None) {
            let in_play = for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().any(|&(_, c, _)| c == me);
            let a = g.st.players[p].active;
            let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p: p as u8, source: SlotRef::new(p, a), energy_map: SVec::new() })?;
            let metal = matches!(pe, Effect::CheckProvidedEnergy { energy_map, .. } if energy_map.iter().any(|m| m.provides.contains(&ct::METAL) || m.provides.contains(&ct::ANY)));
            if in_play && metal {
                if let Effect::CheckRetreatCost { cost, no_cost, .. } = g.e_mut(e) {
                    *cost = SVec::new();
                    *no_cost = true;
                }
            }
        }
    }
    if was_attack_used(g, e, 0, me) {
        if let Effect::Attack { p, .. } = *g.e(e) {
            let p = p as usize;
            let a = g.st.players[p].active;
            g.st.players[p].slots[a as usize].cannot_attack_next_turn_pending = true;
        }
    }
    Ok(())
}
