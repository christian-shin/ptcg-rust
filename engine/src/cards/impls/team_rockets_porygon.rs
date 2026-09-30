//! Team Rocket's Porygon (DRI): Hacking — discard a card from your hand. If
//! you do, your opponent discards a card from their hand.
//!
//! Twinleaf: nothing happens with an empty hand; the opponent chooses their
//! own card (only if their hand is non-empty).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "TeamRocketsPorygon", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !was_attack_used(g, e, 0, me) {
        return Ok(());
    }
    let p = match *g.e(e) {
        Effect::Attack { p, .. } => p as usize,
        _ => return Ok(()),
    };
    if g.st.players[p].hand.is_empty() {
        return Ok(());
    }
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    choose_cards(g, p, "CHOOSE_CARD_TO_DISCARD", ListRef::Hand(p as u8), Filter::none(), ChooseCardsOpts::new(1, 1, false), Cont::Card { card: me, frame: f });
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    let o = 1 - p;
    let cards: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
    match f.stage {
        1 => {
            move_cards(g, ListRef::Hand(p as u8), ListRef::Discard(p as u8), &cards, me)?;
            if !g.st.players[o].hand.is_empty() {
                let mut nf = CardFrame::at(2);
                nf.a[0] = p as i32;
                choose_cards(g, o, "CHOOSE_CARD_TO_DISCARD", ListRef::Hand(o as u8), Filter::none(), ChooseCardsOpts::new(1, 1, false), Cont::Card { card: me, frame: nf });
            }
            Ok(())
        }
        2 => move_cards(g, ListRef::Hand(o as u8), ListRef::Discard(o as u8), &cards, me),
        _ => Ok(()),
    }
}
