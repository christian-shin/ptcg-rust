//! Chatot (TEF): A Capella — search your deck for up to 3 Basic Pokémon and
//! put them onto your Bench, then shuffle. Gust — 20.
//!
//! Twinleaf: with no empty Bench slot the attack does nothing, with no search
//! and no shuffle (phase 4b R7E, ruling 337; it used to open a prompt with max
//! 0 and shuffle); no empty-deck check; the ShuffleDeckPrompt has no trailing
//! wait.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Chatot", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        if empty_bench_slots(g, p).is_empty() {
            return Ok(());
        }
        bench_search(g, me, p, 3, false);
    }
    Ok(())
}

/// Shared by Chatot (TEF) and Eevee (SCR): ChooseCardsPrompt for up to
/// `min(empty slots, limit)` Basic Pokémon, then PlayPokemonFromDeck each and shuffle.
pub fn bench_search(g: &mut Game, me: CardId, p: usize, limit: usize, allow_cancel: bool) {
    let open = empty_bench_slots(g, p);
    let max = open.len().min(limit) as u8;
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    for (i, s) in open.iter().enumerate().take(3) {
        f.a[1 + i] = *s as i32;
    }
    f.l[0] = open.len() as u8;
    let filter = Filter { super_type: Some(SuperType::Pokemon as u8), stage: Some(Stage::Basic as u8), ..Filter::none() };
    choose_cards(g, p, "CHOOSE_CARD_TO_PUT_ONTO_BENCH", ListRef::Deck(p as u8), filter, ChooseCardsOpts::new(0, max, allow_cancel), Cont::Card { card: me, frame: f });
}

pub fn bench_search_resume(g: &mut Game, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let cards: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
    let n = cards.len().min(f.l[0] as usize).min(3);
    for (i, c) in cards.iter().take(n).enumerate() {
        let s = f.a[1 + i] as SlotId;
        g.run_fx(Effect::PlayPokemonFromDeck { p: p as u8, card: *c, target: SlotRef::new(p, s) })?;
    }
    let id = g.player_id(p);
    g.prompt(id, "", PromptKind::ShuffleDeck, Cont::ShuffleApplyNoWait { p: p as u8 });
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    bench_search_resume(g, f, results)
}
