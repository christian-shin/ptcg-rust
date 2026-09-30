//! Ultra Ball (SVI): discard 2 other cards from your hand; search your deck
//! for a Pokémon, reveal it, put it into your hand, then shuffle.
//!
//! Twinleaf's final ShuffleDeckPrompt has no trailing WaitPrompt.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "UltraBall", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    let others: Vec<CardId> = g.st.players[p].hand.iter().filter(|c| *c != me).collect();
    if others.len() < 2 || g.st.players[p].deck.is_empty() {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    let temp = g.alloc_temp(&others);
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    let id = g.player_id(p);
    g.prompt(
        id,
        "CHOOSE_CARD_TO_DISCARD",
        PromptKind::ChooseCards { cards: temp, filter: Filter::none(), opts: ChooseCardsOpts::new(2, 2, false) },
        Cont::Card { card: me, frame: f },
    );
    Ok(())
}

fn shuffle(g: &mut Game, me: CardId, p: usize) {
    let mut f = CardFrame::at(4);
    f.a[0] = p as i32;
    let id = g.player_id(p);
    g.prompt(id, "", PromptKind::ShuffleDeck, Cont::Card { card: me, frame: f });
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    let first = results.first().copied().unwrap_or(Res::Null);
    match f.stage {
        1 => {
            if let Res::Cards(c) = first {
                move_cards(g, ListRef::Hand(p as u8), ListRef::Discard(p as u8), c.as_slice(), NO_CARD)?;
            }
            let mut nf = CardFrame::at(2);
            nf.a[0] = p as i32;
            let opts = ChooseCardsOpts::new(0, 1, false);
            choose_cards(g, p, "CHOOSE_CARD_TO_HAND", ListRef::Deck(p as u8), Filter::super_type(SuperType::Pokemon), opts, Cont::Card { card: me, frame: nf });
            Ok(())
        }
        2 => {
            let cards: Vec<CardId> = first.cards().to_vec();
            if !cards.is_empty() {
                move_cards(g, ListRef::Deck(p as u8), ListRef::Hand(p as u8), &cards, me)?;
                let mut nf = CardFrame::at(3);
                nf.a[0] = p as i32;
                let id = g.player_id(1 - p);
                g.prompt(id, "CARDS_SHOWED_BY_THE_OPPONENT", PromptKind::ShowCards, Cont::Card { card: me, frame: nf });
            } else {
                shuffle(g, me, p);
            }
            Ok(())
        }
        3 => {
            shuffle(g, me, p);
            Ok(())
        }
        4 => {
            if let Res::Order(o) = first {
                crate::game::apply_order(&mut g.st.players[p].deck, o.as_slice());
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

