//! Charmander (M2 / PFL): Nimble — if this Pokémon has no Energy attached to
//! it, it has no Retreat Cost. Live Coal — 20.
//!
//! Twinleaf: on any CheckRetreatCostEffect while this card is in the player's
//! Active slot (and is its top Pokémon) and the ability isn't blocked, a
//! CheckProvidedEnergyEffect on the Active with an empty map clears the cost.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Charmander@Charmander M2", mask: mask(&[k::CHECK_RETREAT_COST]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match *g.e(e) {
        Effect::CheckRetreatCost { p, .. } => p as usize,
        _ => return Ok(()),
    };
    let a = g.st.players[p].active;
    if !g.st.slot(p, a).cards.contains(me) {
        return Ok(());
    }
    if g.st.slot_pokemon(p, a) != Some(me) {
        return Ok(());
    }
    if is_ability_blocked(g, p, me, None) {
        return Ok(());
    }
    let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p: p as u8, source: SlotRef::new(p, a), energy_map: SVec::new() })?;
    let empty = match pe {
        Effect::CheckProvidedEnergy { energy_map, .. } => energy_map.is_empty(),
        _ => false,
    };
    if empty {
        if let Effect::CheckRetreatCost { cost, .. } = g.e_mut(e) {
            cost.clear();
        }
    }
    Ok(())
}
