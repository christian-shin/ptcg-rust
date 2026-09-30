//! Brilliant Blender (SSP, ACE SPEC): search your deck for up to 5 cards and
//! discard them. Then, shuffle your deck.
//!
//! Twinleaf quirks kept: the choice is min 1 / max 5; the chosen cards are
//! shown to the opponent before they are discarded; the final shuffle has
//! no trailing wait.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "BrilliantBlender", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    if g.st.players[p].deck.is_empty() {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    g.set_prevent(e, true);
    move_cards(g, ListRef::Hand(p as u8), ListRef::Supporter(p as u8), &[me], me)?;
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    choose_cards(g, p, "CHOOSE_CARD_TO_DISCARD", ListRef::Deck(p as u8), Filter::none(), ChooseCardsOpts::new(1, 5, false), Cont::Card { card: me, frame: f });
    Ok(())
}

fn finish(g: &mut Game, me: CardId, p: usize, cards: &[CardId]) -> R {
    move_cards(g, ListRef::Deck(p as u8), ListRef::Discard(p as u8), cards, me)?;
    let id = g.player_id(p);
    g.prompt(id, "", PromptKind::ShuffleDeck, Cont::ShuffleApplyNoWait { p: p as u8 });
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    match f.stage {
        1 => {
            let cards: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
            if !cards.is_empty() {
                // Keep the chosen cards for after the reveal.
                let temp = g.alloc_temp(&cards);
                let mut nf = CardFrame::at(2);
                nf.a[0] = p as i32;
                nf.l[0] = match temp {
                    ListRef::Temp(i) => i,
                    _ => 0,
                };
                let id = g.player_id(1 - p);
                g.prompt(id, "CARDS_SHOWED_BY_THE_OPPONENT", PromptKind::ShowCards, Cont::Card { card: me, frame: nf });
                return Ok(());
            }
            finish(g, me, p, &cards)
        }
        2 => {
            let cards: Vec<CardId> = g.lst(ListRef::Temp(f.l[0])).to_vec();
            finish(g, me, p, &cards)
        }
        _ => Ok(()),
    }
}
