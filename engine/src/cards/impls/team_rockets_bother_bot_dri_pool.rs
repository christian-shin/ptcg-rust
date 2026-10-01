//! Team Rocket's Bother-Bot (DRI 172, Item): turn 1 of your opponent's
//! face-down Prize cards face up and choose a random card from your opponent's
//! hand. Your opponent reveals that card. You may have your opponent switch
//! those cards. (That Prize card remains face up for the rest of the game.)
//!
//! Twinleaf: throws CANNOT_PLAY_THIS_CARD when every non-empty Prize card of
//! the opponent is already face up; the card goes to the supporter zone by
//! hand (preventDefault). A ChoosePrizePrompt (opponent's Prizes, face-down
//! only, face-up ones blocked) picks the Prize; its list gets `faceUpPrize`.
//! An empty opposing hand: ShowCardsPrompt of the Prize card, then the card is
//! discarded. Otherwise `Chance.index(hand.length)` picks the hand card, a
//! ShowCardsPrompt shows Prize + hand card, a ConfirmPrompt asks the player;
//! on yes the Prize card goes to the hand and the hand card into the Prize
//! list (still face up). Finally the item moves supporter -> discard.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "TeamRocketsBotherBotDRIPool", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    let o = 1 - p;
    let pl = &g.st.players[o];
    let mut blocked: SVec<u8, 6> = SVec::new();
    let mut n = 0u8;
    for i in 0..pl.prize_count {
        if pl.prizes[i as usize].is_empty() {
            continue;
        }
        if pl.prize_face_up & (1 << i) != 0 {
            blocked.push(n);
        }
        n += 1;
    }
    if blocked.len() == n as usize {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    g.set_prevent(e, true);
    move_cards(g, ListRef::Hand(p as u8), ListRef::Supporter(p as u8), &[me], me)?;
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    let id = g.player_id(p);
    g.prompt(
        id,
        "CHOOSE_PRIZE_CARD",
        PromptKind::ChoosePrize { count: 1, blocked, use_opponent_prizes: true, allow_cancel: false, is_secret: false, destination: None, face_down_only: true },
        Cont::Card { card: me, frame: f },
    );
    Ok(())
}

fn finish(g: &mut Game, me: CardId, p: usize) -> R {
    move_cards(g, ListRef::Supporter(p as u8), ListRef::Discard(p as u8), &[me], me)
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    let o = 1 - p;
    let first = results.first().copied().unwrap_or(Res::Null);
    match f.stage {
        1 => {
            let idx = match first {
                Res::Prizes(ix) if ix.len() >= 1 => ix.as_slice()[0],
                _ => bail!("INVALID_PROMPT_RESULT"),
            };
            if g.st.players[o].prize_face_up & (1 << idx) != 0 {
                bail!("INVALID_PROMPT_RESULT");
            }
            // That Prize card remains face up for the rest of the game.
            g.st.players[o].prize_face_up |= 1 << idx;
            g.st.players[o].prize_not_secret |= 1 << idx;
            let id = g.player_id(p);
            if g.st.players[o].hand.is_empty() {
                let mut nf = CardFrame::at(4);
                nf.a[0] = p as i32;
                g.prompt(id, "CARDS_SHOWED_BY_THE_OPPONENT", PromptKind::ShowCards, Cont::Card { card: me, frame: nf });
                return Ok(());
            }
            let n = g.st.players[o].hand.len();
            let hand_card = g.st.players[o].hand.as_slice()[g.rng.index(n)];
            let mut nf = CardFrame::at(2);
            nf.a[0] = p as i32;
            nf.a[1] = idx as i32;
            nf.a[2] = hand_card as i32;
            g.prompt(id, "CARDS_SHOWED_BY_THE_OPPONENT", PromptKind::ShowCards, Cont::Card { card: me, frame: nf });
            Ok(())
        }
        2 => {
            let mut nf = CardFrame::at(3);
            nf.a[0] = f.a[0];
            nf.a[1] = f.a[1];
            nf.a[2] = f.a[2];
            confirmation_prompt(g, p, "WANT_TO_USE_ABILITY", Cont::Card { card: me, frame: nf });
            Ok(())
        }
        3 => {
            if first.as_bool() {
                let idx = f.a[1] as u8;
                let hand_card = f.a[2] as CardId;
                let prize = ListRef::Prize(o as u8, idx);
                let prize_cards: Vec<CardId> = g.lst(prize).to_vec();
                move_cards(g, prize, ListRef::Hand(o as u8), &prize_cards, me)?;
                move_cards(g, ListRef::Hand(o as u8), prize, &[hand_card], me)?;
                g.st.players[o].prize_face_up |= 1 << idx;
                g.st.players[o].prize_not_secret |= 1 << idx;
            }
            finish(g, me, p)
        }
        4 => finish(g, me, p),
        _ => Ok(()),
    }
}
