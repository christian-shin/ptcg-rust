//! Palafin (TWM): Zero to Hero — once during your turn, when this Pokémon
//! moves from the Active Spot to the Bench, you may search your deck for a
//! Palafin ex and switch it with this Pokémon (the Palafin ex goes onto the
//! slot, this card goes into the deck, then shuffle). Wave Splash — 30.
//!
//! Twinleaf: on MovedFromActiveToBenchEffect for this card during its owner's
//! turn (listed in movedFromActiveToBenchThisTurn): marker set → nothing; lock
//! probe fails → nothing; ConfirmPrompt, whose callback sets the marker
//! before looking at the answer. Yes: empty deck → nothing; otherwise a
//! ChooseCardsPrompt (Palafin ex, min 0, max 1, no cancel) and, if chosen,
//! MOVE_CARDS deck→slot then MOVE_CARDS slot→deck for this card; a bare
//! ShuffleDeckPrompt follows in every case.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "Palafin@Palafin TWM",
    mask: mask(&[k::END_TURN, k::MOVED_FROM_ACTIVE_TO_BENCH]),
    reduce,
    resume: Some(resume),
    coin: None,
    can_play: None,
};

fn used() -> crate::markers::MarkerName {
    crate::marker!("ABILITY_USED_MARKER")
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    remove_marker_at_end_of_turn(g, e, used(), me);

    if let Effect::MovedFromActiveToBench { p, card } = *g.e(e) {
        let p = p as usize;
        if card == me && g.st.active_player as usize == p && g.st.players[p].moved_from_active_to_bench_this_turn.contains(&me) {
            if g.st.players[p].marker.has_from(used(), me) {
                return Ok(());
            }
            if is_ability_blocked(g, p, me, None) {
                return Ok(());
            }
            let mut f = CardFrame::at(1);
            f.a[0] = p as i32;
            confirmation_prompt(g, p, "WANT_TO_USE_ABILITY", Cont::Card { card: me, frame: f });
        }
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    let first = results.first().copied().unwrap_or(Res::Null);
    match f.stage {
        1 => {
            g.st.players[p].marker.add(used(), me, crate::markers::SourceType::None, crate::markers::TargetScope::None);
            if !first.as_bool() {
                return Ok(());
            }
            if g.st.players[p].deck.is_empty() {
                return Ok(());
            }
            let filter = Filter { super_type: Some(SuperType::Pokemon as u8), name: Some("Palafin ex"), ..Filter::none() };
            let mut nf = CardFrame::at(2);
            nf.a[0] = p as i32;
            choose_cards(g, p, "CHOOSE_CARD_TO_EVOLVE", ListRef::Deck(p as u8), filter, ChooseCardsOpts::new(0, 1, false), Cont::Card { card: me, frame: nf });
            Ok(())
        }
        2 => {
            let cards: Vec<CardId> = first.cards().to_vec();
            if let Some(c) = cards.first().copied() {
                let slot = g.st.players[p].in_play().iter().copied().find(|s| g.st.slot(p, *s).cards.contains(me));
                if let Some(s) = slot {
                    move_cards(g, ListRef::Deck(p as u8), ListRef::Slot(p as u8, s), &[c], me)?;
                    move_cards(g, ListRef::Slot(p as u8, s), ListRef::Deck(p as u8), &[me], me)?;
                }
            }
            let id = g.player_id(p);
            g.prompt(id, "", PromptKind::ShuffleDeck, Cont::ShuffleApplyNoWait { p: p as u8 });
            Ok(())
        }
        _ => Ok(()),
    }
}
