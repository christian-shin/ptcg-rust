//! Eri (TEF): your opponent reveals their hand. Discard up to 2 Item cards
//! you find there.
//!
//! Twinleaf: the card moves itself to the supporter pile, then to the discard
//! when the prompt is answered; chosen Items are discarded one MOVE_CARDS each.
//! Fixed (phase 4b, R3): min 1 when the opponent's hand holds an Item (it was 0).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Eri", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    let o = 1 - p;
    if g.st.players[p].supporter_turn > 0 {
        bail!("SUPPORTER_ALREADY_PLAYED");
    }
    move_cards(g, ListRef::Hand(p as u8), ListRef::Supporter(p as u8), &[me], me)?;
    g.set_prevent(e, true);
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    let filter = Filter { super_type: Some(SuperType::Trainer as u8), trainer_type: Some(TrainerType::Item as u8), ..Filter::none() };
    // Fixed (phase 4b, R3): when the opponent's hand holds an Item, at least 1 must be
    // discarded (a Supporter can't choose to discard zero); used as the effect of an attack
    // (Look-Alike Show) it may discard zero (ruling 1844).
    let has_item = g.st.players[o].hand.iter().any(|c| {
        let d = g.st.cdef(c);
        d.is_trainer() && d.trainer_type == TrainerType::Item as u8
    });
    choose_cards(g, p, "CHOOSE_CARD_TO_DISCARD", ListRef::Hand(o as u8), filter, ChooseCardsOpts::new(if has_item && !trainer_via_attack(g, e) { 1 } else { 0 }, 2, false), Cont::Card { card: me, frame: f });
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let o = 1 - p;
    let cards: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
    move_cards(g, ListRef::Supporter(p as u8), ListRef::Discard(p as u8), &[me], me)?;
    for c in cards {
        move_cards(g, ListRef::Hand(o as u8), ListRef::Discard(o as u8), &[c], me)?;
    }
    Ok(())
}
