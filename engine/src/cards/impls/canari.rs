//! Canari (ASC / M2a): discard another card from your hand; search your deck
//! for up to 4 [L] Pokémon, reveal them, put them into your hand, shuffle.
//!
//! Twinleaf: `playedCanari` is set on play (before any check) and cleared on
//! every EndTurnEffect for that player; the discard prompt runs on a copy of
//! the hand; the deck prompt blocks everything but [L] Pokémon (max 4 or the
//! number present); ShowCards only when something was taken; the final
//! shuffle prompt has no wait.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Canari", mask: mask(&[k::TRAINER, k::END_TURN]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Some(p) = trainer_played(g, e, me) {
        g.st.players[p].played_canari = true;
        if g.st.players[p].supporter_turn > 0 {
            bail!("SUPPORTER_ALREADY_PLAYED");
        }
        let others: Vec<CardId> = g.st.players[p].hand.iter().filter(|c| *c != me).collect();
        if others.is_empty() {
            bail!("CANNOT_PLAY_THIS_CARD");
        }
        g.set_prevent(e, true);
        move_cards(g, ListRef::Hand(p as u8), ListRef::Supporter(p as u8), &[me], me)?;
        let others: Vec<CardId> = g.st.players[p].hand.iter().filter(|c| *c != me).collect();
        let temp = g.alloc_temp(&others);
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        choose_cards(g, p, "CHOOSE_CARD_TO_DISCARD", temp, Filter::none(), ChooseCardsOpts::new(1, 1, false), Cont::Card { card: me, frame: f });
        return Ok(());
    }
    if let Effect::EndTurn { p } = *g.e(e) {
        g.st.players[p as usize].played_canari = false;
    }
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
            let mut n = 0u8;
            let mut opts = ChooseCardsOpts::new(0, 0, false);
            for (i, c) in g.st.players[p].deck.iter().enumerate() {
                let d = g.st.cdef(c);
                if d.is_pokemon() && d.card_type.contains(&ct::LIGHTNING) {
                    n += 1;
                } else {
                    opts.blocked.push(i as u8);
                }
            }
            opts.max = n.min(4);
            let mut nf = CardFrame::at(2);
            nf.a[0] = p as i32;
            choose_cards(g, p, "CHOOSE_CARD_TO_HAND", ListRef::Deck(p as u8), Filter::none(), opts, Cont::Card { card: me, frame: nf });
            Ok(())
        }
        2 => {
            let cards: Vec<CardId> = first.cards().to_vec();
            move_cards(g, ListRef::Deck(p as u8), ListRef::Hand(p as u8), &cards, me)?;
            if !cards.is_empty() {
                let mut nf = CardFrame::at(3);
                nf.a[0] = p as i32;
                let id = g.player_id(1 - p);
                g.prompt(id, "CARDS_SHOWED_BY_THE_OPPONENT", PromptKind::ShowCards, Cont::Card { card: me, frame: nf });
                return Ok(());
            }
            shuffle_nowait(g, p);
            Ok(())
        }
        3 => {
            shuffle_nowait(g, p);
            Ok(())
        }
        _ => Ok(()),
    }
}

fn shuffle_nowait(g: &mut Game, p: usize) {
    let id = g.player_id(p);
    g.prompt(id, "", PromptKind::ShuffleDeck, Cont::ShuffleApplyNoWait { p: p as u8 });
}
