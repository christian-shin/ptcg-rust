//! Eevee (SCR): Call for Family — search your deck for a Basic Pokémon and
//! put it onto your Bench, then shuffle. Gnaw — 20. Ported so Umbreon ex
//! (PRE) can evolve in check decks.
//!
//! Twinleaf: nothing happens (no shuffle) with a full Bench; otherwise a
//! cancellable ChooseCardsPrompt (Basic Pokémon, 0..1), a
//! PlayPokemonFromDeckEffect, then a ShuffleDeckPrompt with no wait.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Eevee@SCR", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !was_attack_used(g, e, 0, me) {
        return Ok(());
    }
    let p = match *g.e(e) {
        Effect::Attack { p, .. } => p as usize,
        _ => return Ok(()),
    };
    let slots = empty_bench_slots(g, p);
    if slots.is_empty() {
        return Ok(());
    }
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    f.l[0] = slots.as_slice()[0];
    let filter = Filter { super_type: Some(SuperType::Pokemon as u8), stage: Some(Stage::Basic as u8), ..Filter::none() };
    choose_cards(g, p, "CHOOSE_CARD_TO_PUT_ONTO_BENCH", ListRef::Deck(p as u8), filter, ChooseCardsOpts::new(0, 1, true), Cont::Card { card: me, frame: f });
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let cards: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
    if let Some(c) = cards.first() {
        g.run_fx(Effect::PlayPokemonFromDeck { p: p as u8, card: *c, target: SlotRef::new(p, f.l[0]) })?;
    }
    let id = g.player_id(p);
    g.prompt(id, "", PromptKind::ShuffleDeck, Cont::ShuffleApplyNoWait { p: p as u8 });
    Ok(())
}
