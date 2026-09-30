//! Team Rocket's Transceiver (DRI, Item): search your deck for a Team Rocket
//! Supporter, reveal it, put it into your hand, then shuffle.
//!
//! Twinleaf: the empty-deck check throws after MOVE_CARDS to the supporter
//! pile; the ShowCards prompt is awaited before the shuffle is created; the
//! shuffle has no animation wait.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "TeamRocketsTransceiver", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    move_cards(g, ListRef::Hand(p as u8), ListRef::Supporter(p as u8), &[me], me)?;
    g.set_prevent(e, true);
    if g.st.players[p].deck.is_empty() {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    let mut opts = ChooseCardsOpts::new(0, 1, false);
    for (i, c) in g.st.players[p].deck.iter().enumerate() {
        let d = g.st.cdef(c);
        if !(d.is_trainer() && d.trainer_type == TrainerType::Supporter as u8 && d.has_tag(tag::TEAM_ROCKET)) {
            opts.blocked.push(i as u8);
        }
    }
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    choose_cards(g, p, "CHOOSE_CARD_TO_HAND", ListRef::Deck(p as u8), Filter::none(), opts, Cont::Card { card: me, frame: f });
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    match f.stage {
        1 => {
            let cards: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
            move_cards(g, ListRef::Deck(p as u8), ListRef::Hand(p as u8), &cards, me)?;
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

fn shuffle(g: &mut Game, p: usize) {
    let id = g.player_id(p);
    g.prompt(id, "", PromptKind::ShuffleDeck, Cont::ShuffleApplyNoWait { p: p as u8 });
}
