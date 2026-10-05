//! Dedenne (SSP): Electromagnetic Sonar - put a Trainer card from your discard
//! pile into your hand. Gnaw - 30.
//!
//! Twinleaf: ChooseCardsPrompt (max 1, no cancel) over the discard pile; the
//! chosen cards are shown to the opponent (info prompt only when any) and
//! moved. Fixed in phase 4b (R4): min is 1 when the discard pile holds a
//! Trainer card (it was always 0, so the Trainer could be declined).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Dedenne", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        let has_trainer = g.st.players[p].discard.iter().any(|c| g.st.cdef(c).is_trainer());
        choose_cards(
            g,
            p,
            "CHOOSE_CARD_TO_HAND",
            ListRef::Discard(p as u8),
            Filter::super_type(SuperType::Trainer),
            ChooseCardsOpts::new(if has_trainer { 1 } else { 0 }, 1, false),
            Cont::Card { card: me, frame: f },
        );
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let first = results.first().copied().unwrap_or(Res::Null);
    let cards: Vec<CardId> = first.cards().to_vec();
    show_cards_to_player(g, 1 - p, cards.len());
    move_cards(g, ListRef::Discard(p as u8), ListRef::Hand(p as u8), &cards, me)?;
    Ok(())
}
