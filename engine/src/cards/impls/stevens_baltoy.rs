//! Steven's Baltoy (DRI): Summoning Sign — search your deck for up to 2 Basic
//! Steven's Pokémon and put them onto your Bench. Then, shuffle your deck.
//! Psychic Sphere — 20.
//!
//! Twinleaf: no-op without a free Bench slot; otherwise a ChooseCardsPrompt on
//! the deck (Basic Pokémon, 0..min(slots, 2), no cancel) with every non-Steven's
//! card blocked by deck index; the callback plays each card into the slot it
//! saw at prompt time, then SHUFFLE_DECK (even when nothing was chosen).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "StevensBaltoy", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let slots = empty_bench_slots(g, p);
        if slots.is_empty() {
            return Ok(());
        }
        let filter = Filter { super_type: Some(SuperType::Pokemon as u8), stage: Some(Stage::Basic as u8), ..Filter::none() };
        let mut opts = ChooseCardsOpts::new(0, slots.len().min(2) as u8, false);
        for (i, c) in g.st.players[p].deck.iter().enumerate() {
            if !g.st.cdef(c).has_tag(tag::STEVENS) {
                opts.blocked.push(i as u8);
            }
        }
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        f.l[0] = *slots.get(0).unwrap();
        if slots.len() > 1 {
            f.l[1] = *slots.get(1).unwrap();
        }
        choose_cards(g, p, "CHOOSE_CARD_TO_PUT_ONTO_BENCH", ListRef::Deck(p as u8), filter, opts, Cont::Card { card: me, frame: f });
    }
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let cards: Vec<CardId> = results.first().copied().unwrap_or(Res::Null).cards().to_vec();
    for (i, c) in cards.iter().enumerate() {
        let s = match f.l.get(i) {
            Some(s) => *s,
            None => bail!("TypeError: Cannot read properties of undefined"),
        };
        g.run_fx(Effect::PlayPokemonFromDeck { p: p as u8, card: *c, target: SlotRef::new(p, s) })?;
    }
    shuffle_deck(g, p);
    Ok(())
}
