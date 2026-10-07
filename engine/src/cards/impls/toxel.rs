//! Toxel (M2 / PFL 67): Call For Family — search your deck for up to 2 Basic
//! Pokémon and put them onto your Bench, then shuffle. Rascal Kick — 20.
//!
//! Twinleaf: nothing happens (no shuffle) with a full Bench; the empty
//! slots are fixed when the attack starts; the shuffle has no animation wait.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Toxel", mask: mask(&[k::AFTER_ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !after_attack_used(g, e, 0, me) {
        return Ok(());
    }
    let e = real_attack(g, e);
    let p = match *g.e(e) {
        Effect::Attack { p, .. } => p as usize,
        _ => return Ok(()),
    };
    let slots = empty_bench_slots(g, p);
    let max = slots.len().min(2);
    if max == 0 {
        return Ok(());
    }
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    f.a[1] = slots.len() as i32;
    // At most 2 cards are chosen: only the first 2 slots can be used.
    for (i, s) in slots.iter().enumerate().take(2) {
        f.l[i] = *s;
    }
    let filter = Filter { super_type: Some(SuperType::Pokemon as u8), stage: Some(Stage::Basic as u8), ..Filter::none() };
    choose_cards(g, p, "CHOOSE_CARD_TO_PUT_ONTO_BENCH", ListRef::Deck(p as u8), filter, ChooseCardsOpts::new(0, max as u8, false), Cont::Card { card: me, frame: f });
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let n = f.a[1] as usize;
    let mut cards: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
    cards.truncate(n);
    for (i, c) in cards.iter().enumerate() {
        g.run_fx(Effect::PlayPokemonFromDeck { p: p as u8, card: *c, target: SlotRef::new(p, f.l[i]) })?;
    }
    let id = g.player_id(p);
    g.prompt(id, "", PromptKind::ShuffleDeck, Cont::ShuffleApplyNoWait { p: p as u8 });
    Ok(())
}
