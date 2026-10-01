//! Energy Retrieval (SVI, as Energy Retrieval FA CRI): put 2 basic Energy
//! cards from your discard pile into your hand.
//!
//! Twinleaf (scarlet-and-violet file): a DiscardToHandEffect probe first (if
//! prevented, nothing happens); throws with no Basic Energy in the discard;
//! ChooseCardsPrompt min 1, max min(2, Basic Energy), no cancel. The callback
//! moves the cards, shows them to the opponent (ShowCardsPrompt), then runs
//! the same MOVE_CARDS again (the cards are no longer in the discard).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "EnergyRetrieval@CRI", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn basic_energy() -> Filter {
    Filter { super_type: Some(SuperType::Energy as u8), energy_type: Some(EnergyType::Basic as u8), ..Filter::none() }
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    let (_, prevented) = g.run_fx(Effect::DiscardToHand { p: p as u8, card: me })?;
    if prevented {
        return Ok(());
    }
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
    let max = n.min(2) as u8;
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    choose_cards(g, p, "CHOOSE_CARD_TO_HAND", ListRef::Discard(p as u8), basic_energy(), ChooseCardsOpts::new(1, max, false), Cont::Card { card: me, frame: f });
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let cards: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
    if !cards.is_empty() {
        move_cards(g, ListRef::Discard(p as u8), ListRef::Hand(p as u8), &cards, me)?;
        show_cards_to_player(g, 1 - p, cards.len());
        move_cards(g, ListRef::Discard(p as u8), ListRef::Hand(p as u8), &cards, me)?;
    }
    Ok(())
}
