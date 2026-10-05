//! Levincia (JTG / DRI, stadium): once during each player's turn, that player
//! may put up to 2 Basic [L] Energy cards from their discard pile into their
//! hand.
//!
//! Twinleaf: throws CANNOT_USE_POWER unless the discard pile holds a basic
//! Energy providing [L]; the ChooseCardsPrompt (name "Lightning Energy",
//! min 1, max 2, no cancel: up to 2 from a public zone, rulings 1778/1853) is followed by a MOVE_CARDS to the hand (an
//! empty selection still reduces a MoveCardsEffect).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Levincia", mask: mask(&[k::USE_STADIUM]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match *g.e(e) {
        Effect::UseStadium { p, .. } if g.st.stadium_card() == Some(me) => p as usize,
        _ => return Ok(()),
    };
    let has = g.st.players[p].discard.iter().any(|c| {
        let d = g.st.cdef(c);
        d.is_energy() && d.energy_type == EnergyType::Basic as u8 && d.provides.contains(&ct::LIGHTNING)
    });
    if !has {
        bail!("CANNOT_USE_POWER");
    }
    let filter = Filter { super_type: Some(SuperType::Energy as u8), energy_type: Some(EnergyType::Basic as u8), name: Some("Lightning Energy"), ..Filter::none() };
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    choose_cards(g, p, "CHOOSE_CARD_TO_HAND", ListRef::Discard(p as u8), filter, ChooseCardsOpts::new(1, 2, false), Cont::Card { card: me, frame: f });
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let cards: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
    move_cards(g, ListRef::Discard(p as u8), ListRef::Hand(p as u8), &cards, me)?;
    Ok(())
}
