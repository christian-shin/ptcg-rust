//! Master Ball (TEF, ACE SPEC): search your deck for a Pokémon, reveal it,
//! and put it into your hand. Then, shuffle your deck.
//!
//! Twinleaf order: the reveal (if any) comes before MOVE_CARDS, then the
//! ShuffleDeckPrompt. Hyper Aroma shares this flow.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "MasterBall", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    search(g, me, e, p, Filter::super_type(SuperType::Pokemon), 1)
}

/// Deck-empty check, preventDefault, then the choice (min 0, no cancel).
pub(crate) fn search(g: &mut Game, me: CardId, e: EffId, p: usize, filter: Filter, max: u8) -> R {
    if g.st.players[p].deck.is_empty() {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    g.set_prevent(e, true);
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    choose_cards(g, p, "CHOOSE_CARD_TO_HAND", ListRef::Deck(p as u8), filter, ChooseCardsOpts::new(0, max, false), Cont::Card { card: me, frame: f });
    Ok(())
}

fn finish(g: &mut Game, me: CardId, p: usize, cards: &[CardId]) -> R {
    move_cards(g, ListRef::Deck(p as u8), ListRef::Hand(p as u8), cards, me)?;
    let mut f = CardFrame::at(3);
    f.a[0] = p as i32;
    let id = g.player_id(p);
    g.prompt(id, "", PromptKind::ShuffleDeck, Cont::Card { card: me, frame: f });
    Ok(())
}

pub(crate) fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    let first = results.first().copied().unwrap_or(Res::Null);
    match f.stage {
        1 => {
            let cards: Vec<CardId> = first.cards().to_vec();
            if !cards.is_empty() {
                // The chosen cards travel in a temp list until the reveal resolves.
                let temp = g.alloc_temp(&cards);
                let mut nf = CardFrame::at(2);
                nf.a[0] = p as i32;
                nf.l[0] = match temp {
                    ListRef::Temp(i) => i,
                    _ => unreachable!(),
                };
                let id = g.player_id(1 - p);
                g.prompt(id, "CARDS_SHOWED_BY_THE_OPPONENT", PromptKind::ShowCards, Cont::Card { card: me, frame: nf });
                return Ok(());
            }
            finish(g, me, p, &[])
        }
        2 => {
            let cards: Vec<CardId> = g.lst(ListRef::Temp(f.l[0])).to_vec();
            finish(g, me, p, &cards)
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
