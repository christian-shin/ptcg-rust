//! Ignition Energy (SV11W, Special): provides [C]; [C][C][C] on an Evolution
//! Pokémon. If attached to 1 of your Pokémon, discard it at the end of your
//! turn.
//!
//! Twinleaf: attaching (AttachEnergyEffect of this card) adds a player
//! marker sourced by this card; between turns, for the player holding it,
//! every in-play slot holding the card discards it and removes the marker
//! (the marker stays if the card is no longer in play). No Special Energy
//! block probe anywhere. Stage is read from the slot's top Pokémon; not
//! Basic and not Restored gives [C][C][C].
use crate::cards::prelude::*;
use crate::effects::EnergyEntry;
use crate::marker;

pub static IMPL: CardImpl = CardImpl {
    class: "IgnitionEnergy",
    mask: mask(&[k::ATTACH_ENERGY, k::CHECK_PROVIDED_ENERGY, k::BETWEEN_TURNS]),
    reduce,
    resume: None,
    coin: None,
    can_play: None,
};

fn ignition() -> crate::markers::MarkerName {
    marker!("IGNITION_ENERGY_MARKER")
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    match *g.e(e) {
        Effect::AttachEnergy { p, card, .. } => {
            if card == me {
                g.st.players[p as usize].marker.add(ignition(), me, crate::markers::SourceType::None, crate::markers::TargetScope::None);
            }
        }
        Effect::CheckProvidedEnergy { source, .. } => {
            if !g.st.slot(source.p as usize, source.s).cards.contains(me) {
                return Ok(());
            }
            let stage = match g.st.slot_pokemon(source.p as usize, source.s) {
                Some(c) => g.st.cdef(c).stage,
                None => return Ok(()),
            };
            let mut provides = SVec::new();
            if stage == Stage::Basic as u8 {
                provides.push(ct::COLORLESS);
            } else if stage != Stage::Restored as u8 {
                provides.push(ct::COLORLESS);
                provides.push(ct::COLORLESS);
                provides.push(ct::COLORLESS);
            } else {
                return Ok(());
            }
            if let Effect::CheckProvidedEnergy { energy_map, .. } = g.e_mut(e) {
                energy_map.push(EnergyEntry { card: me, provides });
            }
        }
        Effect::BetweenTurns { p, .. } => {
            let p = p as usize;
            if !g.st.players[p].marker.has_from(ignition(), me) {
                return Ok(());
            }
            for (s, _, _) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
                if g.st.slot(p, s).cards.contains(me) {
                    move_cards(g, ListRef::Slot(p as u8, s), ListRef::Discard(p as u8), &[me], me)?;
                    g.st.players[p].marker.remove_from(ignition(), me);
                }
            }
        }
        _ => {}
    }
    Ok(())
}
