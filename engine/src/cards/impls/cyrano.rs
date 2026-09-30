//! Cyrano (SSP): search your deck for up to 3 Pokémon ex, reveal them, put
//! them into your hand, then shuffle.
//!
//! Twinleaf quirks kept: no supporter-turn check, the card stays in hand
//! until the search resolves (then it is moved hand → discard), and the
//! shuffle is a bare ShuffleDeckPrompt (no trailing wait).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Cyrano", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    if g.st.players[p].deck.is_empty() {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    let mut opts = ChooseCardsOpts::new(0, 3, false);
    let deck: Vec<CardId> = g.st.players[p].deck.iter().collect();
    for (i, c) in deck.iter().enumerate() {
        if !g.st.cdef(*c).has_tag(tag::POKEMON_EX_LOWER) {
            opts.blocked.push(i as u8);
        }
    }
    g.set_prevent(e, true);
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    choose_cards(g, p, "CHOOSE_CARD_TO_HAND", ListRef::Deck(p as u8), Filter::super_type(SuperType::Pokemon), opts, Cont::Card { card: me, frame: f });
    Ok(())
}

fn shuffle(g: &mut Game, me: CardId, p: usize) {
    let id = g.player_id(p);
    let mut f = CardFrame::at(3);
    f.a[0] = p as i32;
    g.prompt(id, "", PromptKind::ShuffleDeck, Cont::Card { card: me, frame: f });
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    let first = results.first().copied().unwrap_or(Res::Null);
    match f.stage {
        1 => {
            let cards: Vec<CardId> = first.cards().to_vec();
            move_cards(g, ListRef::Deck(p as u8), ListRef::Hand(p as u8), &cards, me)?;
            move_cards(g, ListRef::Hand(p as u8), ListRef::Discard(p as u8), &[me], me)?;
            if !cards.is_empty() {
                let id = g.player_id(1 - p);
                let mut nf = CardFrame::at(2);
                nf.a[0] = p as i32;
                g.prompt(id, "CARDS_SHOWED_BY_THE_OPPONENT", PromptKind::ShowCards, Cont::Card { card: me, frame: nf });
                return Ok(());
            }
            shuffle(g, me, p);
            Ok(())
        }
        2 => {
            shuffle(g, me, p);
            Ok(())
        }
        3 => {
            if let Res::Order(o) = first {
                crate::game::apply_order(&mut g.st.players[p].deck, o.as_slice());
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
