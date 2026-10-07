//! Armarouge (SSP): Combustion — 50. Crimson Blaster — discard all [R] Energy
//! from this Pokémon, and 180 damage to 1 of your opponent's Benched Pokémon.
//!
//! Twinleaf: a DiscardCardsEffect of every attached card named "Fire Energy"
//! on the Active (even when empty), then THIS_ATTACK_DOES_X_DAMAGE_TO_1_OF_
//! YOUR_OPPONENTS_BENCHED_POKEMON. Fixed in phase 4b (R4): the Active could
//! be chosen too (it used the "1 of your opponent's Pokémon" prefab).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Armarouge", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 1, me) {
        let (p, opp, attack, source) = match *g.e(e) {
            Effect::Attack { p, opp, attack, source, .. } => (p, opp, attack, source),
            _ => return Ok(()),
        };
        let a = g.st.players[p as usize].active;
        // "Discard all [R] Energy": every Energy that provides [R] (CheckProvidedEnergy), including one that
        // provides every type (Advanced Rulebook D-08).
        let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p, source: SlotRef::new(p as usize, a), energy_map: SVec::new() })?;
        let mut cards: SVec<CardId, 16> = SVec::new();
        if let Effect::CheckProvidedEnergy { energy_map, .. } = pe {
            for m in energy_map.iter() {
                if (m.provides.contains(&ct::FIRE) || m.provides.contains(&ct::ANY)) && !cards.contains(&m.card) {
                    cards.push(m.card);
                }
            }
        }
        let target = SlotRef::new(p as usize, a);
        g.run_fx(Effect::DiscardCards { b: AtkBase { attack_effect: e, player: p, opponent: opp, attack, source, target }, cards })?;
        damage_1_opponent_pokemon(g, e, 180, true);
    }
    Ok(())
}
