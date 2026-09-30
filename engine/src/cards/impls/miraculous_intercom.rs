//! Miraculous Intercom (SSP, ACE SPEC Item): put up to 2 Supporter cards
//! from your discard pile into your hand.
//!
//! Twinleaf quirks kept: the prompt requires at least 1 card; the card is
//! MOVE_CARDS'd hand -> supporter pile (already there), then after the
//! reveal hand -> discard (a no-op), the chosen cards discard -> hand, and
//! finally supporter pile -> discard.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "MiraculousIntercom", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    let has = g.st.players[p].discard.iter().any(|c| {
        let d = g.st.cdef(c);
        d.is_trainer() && d.trainer_type == TrainerType::Supporter as u8
    });
    if !has {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    g.set_prevent(e, true);
    move_cards(g, ListRef::Hand(p as u8), ListRef::Supporter(p as u8), &[me], me)?;
    let filter = Filter { super_type: Some(SuperType::Trainer as u8), trainer_type: Some(TrainerType::Supporter as u8), ..Default::default() };
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    choose_cards(g, p, "CHOOSE_CARD_TO_HAND", ListRef::Discard(p as u8), filter, ChooseCardsOpts::new(1, 2, false), Cont::Card { card: me, frame: f });
    Ok(())
}

fn finish(g: &mut Game, me: CardId, p: usize, cards: &[CardId]) -> R {
    if !cards.is_empty() {
        move_cards(g, ListRef::Hand(p as u8), ListRef::Discard(p as u8), &[me], me)?;
        move_cards(g, ListRef::Discard(p as u8), ListRef::Hand(p as u8), cards, me)?;
    }
    move_cards(g, ListRef::Supporter(p as u8), ListRef::Discard(p as u8), &[me], me)
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    match f.stage {
        1 => {
            let cards: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
            if !cards.is_empty() {
                let mut nf = CardFrame::at(2);
                nf.a[0] = p as i32;
                // Keep the chosen cards (at most 2) for after the reveal.
                nf.a[1] = cards.len() as i32;
                for (i, c) in cards.iter().take(2).enumerate() {
                    nf.a[2 + i] = *c as i32;
                }
                let oid = g.player_id(1 - p);
                g.prompt(oid, "CARDS_SHOWED_BY_THE_OPPONENT", PromptKind::ShowCards, Cont::Card { card: me, frame: nf });
                return Ok(());
            }
            finish(g, me, p, &[])
        }
        2 => {
            let cards: Vec<CardId> = (0..f.a[1].min(2) as usize).map(|i| f.a[2 + i] as CardId).collect();
            finish(g, me, p, &cards)
        }
        _ => Ok(()),
    }
}
