//! Kofu (SCR, supporter): put 2 cards from your hand on the bottom of your
//! deck in any order, then draw 4 cards.
//!
//! Twinleaf: the hand cards are chosen from `player.hand` (2 required, no
//! cancel); the core TrainerEffect reducer has already moved the Supporter to
//! the supporter pile when the prompt is answered, so Kofu itself is never a
//! pick (phase 4b #43: checked in corpus traces, not a bug). After the
//! order prompt the cards go to the deck bottom and `min(4, deck size)` cards
//! are moved to the hand (no shuffle, no supporter-turn marker).
//!
//! R7C: counts the cards other than Kofu in the hand (as the effect of an attack Kofu is
//! not in the hand).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Kofu", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    if g.st.players[p].supporter_turn > 0 {
        bail!("SUPPORTER_ALREADY_PLAYED");
    }
    // 2 cards other than this one (used as the effect of an attack it isn't in the hand).
    if g.st.players[p].hand.iter().filter(|c| *c != me).count() < 2 {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    let bottom = g.alloc_temp(&[]);
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    f.l[0] = match bottom {
        ListRef::Temp(i) => i,
        _ => 0,
    };
    choose_cards(g, p, "CHOOSE_CARDS_TO_PUT_ON_BOTTOM_OF_THE_DECK", ListRef::Hand(p as u8), Filter::none(), ChooseCardsOpts::new(2, 2, false), Cont::Card { card: me, frame: f });
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    let bottom = ListRef::Temp(f.l[0]);
    let first = results.first().copied().unwrap_or(Res::Null);
    match f.stage {
        1 => {
            let cards: Vec<CardId> = first.cards().to_vec();
            move_cards(g, ListRef::Hand(p as u8), bottom, &cards, me)?;
            let id = g.player_id(p);
            let mut nf = f;
            nf.stage = 2;
            g.prompt(id, "CHOOSE_CARDS_ORDER", PromptKind::OrderCards { cards: bottom, allow_cancel: false }, Cont::Card { card: me, frame: nf });
            Ok(())
        }
        2 => {
            let o = match first {
                Res::Order(o) => o,
                _ => return Ok(()),
            };
            crate::game::apply_order(&mut g.temps[f.l[0] as usize], o.as_slice());
            g.run_fx(Effect::MoveCards {
                source: bottom,
                destination: ListRef::Deck(p as u8),
                cards: None,
                count: None,
                to_top: false,
                to_bottom: false,
                skip_cleanup: false,
                source_card: me,
            })?;
            let n = g.st.players[p].deck.len().min(4);
            move_count_from(g, ListRef::Deck(p as u8), ListRef::Hand(p as u8), n, me)
        }
        _ => Ok(()),
    }
}
