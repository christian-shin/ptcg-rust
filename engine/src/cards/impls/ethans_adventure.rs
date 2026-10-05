//! Ethan's Adventure (DRI): search your deck for up to 3 in any combination
//! of Ethan's Pokémon and Basic [R] Energy, reveal them, put them into your
//! hand, shuffle.
//!
//! Twinleaf: the card moves to the supporter pile, the empty-deck check
//! follows, non-matching cards are blocked (Energy must be a Basic card
//! named "Fire Energy"). The Supporter is discarded right after the prompt
//! is created, before it is answered; the callback (when something was
//! chosen) creates the ShowCards prompt, moves the cards and SHUFFLE_DECKs
//! (shuffle + silent wait).
//!
//! Fixed (phase 4b, R3): the deck is shuffled even when nothing was taken
//! (the callback used to return before SHUFFLE_DECK).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "EthansAdventure", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    if g.st.players[p].supporter_turn > 0 {
        bail!("SUPPORTER_ALREADY_PLAYED");
    }
    move_cards(g, ListRef::Hand(p as u8), ListRef::Supporter(p as u8), &[me], me)?;
    if g.st.players[p].deck.is_empty() {
        bail!("NO_CARDS_IN_DECK");
    }
    let mut opts = ChooseCardsOpts::new(0, 3, false);
    for (i, c) in g.st.players[p].deck.iter().enumerate() {
        let d = g.st.cdef(c);
        let pokemon = d.is_pokemon() && d.has_tag(tag::ETHANS);
        let fire = d.is_energy() && d.energy_type == 0 && d.name == "Fire Energy";
        if !pokemon && !fire {
            opts.blocked.push(i as u8);
        }
    }
    g.set_prevent(e, true);
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    choose_cards(g, p, "CHOOSE_CARD_TO_DECK", ListRef::Deck(p as u8), Filter::none(), opts, Cont::Card { card: me, frame: f });
    move_cards(g, ListRef::Supporter(p as u8), ListRef::Discard(p as u8), &[me], me)?;
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let cards: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
    if !cards.is_empty() {
        show_cards_to_player(g, 1 - p, cards.len());
        move_cards(g, ListRef::Deck(p as u8), ListRef::Hand(p as u8), &cards, me)?;
    }
    shuffle_deck(g, p);
    Ok(())
}
