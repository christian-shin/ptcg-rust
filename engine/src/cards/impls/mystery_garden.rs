//! Mystery Garden (MEG / ASC, stadium): once during each player's turn, that
//! player may discard 1 Energy card from their hand. If they do, that player
//! draws cards until they have as many cards in hand as they have [P]
//! Pokémon in play.
//!
//! Twinleaf: throws when the hand holds no Energy; the choice may be
//! cancelled (nothing happens); the [P] count runs a CheckPokemonTypeEffect on
//! each slot (phase 4b: it used to read the printed type); the draw is a
//! MOVE_CARDS with `count`, clamped to the deck size (phase 4b; no ability or
//! stadium block check).
//! Phase 4b (R6, rulings 1733/1734): can't be used unless it would draw at
//! least 1 card (the [P] count and deck size are checked before the prompt).
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
    let mut psychic = 0;
    for (s, _, _) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
        let t = SlotRef::new(p, s);
        let types = crate::engine::game_effect::pokemon_types(g, t);
        let (ce, _) = g.run_fx(Effect::CheckPokemonType { target: t, card_types: types })?;
        if matches!(ce, Effect::CheckPokemonType { card_types, .. } if card_types.contains(&ct::PSYCHIC)) {
            psychic += 1;
        }
    }
    if (psychic - (g.st.players[p].hand.len() as i32 - 1)).min(g.st.players[p].deck.len() as i32) <= 0 {
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
    let mut psychic = 0;
    for (s, _, _) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
        let t = SlotRef::new(p, s);
        let types = crate::engine::game_effect::pokemon_types(g, t);
        let (ce, _) = g.run_fx(Effect::CheckPokemonType { target: t, card_types: types })?;
        if matches!(ce, Effect::CheckPokemonType { card_types, .. } if card_types.contains(&ct::PSYCHIC)) {
            psychic += 1;
        }
    }
    let to_draw = (psychic - g.st.players[p].hand.len() as i32).min(g.st.players[p].deck.len() as i32);
    if to_draw > 0 {
        move_count_from(g, ListRef::Deck(p as u8), ListRef::Hand(p as u8), to_draw as usize, me)?;
    }
    Ok(())
}
