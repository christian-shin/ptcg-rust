//! Team Rocket's Proton (DRI): usable on the first turn if you go first.
//! Search your deck for up to 3 Basic Team Rocket's Pokémon, reveal them,
//! put them into your hand, then shuffle.
//!
//! Twinleaf: sets `rocketSupporter`; the prompt message is
//! CHOOSE_CARD_TO_PUT_ONTO_BENCH; non-Team Rocket cards are blocked (by
//! first index), the filter is Basic Pokémon.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "TeamRocketsProton", mask: mask(&[k::TRAINER, k::END_TURN]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Some(p) = trainer_played(g, e, me) {
        if g.st.players[p].supporter_turn > 0 {
            bail!("SUPPORTER_ALREADY_PLAYED");
        }
        if g.st.players[p].deck.is_empty() {
            bail!("CANNOT_PLAY_THIS_CARD");
        }
        g.st.players[p].rocket_supporter = true;
        g.set_prevent(e, true);
        move_cards(g, ListRef::Hand(p as u8), ListRef::Supporter(p as u8), &[me], me)?;
        let mut opts = ChooseCardsOpts::new(0, 3, false);
        for (i, c) in g.st.players[p].deck.iter().enumerate() {
            if !g.st.cdef(c).has_tag(tag::TEAM_ROCKET) {
                opts.blocked.push(i as u8);
            }
        }
        let filter = Filter { super_type: Some(SuperType::Pokemon as u8), stage: Some(Stage::Basic as u8), ..Default::default() };
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        choose_cards(g, p, "CHOOSE_CARD_TO_PUT_ONTO_BENCH", ListRef::Deck(p as u8), filter, opts, Cont::Card { card: me, frame: f });
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
