//! Lt. Surge's Bargain (MEG 120, Supporter): ask your opponent if each player
//! may take a Prize card. If yes, each player takes a Prize card. If no, you
//! draw 4 cards.
//!
//! Twinleaf: throws SUPPORTER_ALREADY_PLAYED when `supporterTurn > 0`; the
//! OPPONENT answers a ConfirmPrompt. Yes: TAKE_X_PRIZES(player, 1) with a
//! callback that runs TAKE_X_PRIZES(opponent, 1) (the callback is skipped when
//! the player has no Prize cards left). No: DRAW_CARDS(player, 4).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "LtSurgesBargainMEGPool", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    if g.st.players[p].supporter_turn > 0 {
        bail!("SUPPORTER_ALREADY_PLAYED");
    }
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    confirmation_prompt(g, 1 - p, "WANT_TO_USE_ABILITY", Cont::Card { card: me, frame: f });
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    match f.stage {
        1 => {
            let yes = results.first().map(|r| r.as_bool()).unwrap_or(false);
            if !yes {
                return draw_cards(g, p, 4);
            }
            // TAKE_X_PRIZES(player, 1, {}, callback)
            let left = g.st.players[p].prize_left() as i32;
            if left <= 0 {
                return Ok(());
            }
            if left <= 1 {
                let pl = &g.st.players[p];
                let first = (0..pl.prize_count).find(|i| !pl.prizes[*i as usize].is_empty());
                if let Some(i) = first {
                    crate::engine::check::take_specific_prizes(g, p, &[i], ListRef::Hand(p as u8), false)?;
                }
                return crate::engine::check::take_x_prizes(g, 1 - p, 1);
            }
            let mut f2 = CardFrame::at(2);
            f2.a[0] = p as i32;
            let id = g.player_id(p);
            g.prompt(
                id,
                "CHOOSE_PRIZE_CARD",
                PromptKind::ChoosePrize { count: 1, blocked: SVec::new(), use_opponent_prizes: false, allow_cancel: false, is_secret: false, destination: None },
                Cont::Card { card: me, frame: f2 },
            );
            Ok(())
        }
        2 => {
            if let Some(Res::Prizes(ix)) = results.first() {
                crate::engine::check::take_specific_prizes(g, p, ix.as_slice(), ListRef::Hand(p as u8), false)?;
            }
            crate::engine::check::take_x_prizes(g, 1 - p, 1)
        }
        _ => Ok(()),
    }
}
