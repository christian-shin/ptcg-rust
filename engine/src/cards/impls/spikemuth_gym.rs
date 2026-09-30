//! Spikemuth Gym (DRI, Stadium): once during each player's turn, that player
//! may search their deck for a Marnie's Pokémon, reveal it, put it into their
//! hand, then shuffle.
//!
//! Twinleaf: the search callback creates the ShowCards prompt (if a card was
//! chosen), moves the card and creates the shuffle prompt without waiting;
//! the shuffle has no animation wait.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "SpikemuthGym", mask: mask(&[k::USE_STADIUM]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let (p, stadium) = match *g.e(e) {
        Effect::UseStadium { p, stadium } if g.st.stadium_card() == Some(me) => (p as usize, stadium),
        _ => return Ok(()),
    };
    if g.st.players[p].deck.is_empty() {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    let mut opts = ChooseCardsOpts::new(0, 1, false);
    for (i, c) in g.st.players[p].deck.iter().enumerate() {
        let d = g.st.cdef(c);
        if d.is_pokemon() && !d.has_tag(tag::MARNIES) {
            opts.blocked.push(i as u8);
        }
    }
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    f.a[1] = stadium as i32;
    choose_cards(g, p, "CHOOSE_CARD_TO_HAND", ListRef::Deck(p as u8), Filter::super_type(SuperType::Pokemon), opts, Cont::Card { card: me, frame: f });
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let stadium = f.a[1] as CardId;
    let cards: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
    if !cards.is_empty() {
        let oid = g.player_id(1 - p);
        g.prompt(oid, "CARDS_SHOWED_BY_THE_OPPONENT", PromptKind::ShowCards, Cont::Noop);
    }
    for c in cards {
        move_cards(g, ListRef::Deck(p as u8), ListRef::Hand(p as u8), &[c], stadium)?;
    }
    let id = g.player_id(p);
    g.prompt(id, "", PromptKind::ShuffleDeck, Cont::ShuffleApplyNoWait { p: p as u8 });
    Ok(())
}
