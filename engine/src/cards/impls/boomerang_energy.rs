//! Boomerang Energy (TWM): provides [C]. If discarded by an effect of an
//! attack of the Pokémon it is attached to, attach it from the discard pile
//! to that Pokémon after attacking.
//!
//! Twinleaf quirk kept: the card is re-attached at EndTurn to whatever is
//! Active then.
//!
//! Fixed (phase 4b, W4): any DiscardCardsEffect of the attacking player's
//! turn armed the re-attach; it now needs this card among the discarded cards.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "BoomerangEnergy",
    mask: mask(&[k::ATTACK, k::DISCARD_CARDS, k::END_TURN]),
    reduce,
    resume: None,
    coin: None,
    can_play: None,
};

fn existence() -> crate::markers::MarkerName {
    crate::marker!("BOOMERANG_EXISTANCE_MARKER")
}

fn discarded() -> crate::markers::MarkerName {
    crate::marker!("BOOMERANG_DISCARDED_MARKER")
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::Attack { p, source, .. } = *g.e(e) {
        let pu = p as usize;
        if g.st.slot(source.p as usize, source.s).cards.contains(me) && source.p == p && g.st.players[pu].active == source.s {
            if is_special_energy_blocked(g, pu, me, source, false) {
                return Ok(());
            }
            g.st.players[pu].marker.add(existence(), me, crate::markers::SourceType::None, crate::markers::TargetScope::None);
        }
    }

    if let Effect::DiscardCards { b, ref cards } = *g.e(e) {
        let pu = b.player as usize;
        if cards.contains(&me) && g.st.players[pu].marker.has_from(existence(), me) {
            if is_special_energy_blocked(g, pu, me, b.source, false) {
                return Ok(());
            }
            g.st.players[pu].marker.add(discarded(), me, crate::markers::SourceType::None, crate::markers::TargetScope::None);
        }
    }

    if let Effect::EndTurn { p } = *g.e(e) {
        let pu = p as usize;
        if g.st.players[pu].marker.has_from(existence(), me) {
            g.st.players[pu].marker.remove_from(existence(), me);
            if g.st.players[pu].marker.has_from(discarded(), me) {
                g.st.players[pu].marker.remove_from(discarded(), me);
                if g.st.players[pu].discard.iter().any(|c| c == me) {
                    let a = g.st.players[pu].active;
                    move_cards(g, ListRef::Discard(p), ListRef::Slot(p, a), &[me], me)?;
                }
            }
        }
    }
    Ok(())
}
