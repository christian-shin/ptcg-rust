//! Unfair Stamp (TWM, ACE SPEC): only if one of your Pokémon was Knocked Out
//! during your opponent's last turn. Each player shuffles their hand into
//! their deck; you draw 5 cards and your opponent draws 2.
//!
//! Every Unfair Stamp copy (in any zone) adds its own marker to its owner when
//! that player's Pokémon is Knocked Out during the opponent's turn.
use crate::cards::prelude::*;
use crate::marker;

pub static IMPL: CardImpl = CardImpl {
    class: "UnfairStamp",
    mask: mask(&[k::TRAINER, k::KNOCK_OUT, k::END_TURN]),
    reduce,
    resume: Some(resume),
    coin: None,
    can_play: None,
};

fn stamp_marker() -> crate::markers::MarkerName {
    marker!("UNFAIR_STAMP_MARKER")
}

fn mv(g: &mut Game, src: ListRef, dst: ListRef, cards: Option<&[CardId]>, count: Option<i32>, me: CardId) -> R {
    g.run_fx(Effect::MoveCards {
        source: src,
        destination: dst,
        cards: cards.map(List::from_slice),
        count,
        to_top: false,
        to_bottom: false,
        skip_cleanup: false,
        source_card: me,
    })?;
    Ok(())
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Some(p) = trainer_played(g, e, me) {
        let o = 1 - p;
        if !g.st.players[p].marker.has(stamp_marker()) {
            bail!("CANNOT_PLAY_THIS_CARD");
        }
        if g.st.players[p].deck.is_empty() {
            bail!("CANNOT_PLAY_THIS_CARD");
        }
        let cards: Vec<CardId> = g.st.players[p].hand.iter().filter(|c| *c != me).collect();
        g.set_prevent(e, true);
        mv(g, ListRef::Hand(p as u8), ListRef::Deck(p as u8), Some(&cards), None, me)?;
        mv(g, ListRef::Hand(o as u8), ListRef::Deck(o as u8), None, None, me)?;
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        let (pid, oid) = (g.player_id(p), g.player_id(o));
        g.prompt_group(&[(pid, "", PromptKind::ShuffleDeck), (oid, "", PromptKind::ShuffleDeck)], Cont::Card { card: me, frame: f });
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
            g.st.players[player].marker.add(stamp_marker(), me, crate::markers::SourceType::None, crate::markers::TargetScope::None);
        }
        return Ok(());
    }

    // REMOVE_OPPONENT_LAST_TURN_MARKER_AT_END_OF_TURN (usedTurnSkip: not modeled).
    if let Effect::EndTurn { p } = *g.e(e) {
        let m = &mut g.st.players[p as usize].marker;
        if m.has_from(stamp_marker(), me) {
            m.remove_from(stamp_marker(), me);
        }
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let o = 1 - p;
    if let Some(Res::Order(ord)) = results.first() {
        crate::game::apply_order(&mut g.st.players[p].deck, ord.as_slice());
    }
    if let Some(Res::Order(ord)) = results.get(1) {
        crate::game::apply_order(&mut g.st.players[o].deck, ord.as_slice());
    }
    mv(g, ListRef::Deck(p as u8), ListRef::Hand(p as u8), None, Some(5), me)?;
    mv(g, ListRef::Deck(o as u8), ListRef::Hand(o as u8), None, Some(2), me)?;
    Ok(())
}

