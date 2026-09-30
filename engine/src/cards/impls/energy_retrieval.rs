//! Energy Retrieval (BS): trade 1 of the other cards in your hand for
//! up to 2 basic Energy cards from your discard pile.
//!
//! Twinleaf: throws with no Basic Energy in the discard; the hand card is
//! chosen from a temporary copy of the hand (min 1, no cancel); the second
//! prompt's max is min(2, Basic Energy counted before the discard), min 1.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "EnergyRetrieval@BS", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn basic_energy() -> Filter {
    Filter { super_type: Some(SuperType::Energy as u8), energy_type: Some(EnergyType::Basic as u8), ..Filter::none() }
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    let n = g.st.players[p]
        .discard
        .iter()
        .filter(|c| {
            let d = g.st.cdef(*c);
            d.is_energy() && d.energy_type == EnergyType::Basic as u8
        })
        .count();
    if n == 0 {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    g.set_prevent(e, true);
    let hand: Vec<CardId> = g.st.players[p].hand.iter().filter(|c| *c != me).collect();
    let temp = g.alloc_temp(&hand);
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    f.a[1] = n as i32;
    choose_cards(g, p, "CHOOSE_CARD_TO_DISCARD", temp, Filter::none(), ChooseCardsOpts::new(1, 1, false), Cont::Card { card: me, frame: f });
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    let first = results.first().copied().unwrap_or(Res::Null);
    match f.stage {
        1 => {
            let cards: Vec<CardId> = first.cards().to_vec();
            if cards.is_empty() {
                return Ok(());
            }
            move_cards(g, ListRef::Hand(p as u8), ListRef::Discard(p as u8), &cards, me)?;
            let max = (f.a[1] as u8).min(2);
            let nf = CardFrame { stage: 2, ..f };
            choose_cards(g, p, "CHOOSE_CARD_TO_HAND", ListRef::Discard(p as u8), basic_energy(), ChooseCardsOpts::new(1, max, false), Cont::Card { card: me, frame: nf });
            Ok(())
        }
        2 => {
            let cards: Vec<CardId> = first.cards().to_vec();
            if !cards.is_empty() {
                move_cards(g, ListRef::Discard(p as u8), ListRef::Hand(p as u8), &cards, me)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
