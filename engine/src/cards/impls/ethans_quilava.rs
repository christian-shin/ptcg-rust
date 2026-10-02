//! Ethan's Quilava (DRI): Adventure Bound — once during your turn, you may
//! search your deck for an Ethan's Adventure, reveal it and put it into your
//! hand, then shuffle. Combustion — 40.
//!
//! Twinleaf: the marker check (BLOCKED_BY_EFFECT) and empty-deck check
//! (NO_CARDS_IN_DECK) come first (no ability-lock probe here); the choice is
//! cancellable (0-1); declining only shuffles. Taking the card shows it to the
//! opponent, moves it, adds the marker (player marker sourced by this card),
//! marks the ability used and shuffles.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "EthansQuilava", mask: mask(&[k::POWER, k::END_TURN]), reduce, resume: Some(resume), coin: None, can_play: None };

fn adventure_bound() -> crate::markers::MarkerName {
    crate::marker!("ADVENTURE_BOUND")
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_power_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Power { p, .. } => p as usize,
            _ => return Ok(()),
        };
        if g.st.players[p].marker.has_from(adventure_bound(), me) {
            bail!("BLOCKED_BY_EFFECT");
        }
        if g.st.players[p].deck.is_empty() {
            bail!("NO_CARDS_IN_DECK");
        }
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        let filter = Filter { name: Some("Ethan's Adventure"), ..Filter::none() };
        choose_cards(g, p, "CHOOSE_CARD_TO_HAND", ListRef::Deck(p as u8), filter, ChooseCardsOpts::new(0, 1, true), Cont::Card { card: me, frame: f });
    }
    remove_marker_at_end_of_turn(g, e, adventure_bound(), me);
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let cards: Vec<CardId> = results.first().copied().unwrap_or(Res::Null).cards().to_vec();
    if cards.is_empty() {
        shuffle_deck(g, p);
        return Ok(());
    }
    show_cards_to_player(g, 1 - p, cards.len());
    move_cards(g, ListRef::Deck(p as u8), ListRef::Hand(p as u8), &cards, me)?;
    g.st.players[p].marker.add(adventure_bound(), me, crate::markers::SourceType::None, crate::markers::TargetScope::None);
    ability_used(g, p, me);
    shuffle_deck(g, p);
    Ok(())
}
