//! Academy at Night (SFA): once during each player's turn, that player may
//! put a card from their hand on top of their deck.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "NightTimeAcademy",
    mask: mask(&[k::USE_STADIUM]),
    reduce,
    resume: Some(resume),
    coin: None,
    can_play: None,
};

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match *g.e(e) {
        Effect::UseStadium { p, .. } if g.st.stadium_card() == Some(me) => p as usize,
        _ => return Ok(()),
    };
    // Putting a card on top of an empty deck still changes the game state (Advanced Rulebook B-04).
    if g.st.players[p].hand.is_empty() {
        bail!("CANNOT_USE_POWER");
    }
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    choose_cards(g, p, "CHOOSE_CARD_TO_DECK", ListRef::Hand(p as u8), Filter::none(), ChooseCardsOpts::new(1, 1, false), Cont::Card { card: me, frame: f });
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let cards: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
    let top = g.alloc_temp(&[]);
    if !cards.is_empty() {
        move_cards(g, ListRef::Hand(p as u8), top, &cards, me)?;
    }
    g.move_to_top_of_destination(top, ListRef::Deck(p as u8));
    Ok(())
}
