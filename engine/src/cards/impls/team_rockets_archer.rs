//! Team Rocket's Archer (DRI): only if any of your Team Rocket's Pokémon were
//! Knocked Out during your opponent's last turn. Each player shuffles their
//! hand into their deck; you draw 5 cards and your opponent draws 3.
//!
//! Twinleaf quirk kept: every Archer copy (any zone, either owner) adds its
//! own ARCHER_MARKER to the player whose Team Rocket's Pokémon is Knocked Out
//! during the other player's turn. Phase 4b: playable with an empty deck (the
//! hand is shuffled into it first), and the draws (opponent 3, then you 5)
//! run after both ShuffleDeckPrompts are answered (they used to run right
//! after the prompts were created, from the unshuffled deck); the opponent's
//! shuffle/draw is skipped if their MoveCardsEffect is prevented.
use crate::cards::prelude::*;
use crate::marker;

pub static IMPL: CardImpl = CardImpl {
    class: "TeamRocketsArcher",
    mask: mask(&[k::TRAINER, k::KNOCK_OUT, k::END_TURN]),
    reduce,
    resume: Some(resume),
    coin: None,
    can_play: None,
};

fn archer() -> crate::markers::MarkerName {
    marker!("ARCHER_MARKER")
}

fn mv_all(g: &mut Game, src: ListRef, dst: ListRef, cards: Option<&[CardId]>, me: CardId) -> R<bool> {
    let (_, prevented) = g.run_fx(Effect::MoveCards {
        source: src,
        destination: dst,
        cards: cards.map(List::from_slice),
        count: None,
        to_top: false,
        to_bottom: false,
        skip_cleanup: false,
        source_card: me,
    })?;
    Ok(prevented)
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Some(p) = trainer_played(g, e, me) {
        let o = 1 - p;
        if !g.st.players[p].marker.has(archer()) {
            bail!("CANNOT_PLAY_THIS_CARD");
        }
        if g.st.players[p].supporter_turn > 0 {
            bail!("SUPPORTER_ALREADY_PLAYED");
        }
        g.st.players[p].rocket_supporter = true;
        move_cards(g, ListRef::Hand(p as u8), ListRef::Supporter(p as u8), &[me], me)?;
        g.set_prevent(e, true);
        let cards: Vec<CardId> = g.st.players[p].hand.iter().filter(|c| *c != me).collect();
        mv_all(g, ListRef::Hand(p as u8), ListRef::Deck(p as u8), Some(&cards), me)?;
        let opp_prevented = mv_all(g, ListRef::Hand(o as u8), ListRef::Deck(o as u8), None, me)?;
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        f.l[0] = (!opp_prevented) as u8;
        let (pid, oid) = (g.player_id(p), g.player_id(o));
        if !opp_prevented {
            g.prompt_group(&[(oid, "", PromptKind::ShuffleDeck), (pid, "", PromptKind::ShuffleDeck)], Cont::Card { card: me, frame: f });
        } else {
            g.prompt_group(&[(pid, "", PromptKind::ShuffleDeck)], Cont::Card { card: me, frame: f });
        }
        return Ok(());
    }

    if let Effect::KnockOut { p, target, .. } = *g.e(e) {
        let player = p as usize;
        let opponent = 1 - player;
        let during_turn = matches!(g.st.phase, GamePhase::PlayerTurn | GamePhase::Attack);
        if !during_turn || g.st.active_player as usize != opponent {
            return Ok(());
        }
        if let Some(c) = g.st.slot_pokemon(target.p as usize, target.s) {
            if g.st.cdef(c).has_tag(tag::TEAM_ROCKET) {
                g.st.players[player].marker.add(archer(), me, crate::markers::SourceType::None, crate::markers::TargetScope::None);
            }
        }
        return Ok(());
    }

    // REMOVE_OPPONENT_LAST_TURN_MARKER_AT_END_OF_TURN (usedTurnSkip: not modeled).
    if let Effect::EndTurn { p } = *g.e(e) {
        let m = &mut g.st.players[p as usize].marker;
        if m.has_from(archer(), me) {
            m.remove_from(archer(), me);
        }
        if g.st.players[p as usize].rocket_supporter {
            g.st.players[p as usize].rocket_supporter = false;
        }
    }
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let o = 1 - p;
    if f.l[0] != 0 {
        if let Some(Res::Order(ord)) = results.first() {
            crate::game::apply_order(&mut g.st.players[o].deck, ord.as_slice());
        }
        if let Some(Res::Order(ord)) = results.get(1) {
            crate::game::apply_order(&mut g.st.players[p].deck, ord.as_slice());
        }
        draw_cards(g, o, 3)?;
    } else if let Some(Res::Order(ord)) = results.first() {
        crate::game::apply_order(&mut g.st.players[p].deck, ord.as_slice());
    }
    draw_cards(g, p, 5)?;
    Ok(())
}
