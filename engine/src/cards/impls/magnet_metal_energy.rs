//! Magnetic Metal Energy ("Magnet Metal Energy M4", CRI): provides [M]. As
//! long as this card is attached to a [M] Pokémon, that Pokémon has no
//! Retreat Cost.
//!
//! Twinleaf: the [M] entry is pushed unless an EnergyEffect probe throws.
//! On every CheckRetreatCostEffect, if the player's Active holds this card
//! and the special energy isn't blocked, a CheckPokemonTypeEffect on the
//! Active decides: [M] → `cost = []`.
use crate::cards::prelude::*;
use crate::effects::EnergyEntry;

pub static IMPL: CardImpl = CardImpl {
    class: "MagnetMetalEnergy",
    mask: mask(&[k::CHECK_PROVIDED_ENERGY, k::CHECK_RETREAT_COST]),
    reduce,
    resume: None,
    coin: None,
    can_play: None,
};

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    match *g.e(e) {
        Effect::CheckProvidedEnergy { p, source, .. } => {
            if !g.st.slot(source.p as usize, source.s).cards.contains(me) {
                return Ok(());
            }
            if g.run_fx(Effect::Energy { p, card: me }).is_err() {
                return Ok(());
            }
            let mut provides = SVec::new();
            provides.push(ct::METAL);
            if let Effect::CheckProvidedEnergy { energy_map, .. } = g.e_mut(e) {
                energy_map.push(EnergyEntry { card: me, provides });
            }
        }
        Effect::CheckRetreatCost { p, .. } => {
            let p = p as usize;
            let a = g.st.players[p].active;
            let slot = g.st.slot(p, a);
            if !slot.cards.contains(me) && !slot.energies.contains(me) {
                return Ok(());
            }
            let t = SlotRef::new(p, a);
            if is_special_energy_blocked(g, p, me, t, false) {
                return Ok(());
            }
            let types = crate::engine::game_effect::pokemon_types(g, t);
            let (ct_e, _) = g.run_fx(Effect::CheckPokemonType { target: t, card_types: types })?;
            if matches!(ct_e, Effect::CheckPokemonType { card_types, .. } if card_types.contains(&ct::METAL)) {
                if let Effect::CheckRetreatCost { cost, .. } = g.e_mut(e) {
                    cost.clear();
                }
            }
        }
        _ => {}
    }
    Ok(())
}
