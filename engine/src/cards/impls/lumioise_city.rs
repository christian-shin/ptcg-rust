//! Lumiose City (POR / M3, stadium): once during each player's turn, that
//! player may search their deck for a Basic Pokémon and put it onto their
//! Bench, then shuffle. If they do, their turn ends.
//!
//! Twinleaf: the turn ends (EndTurnEffect) after the shuffle even when no
//! Pokémon was chosen; the shuffle has no animation wait.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "LumioiseCity", mask: mask(&[k::USE_STADIUM]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match *g.e(e) {
        Effect::UseStadium { p, .. } if g.st.stadium_card() == Some(me) => p as usize,
        _ => return Ok(()),
    };
    let slots = empty_bench_slots(g, p);
    if g.st.players[p].deck.is_empty() {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    if slots.is_empty() {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    f.l[0] = slots.as_slice()[0];
    let filter = Filter { super_type: Some(SuperType::Pokemon as u8), stage: Some(Stage::Basic as u8), ..Filter::none() };
    choose_cards(g, p, "CHOOSE_CARD_TO_PUT_ONTO_BENCH", ListRef::Deck(p as u8), filter, ChooseCardsOpts::new(0, 1, false), Cont::Card { card: me, frame: f });
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    match f.stage {
        1 => {
            let cards: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
            for (i, c) in cards.iter().enumerate() {
                if i > 0 {
                    // `slots[index]` beyond the first: max is 1, never reached.
                    break;
                }
                g.run_fx(Effect::PlayPokemonFromDeck { p: p as u8, card: *c, target: SlotRef::new(p, f.l[0]) })?;
            }
            let mut nf = CardFrame::at(2);
            nf.a[0] = p as i32;
            let id = g.player_id(p);
            g.prompt(id, "", PromptKind::ShuffleDeck, Cont::Card { card: me, frame: nf });
            Ok(())
        }
        2 => {
            if let Some(Res::Order(o)) = results.first() {
                crate::game::apply_order(&mut g.st.players[p].deck, o.as_slice());
            }
            g.run_fx(Effect::EndTurn { p: p as u8 })?;
            Ok(())
        }
        _ => Ok(()),
    }
}
