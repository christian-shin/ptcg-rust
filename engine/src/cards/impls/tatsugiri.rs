//! Tatsugiri (TWM): Attract Customers - once during your turn, if this
//! Pokémon is in the Active Spot, look at the top 6 cards of your deck,
//! reveal a Supporter there and put it into your hand; shuffle the rest back.
//!
//! Twinleaf quirks kept: the Active check is `player.active.cards[0] === this`;
//! when a Supporter is taken the deck is not shuffled (the callback returns
//! after the ShowCards prompt); cancelling moves every looked-at card into the
//! hand, then throws on `selected.length`.
use crate::cards::prelude::*;
use crate::marker;

pub static IMPL: CardImpl = CardImpl {
    class: "Tatsugiri",
    mask: mask(&[k::PLAY_POKEMON, k::POWER, k::END_TURN]),
    reduce,
    resume: Some(resume),
    coin: None,
    can_play: None,
};

fn crowd_puller() -> crate::markers::MarkerName {
    marker!("CROWD_PULLER_MARKER")
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
            g.st.players[p as usize].marker.remove_from(crowd_puller(), me);
            return Ok(());
        }
    }

    if was_power_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Power { p, .. } => p as usize,
            _ => return Ok(()),
        };
        if g.st.players[p].deck.is_empty() {
            bail!("CANNOT_USE_POWER");
        }
        if g.st.players[p].marker.has_from(crowd_puller(), me) {
            bail!("POWER_ALREADY_USED");
        }
        let a = g.st.players[p].active;
        if g.st.slot(p, a).cards.get(0) != Some(me) {
            bail!("CANNOT_USE_POWER");
        }
        let top = g.alloc_temp(&[]);
        mv(g, ListRef::Deck(p as u8), top, None, Some(6), me)?;
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        f.l[0] = match top {
            ListRef::Temp(i) => i,
            _ => unreachable!(),
        };
        let filter = Filter { super_type: Some(SuperType::Trainer as u8), trainer_type: Some(TrainerType::Supporter as u8), ..Filter::none() };
        let id = g.player_id(p);
        g.prompt(
            id,
            "CHOOSE_CARD_TO_HAND",
            PromptKind::ChooseCards { cards: top, filter, opts: ChooseCardsOpts::new(0, 1, true) },
            Cont::Card { card: me, frame: f },
        );
        return Ok(());
    }

    if let Effect::EndTurn { p } = *g.e(e) {
        let m = &mut g.st.players[p as usize].marker;
        if m.has_from(crowd_puller(), me) {
            m.remove_from(crowd_puller(), me);
        }
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    match f.stage {
        1 => {
            let top = ListRef::Temp(f.l[0]);
            g.st.players[p].marker.add(crowd_puller(), me, crate::markers::SourceType::None, crate::markers::TargetScope::None);
            ability_used(g, p, me);
            let selected: Option<Vec<CardId>> = match results.first().copied().unwrap_or(Res::Null) {
                Res::Cards(c) => Some(c.as_slice().to_vec()),
                _ => None,
            };
            mv(g, top, ListRef::Hand(p as u8), selected.as_deref(), None, me)?;
            mv(g, top, ListRef::Deck(p as u8), None, None, me)?;
            let selected = match selected {
                Some(s) => s,
                None => bail!("TypeError: Cannot read properties of null (reading 'length')"),
            };
            if !selected.is_empty() {
                let oid = g.player_id(1 - p);
                g.prompt(oid, "CARDS_SHOWED_BY_THE_OPPONENT", PromptKind::ShowCards, Cont::Noop);
                return Ok(());
            }
            let id = g.player_id(p);
            g.prompt(id, "", PromptKind::ShuffleDeck, Cont::Card { card: me, frame: CardFrame { stage: 2, ..f } });
            Ok(())
        }
        2 => {
            if let Some(Res::Order(o)) = results.first().copied() {
                crate::game::apply_order(&mut g.st.players[p].deck, o.as_slice());
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
