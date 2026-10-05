//! Explorer's Guidance (TEF, Ancient Supporter): look at the top 6 cards of
//! your deck and put 2 of them into your hand. Discard the other cards.
//!
//! Twinleaf: any EndTurnEffect for a player with `ancientSupporter` clears
//! it. On play: throws when a Supporter was already played; the card moves
//! to the Supporter area and the play is prevented; throws on an empty deck
//! (after that move); 6 cards go to a temporary list; the non-cancellable
//! ChooseCardsPrompt takes `min = min(2, looked at)` (fixed in phase 4b, R4:
//! it was 1 when the deck had at most 1 card left after taking the 6) and up
//! to 2; the callback sets
//! `ancientSupporter`, moves the chosen cards to the hand and the rest to the
//! discard pile.
//!
//! R7C: `ancient_supporter` is not set when used as the effect of an attack (ruling 1727).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "ExplorersGuidance", mask: mask(&[k::END_TURN, k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn move_all(g: &mut Game, src: ListRef, dst: ListRef, me: CardId) -> R {
    g.run_fx(Effect::MoveCards { source: src, destination: dst, cards: None, count: None, to_top: false, to_bottom: false, skip_cleanup: false, source_card: me })?;
    Ok(())
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::EndTurn { p } = *g.e(e) {
        if g.st.players[p as usize].ancient_supporter {
            g.st.players[p as usize].ancient_supporter = false;
        }
    }

    if let Some(p) = trainer_played(g, e, me) {
        if g.st.players[p].supporter_turn > 0 {
            bail!("SUPPORTER_ALREADY_PLAYED");
        }
        move_cards(g, ListRef::Hand(p as u8), ListRef::Supporter(p as u8), &[me], me)?;
        g.set_prevent(e, true);
        if g.st.players[p].deck.is_empty() {
            bail!("CANNOT_PLAY_THIS_CARD");
        }
        let temp = g.alloc_temp(&[]);
        move_count_from(g, ListRef::Deck(p as u8), temp, 6, me)?;
        let looked = g.lst(temp).len();
        let min = looked.min(2);
        let t = match temp {
            ListRef::Temp(i) => i,
            _ => unreachable!(),
        };
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        f.a[1] = t as i32;
        f.a[2] = trainer_via_attack(g, e) as i32;
        choose_cards(g, p, "CHOOSE_CARD_TO_HAND", temp, Filter::none(), ChooseCardsOpts::new(min as u8, 2, false), Cont::Card { card: me, frame: f });
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let temp = ListRef::Temp(f.a[1] as u8);
    let chosen: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
    // Using the effect of a Supporter as the effect of an attack is not playing it from the hand.
    if f.a[2] == 0 {
        g.st.players[p].ancient_supporter = true;
    }
    move_cards(g, temp, ListRef::Hand(p as u8), &chosen, me)?;
    move_all(g, temp, ListRef::Discard(p as u8), me)
}
