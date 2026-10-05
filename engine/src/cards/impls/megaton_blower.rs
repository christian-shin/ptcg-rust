//! Megaton Blower (SSP, ACE SPEC): discard all Pokémon Tools and Special
//! Energy from all of your opponent's Pokémon, and discard a Stadium in play.
//!
//! Twinleaf: the card throws CANNOT_PLAY_THIS_CARD when there is no Stadium in
//! play and no opposing Pokémon has a Tool or a Special Energy (phase 4b,
//! ruling n=1610); the Stadium goes first; each of the opponent's Pokémon
//! (Active, then occupied Bench slots) is skipped when a TrainerTargetEffect
//! on it is blocked; the Special Energy in `cardList.cards` and (phase 4b: they
//! used to be missed, since attached Tools live in `tools`) the attached Tools
//! are discarded. The card then moves supporter→discard.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "MegatonBlower", mask: mask(&[k::TRAINER]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    let o = 1 - p;
    let mut has_target = g.st.stadium_card().is_some();
    for s in std::iter::once(g.st.players[o].active).chain(g.st.players[o].bench.iter().copied()) {
        let slot = g.st.slot(o, s);
        if !slot.tools.is_empty() || slot.cards.iter().any(|c| {
            let d = g.st.cdef(c);
            d.is_energy() && d.energy_type == EnergyType::Special as u8
        }) {
            has_target = true;
        }
    }
    if !has_target {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    g.set_prevent(e, true);
    if let Some(stadium) = g.st.stadium_card() {
        if let Some(l) = g.st.locate(stadium) {
            let owner = l.owner().unwrap_or(p);
            move_cards(g, l, ListRef::Discard(owner as u8), &[stadium], me)?;
        }
    }
    let mut slots: Vec<SlotId> = vec![g.st.players[o].active];
    for &b in g.st.players[o].bench.iter() {
        if !g.st.slot(o, b).cards.is_empty() {
            slots.push(b);
        }
    }
    for s in slots {
        let target = SlotRef::new(o, s);
        let (t, prevented) = g.run_fx(Effect::TrainerTarget { p: p as u8, card: me, target: Some(target) })?;
        if prevented || matches!(t, Effect::TrainerTarget { target: None, .. }) {
            continue;
        }
        let slot = g.st.slot(o, s);
        let mut cards: Vec<CardId> = slot
            .cards
            .iter()
            .filter(|c| {
                let d = g.st.cdef(*c);
                d.is_energy() && d.energy_type == EnergyType::Special as u8
            })
            .collect();
        cards.extend(slot.tools.iter());
        if !cards.is_empty() {
            move_cards(g, target.list(), ListRef::Discard(o as u8), &cards, NO_CARD)?;
        }
    }
    move_cards(g, ListRef::Supporter(p as u8), ListRef::Discard(p as u8), &[me], NO_CARD)
}
