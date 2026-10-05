//! Poké Ball (JU): flip a coin. If heads, you may search your deck for any
//! Basic Pokémon or Evolution card, show it to your opponent, and put it
//! into your hand. Shuffle your deck afterward.
//!
//! Twinleaf quirks kept: the choice allows any Pokémon (min 0, max 1); a
//! chosen card first goes through a no-op discard→deck MOVE_CARDS, the
//! reveal is queued without waiting, then deck→hand; MOVE_CARDS runs even
//! with nothing chosen. Tails or not, the final shuffle has no trailing wait.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "PokeBall", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: Some(coin), can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    // Fixed (phase 4b, rulings 779/851): a search of an empty deck is not possible, so the card can't be played.
    if g.st.players[p].deck.is_empty() {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    g.set_prevent(e, true);
    let mut f = CardFrame::at(0);
    f.a[0] = p as i32;
    g.coin_flip(p, CoinCb::Card { card: me, frame: f })?;
    Ok(())
}

fn shuffle(g: &mut Game, p: usize) {
    let id = g.player_id(p);
    g.prompt(id, "", PromptKind::ShuffleDeck, Cont::ShuffleApplyNoWait { p: p as u8 });
}

fn coin(g: &mut Game, me: CardId, f: CardFrame, heads: bool) -> R {
    let p = f.a[0] as usize;
    if !heads {
        shuffle(g, p);
        return Ok(());
    }
    let mut nf = CardFrame::at(1);
    nf.a[0] = p as i32;
    choose_cards(g, p, "CHOOSE_CARD_TO_HAND", ListRef::Deck(p as u8), Filter::super_type(SuperType::Pokemon), ChooseCardsOpts::new(0, 1, false), Cont::Card { card: me, frame: nf });
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let cards: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
    if !cards.is_empty() {
        move_cards(g, ListRef::Discard(p as u8), ListRef::Deck(p as u8), &cards, me)?;
        let id = g.player_id(1 - p);
        g.prompt(id, "CARDS_SHOWED_BY_THE_OPPONENT", PromptKind::ShowCards, Cont::Noop);
    }
    move_cards(g, ListRef::Deck(p as u8), ListRef::Hand(p as u8), &cards, me)?;
    shuffle(g, p);
    Ok(())
}
