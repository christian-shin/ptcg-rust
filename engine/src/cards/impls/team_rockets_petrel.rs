//! Team Rocket's Petrel (DRI): search your deck for a Trainer card, reveal
//! it, put it into your hand, then shuffle.
//!
//! Twinleaf sets `player.rocketSupporter` (read by Team Rocket's Factory)
//! before the search; every Petrel copy clears it at the end of that
//! player's turn.
//!
//! R7C: `rocket_supporter` is not set when used as the effect of an attack (ruling 1727).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "TeamRocketsPetrel", mask: mask(&[k::TRAINER, k::END_TURN]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Some(p) = trainer_played(g, e, me) {
        if g.st.players[p].supporter_turn > 0 {
            bail!("SUPPORTER_ALREADY_PLAYED");
        }
        if g.st.players[p].deck.is_empty() {
            bail!("CANNOT_PLAY_THIS_CARD");
        }
        // Using the effect of a Supporter as the effect of an attack is not playing it from the hand.
        if !trainer_via_attack(g, e) {
            g.st.players[p].rocket_supporter = true;
        }
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        choose_cards(
            g,
            p,
            "CHOOSE_CARD_TO_HAND",
            ListRef::Deck(p as u8),
            Filter::super_type(SuperType::Trainer),
            ChooseCardsOpts::new(0, 1, false),
            Cont::Card { card: me, frame: f },
        );
        return Ok(());
    }

    if let Effect::EndTurn { p } = *g.e(e) {
        if g.st.players[p as usize].rocket_supporter {
            g.st.players[p as usize].rocket_supporter = false;
        }
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let cards: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
    move_cards(g, ListRef::Deck(p as u8), ListRef::Hand(p as u8), &cards, me)?;
    show_cards_to_player(g, 1 - p, cards.len());
    shuffle_deck(g, p);
    Ok(())
}
