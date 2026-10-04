//! Accompanying Flute (TWM): reveal the top 5 cards of your opponent's deck,
//! and put any number of Basic Pokémon you find there onto your opponent's
//! Bench. Then, they shuffle the remaining cards back into their deck.
//!
//! Twinleaf quirks kept: the chosen cards are moved straight into the empty
//! Bench slots (MOVE_CARDS, no PlayPokemon effect) with `pokemonPlayedTurn`
//! set; with nothing chosen the opponent is shown the cards, they go back to
//! the deck, and the opponent's deck is shuffled. Neither shuffle has a
//! trailing wait.
//!
//! Fixed (phase 4b, W4): with nothing chosen Twinleaf shuffled the player's own
//! deck and left the opponent's top 5 cards at the bottom, in order.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "PerformanceFlute", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    let o = 1 - p;
    let open = empty_bench_slots(g, o);
    if open.is_empty() {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    g.set_prevent(e, true);
    if g.st.players[o].deck.is_empty() {
        bail!("CANNOT_USE_POWER");
    }
    let top = g.alloc_temp(&[]);
    move_count_from(g, ListRef::Deck(o as u8), top, 5, me)?;
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    f.l[0] = match top {
        ListRef::Temp(i) => i,
        _ => 0,
    };
    // Empty slots at prompt time, packed 4 bits each.
    let mut packed: u32 = 0;
    for (i, s) in open.iter().enumerate() {
        packed |= (*s as u32) << (4 * i);
    }
    f.a[1] = packed as i32;
    f.a[2] = open.len() as i32;
    let filter = Filter { super_type: Some(SuperType::Pokemon as u8), stage: Some(Stage::Basic as u8), ..Filter::none() };
    let id = g.player_id(p);
    g.prompt(
        id,
        "CHOOSE_CARD_TO_HAND",
        PromptKind::ChooseCards { cards: top, filter, opts: ChooseCardsOpts::new(0, open.len() as u8, false) },
        Cont::Card { card: me, frame: f },
    );
    Ok(())
}

fn move_all(g: &mut Game, src: ListRef, dst: ListRef, me: CardId) -> R {
    g.run_fx(Effect::MoveCards { source: src, destination: dst, cards: None, count: None, to_top: false, to_bottom: false, skip_cleanup: false, source_card: me })?;
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    let o = 1 - p;
    let top = ListRef::Temp(f.l[0]);
    match f.stage {
        1 => {
            let cards: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
            if cards.is_empty() {
                let id = g.player_id(o);
                g.prompt(id, "CARDS_SHOWED_BY_THE_OPPONENT", PromptKind::ShowCards, Cont::Card { card: me, frame: CardFrame { stage: 2, ..f } });
                return Ok(());
            }
            let n = f.a[2] as usize;
            for (i, c) in cards.iter().enumerate() {
                if i >= n {
                    // `slots[index]` undefined: MOVE_CARDS throws on it.
                    bail!("TypeError: Cannot read properties of undefined");
                }
                let s = (((f.a[1] as u32) >> (4 * i)) & 0xF) as SlotId;
                move_cards(g, top, ListRef::Slot(o as u8, s), &[*c], me)?;
                let turn = g.st.turn;
                g.st.players[o].slots[s as usize].pokemon_played_turn = turn;
            }
            move_all(g, top, ListRef::Deck(o as u8), me)?;
            let id = g.player_id(o);
            g.prompt(id, "", PromptKind::ShuffleDeck, Cont::ShuffleApplyNoWait { p: o as u8 });
            Ok(())
        }
        2 => {
            move_all(g, top, ListRef::Deck(o as u8), me)?;
            let id = g.player_id(o);
            g.prompt(id, "", PromptKind::ShuffleDeck, Cont::ShuffleApplyNoWait { p: o as u8 });
            Ok(())
        }
        _ => Ok(()),
    }
}
