//! Mystery Garden (MEG / ASC, stadium): once during each player's turn, that
//! player may discard 1 Energy card from their hand. If they do, that player
//! draws cards until they have as many cards in hand as they have [P]
//! Pokémon in play.
//!
//! Twinleaf: throws when the hand holds no Energy; the choice may be
//! cancelled (nothing happens); the [P] count uses the top card of each slot
//! (printed type, no CheckPokemonTypeEffect); the draw is a MOVE_CARDS with
//! `count` (no clamp to the deck size, no ability or stadium block check).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "MysteryGarden", mask: mask(&[k::USE_STADIUM]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match *g.e(e) {
        Effect::UseStadium { p, .. } if g.st.stadium_card() == Some(me) => p as usize,
        _ => return Ok(()),
    };
    if g.st.players[p].hand.iter().all(|c| !g.st.cdef(c).is_energy()) {
        bail!("CANNOT_USE_STADIUM");
    }
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    choose_cards(
        g,
        p,
        "CHOOSE_CARD_TO_DISCARD",
        ListRef::Hand(p as u8),
        Filter::super_type(SuperType::Energy),
        ChooseCardsOpts::new(1, 1, true),
        Cont::Card { card: me, frame: f },
    );
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let cards: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
    if cards.is_empty() {
        return Ok(());
    }
    move_cards(g, ListRef::Hand(p as u8), ListRef::Discard(p as u8), &cards, me)?;
    let psychic = for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().filter(|(_, c, _)| g.st.cdef(*c).card_type.contains(&ct::PSYCHIC)).count() as i32;
    let to_draw = psychic - g.st.players[p].hand.len() as i32;
    if to_draw > 0 {
        move_count_from(g, ListRef::Deck(p as u8), ListRef::Hand(p as u8), to_draw as usize, me)?;
    }
    Ok(())
}
