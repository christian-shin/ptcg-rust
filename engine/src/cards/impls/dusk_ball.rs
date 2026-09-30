//! Dusk Ball (SSP): look at the 7 cards from the bottom of your deck; choose
//! 1 Pokémon there, show it to your opponent, and put it into your hand.
//! Put the rest back and shuffle your deck.
//!
//! Twinleaf: the bottom cards are spliced into a temporary list directly (no
//! MOVE_CARDS). With nothing chosen, the `temp.cards.forEach` loop runs once
//! (its first MOVE_CARDS empties the array) if anything was looked at. With
//! a Pokémon chosen, the reveal is queued without waiting. The card is moved
//! supporter→discard (again) before the wait-less shuffle.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "DuskBall", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    g.set_prevent(e, true);
    let size = g.st.players[p].deck.len();
    let n = size.min(7);
    let start = size - n;
    let cards: Vec<CardId> = g.st.players[p].deck.as_slice()[start..].to_vec();
    for _ in 0..n {
        g.st.players[p].deck.remove_at(start);
    }
    let temp = g.alloc_temp(&cards);
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    f.l[0] = match temp {
        ListRef::Temp(i) => i,
        _ => 0,
    };
    let id = g.player_id(p);
    g.prompt(
        id,
        "CHOOSE_CARD_TO_HAND",
        PromptKind::ChooseCards { cards: temp, filter: Filter::super_type(SuperType::Pokemon), opts: ChooseCardsOpts::new(0, 1, false) },
        Cont::Card { card: me, frame: f },
    );
    Ok(())
}

fn move_all(g: &mut Game, src: ListRef, dst: ListRef, me: CardId) -> R {
    g.run_fx(Effect::MoveCards { source: src, destination: dst, cards: None, count: None, to_top: false, to_bottom: false, skip_cleanup: false, source_card: me })?;
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let pu = p as u8;
    let temp = ListRef::Temp(f.l[0]);
    let chosen: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
    if chosen.is_empty() && !g.lst(temp).is_empty() {
        move_all(g, temp, ListRef::Deck(pu), me)?;
        move_cards(g, ListRef::Supporter(pu), ListRef::Discard(pu), &[me], me)?;
    }
    if !chosen.is_empty() {
        move_cards(g, temp, ListRef::Hand(pu), &chosen[..1], me)?;
        move_all(g, temp, ListRef::Deck(pu), me)?;
        move_cards(g, ListRef::Supporter(pu), ListRef::Discard(pu), &[me], me)?;
        let id = g.player_id(1 - p);
        g.prompt(id, "CARDS_SHOWED_BY_THE_OPPONENT", PromptKind::ShowCards, Cont::Noop);
    }
    move_cards(g, ListRef::Supporter(pu), ListRef::Discard(pu), &[me], me)?;
    let id = g.player_id(p);
    g.prompt(id, "", PromptKind::ShuffleDeck, Cont::ShuffleApplyNoWait { p: pu });
    Ok(())
}
