//! Roto-Stick (PRE, Item): look at the top 4 cards of your deck; reveal any
//! number of Supporters there and put them into your hand; shuffle the rest
//! back into your deck.
//!
//! Twinleaf: the chosen cards are moved, the rest put back, then (if any
//! were chosen) a ShowCards prompt is awaited before the final shuffle,
//! which has no animation wait.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "RotoStick", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    if g.st.players[p].deck.is_empty() {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    g.set_prevent(e, true);
    let top = g.alloc_temp(&[]);
    move_count_from(g, ListRef::Deck(p as u8), top, 4, me)?;
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    f.l[0] = match top {
        ListRef::Temp(i) => i,
        _ => unreachable!(),
    };
    let filter = Filter { super_type: Some(SuperType::Trainer as u8), trainer_type: Some(TrainerType::Supporter as u8), ..Default::default() };
    let id = g.player_id(p);
    g.prompt(id, "CHOOSE_CARD_TO_HAND", PromptKind::ChooseCards { cards: top, filter, opts: ChooseCardsOpts::new(0, 4, false) }, Cont::Card { card: me, frame: f });
    Ok(())
}

fn shuffle(g: &mut Game, p: usize) {
    let id = g.player_id(p);
    g.prompt(id, "", PromptKind::ShuffleDeck, Cont::ShuffleApplyNoWait { p: p as u8 });
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    match f.stage {
        1 => {
            let top = ListRef::Temp(f.l[0]);
            let cards: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
            move_cards(g, top, ListRef::Hand(p as u8), &cards, me)?;
            g.run_fx(Effect::MoveCards {
                source: top,
                destination: ListRef::Deck(p as u8),
                cards: None,
                count: None,
                to_top: false,
                to_bottom: false,
                skip_cleanup: false,
                source_card: me,
            })?;
            if !cards.is_empty() {
                let mut nf = CardFrame::at(2);
                nf.a[0] = p as i32;
                let oid = g.player_id(1 - p);
                g.prompt(oid, "CARDS_SHOWED_BY_THE_OPPONENT", PromptKind::ShowCards, Cont::Card { card: me, frame: nf });
                return Ok(());
            }
            shuffle(g, p);
            Ok(())
        }
        2 => {
            shuffle(g, p);
            Ok(())
        }
        _ => Ok(()),
    }
}
