//! Drakloak (TWM): Recon Directive - once during your turn, look at the top 2
//! cards of your deck and put 1 of them into your hand; the other goes on the
//! bottom of your deck.
//!
//! Fixed (phase 4b, R3): the choice can't be cancelled (it used to be
//! cancellable, and cancelling made the first MOVE_CARDS move every
//! looked-at card into the hand).
use crate::cards::prelude::*;
use crate::marker;

pub static IMPL: CardImpl = CardImpl {
    class: "Drakloak",
    mask: mask(&[k::PLAY_POKEMON, k::POWER, k::END_TURN]),
    reduce,
    resume: Some(resume),
    coin: None,
    can_play: None,
};

fn telling_spirit() -> crate::markers::MarkerName {
    marker!("TELLING_SPIRIT_MARKER")
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
    if let Effect::PlayPokemon { p, card, .. } = *g.e(e) {
        if card == me {
            g.st.players[p as usize].marker.remove_from(telling_spirit(), me);
        }
        return Ok(());
    }

    if was_power_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Power { p, .. } => p as usize,
            _ => return Ok(()),
        };
        if g.st.players[p].deck.is_empty() {
            bail!("CANNOT_USE_POWER");
        }
        if g.st.players[p].marker.has_from(telling_spirit(), me) {
            bail!("POWER_ALREADY_USED");
        }
        let top = g.alloc_temp(&[]);
        mv(g, ListRef::Deck(p as u8), top, None, Some(2), me)?;
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        f.l[0] = match top {
            ListRef::Temp(i) => i,
            _ => unreachable!(),
        };
        let id = g.player_id(p);
        g.prompt(
            id,
            "CHOOSE_CARD_TO_HAND",
            PromptKind::ChooseCards { cards: top, filter: Filter::none(), opts: ChooseCardsOpts::new(1, 1, false) },
            Cont::Card { card: me, frame: f },
        );
        return Ok(());
    }

    if let Effect::EndTurn { p } = *g.e(e) {
        let m = &mut g.st.players[p as usize].marker;
        if m.has_from(telling_spirit(), me) {
            m.remove_from(telling_spirit(), me);
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
    g.st.players[p].marker.add(telling_spirit(), me, crate::markers::SourceType::None, crate::markers::TargetScope::None);
    ability_used(g, p, me);
    let selected: Option<Vec<CardId>> = match results.first().copied().unwrap_or(Res::Null) {
        Res::Cards(c) => Some(c.as_slice().to_vec()),
        _ => None,
    };
    mv(g, top, ListRef::Hand(p as u8), selected.as_deref(), None, me)?;
    let bottom = g.alloc_temp(&[]);
    mv(g, top, bottom, None, None, me)?;
    mv(g, bottom, ListRef::Deck(p as u8), None, None, me)?;
    Ok(())
}

