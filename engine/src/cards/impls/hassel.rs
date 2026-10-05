//! Hassel (TWM): only if any of your Pokémon were Knocked Out during your
//! opponent's last turn. Look at the top 8 cards of your deck; put up to 3
//! into your hand and shuffle the rest into your deck.
//!
//! Twinleaf: every Hassel copy (in any zone) adds its own HASSEL_MARKER to its
//! owner when that player's Pokémon is Knocked Out during the opponent's
//! turn; the marker check comes after the Supporter moves to the supporter
//! pile. The remaining top cards go back to the deck bottom before the
//! (wait-less) shuffle is answered.
use crate::cards::prelude::*;
use crate::marker;

pub static IMPL: CardImpl = CardImpl { class: "Hassel", mask: mask(&[k::TRAINER, k::KNOCK_OUT, k::END_TURN]), reduce, resume: Some(resume), coin: None, can_play: None };

fn hassel_marker() -> crate::markers::MarkerName {
    marker!("HASSEL_MARKER")
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Some(p) = trainer_played(g, e, me) {
        if g.st.players[p].supporter_turn > 0 {
            bail!("SUPPORTER_ALREADY_PLAYED");
        }
        let played_from_hand = g.st.players[p].hand.iter().any(|c| c == me);
        move_cards(g, ListRef::Hand(p as u8), ListRef::Supporter(p as u8), &[me], me)?;
        g.set_prevent(e, true);
        if !g.st.players[p].marker.has(hassel_marker()) {
            bail!("CANNOT_PLAY_THIS_CARD");
        }
        if g.st.players[p].deck.is_empty() {
            bail!("CANNOT_PLAY_THIS_CARD");
        }
        if g.st.players[p].supporter_turn > 0 {
            bail!("SUPPORTER_ALREADY_PLAYED");
        }
        let top = g.alloc_temp(&[]);
        move_count_from(g, ListRef::Deck(p as u8), top, 8, me)?;
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        f.l[0] = match top {
            ListRef::Temp(i) => i,
            _ => 0,
        };
        // Fixed (phase 4b, rulings 1778/1853): the top cards are looked at, so "up to 3" takes at least 1 when played from
        // the hand; used through an attack (Look-Alike Show) it may be 0 (ruling 1844).
        let min = if played_from_hand { 1 } else { 0 };
        choose_cards(g, p, "CHOOSE_CARD_TO_HAND", top, Filter::none(), ChooseCardsOpts::new(min, 3, false), Cont::Card { card: me, frame: f });
        return Ok(());
    }

    if let Effect::KnockOut { p, .. } = *g.e(e) {
        let player = p as usize;
        let opponent = 1 - player;
        let during_turn = matches!(g.st.phase, GamePhase::PlayerTurn | GamePhase::Attack);
        if !during_turn || g.st.active_player as usize != opponent {
            return Ok(());
        }
        let owner = g.st.locate(me).and_then(|l| l.owner()).unwrap_or_else(|| g.st.owner(me));
        if owner == player {
            g.st.players[player].marker.add(hassel_marker(), me, crate::markers::SourceType::None, crate::markers::TargetScope::None);
        }
        return Ok(());
    }

    // REMOVE_OPPONENT_LAST_TURN_MARKER_AT_END_OF_TURN (usedTurnSkip: not modeled).
    if let Effect::EndTurn { p } = *g.e(e) {
        let m = &mut g.st.players[p as usize].marker;
        if m.has_from(hassel_marker(), me) {
            m.remove_from(hassel_marker(), me);
        }
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let top = ListRef::Temp(f.l[0]);
    let cards: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
    move_cards(g, top, ListRef::Hand(p as u8), &cards, me)?;
    g.run_fx(Effect::MoveCards {
        source: top,
        destination: ListRef::Deck(p as u8),
        cards: None,
        count: None,
        to_top: false,
        to_bottom: false,
        skip_cleanup: false,
        source_card: me,
    })?;
    let id = g.player_id(p);
    g.prompt(id, "", PromptKind::ShuffleDeck, Cont::ShuffleApplyNoWait { p: p as u8 });
    Ok(())
}
