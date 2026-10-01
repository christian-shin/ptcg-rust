//! Lampent (PBL / M5): Spreading Light - search your deck for up to 3 Lampent
//! and put them onto your Bench, then shuffle.
//!
//! Twinleaf: max = min(3, empty Bench slots); nothing happens (no shuffle)
//! when the deck is empty or the Bench is full. Each chosen card is placed with
//! a PlayPokemonFromDeckEffect into the i-th empty slot; SHUFFLE_DECK follows.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Lampent", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let slots = empty_bench_slots(g, p);
        let max_put = slots.len().min(3);
        if g.st.players[p].deck.is_empty() || max_put == 0 {
            return Ok(());
        }
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        let filter = Filter { super_type: Some(SuperType::Pokemon as u8), name: Some("Lampent"), ..Default::default() };
        choose_cards(
            g,
            p,
            "CHOOSE_CARD_TO_PUT_ONTO_BENCH",
            ListRef::Deck(p as u8),
            filter,
            ChooseCardsOpts::new(0, max_put as u8, false),
            Cont::Card { card: me, frame: f },
        );
    }
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    // Nothing can change the Bench between the attack and the answer.
    let slots = empty_bench_slots(g, p);
    let first = results.first().copied().unwrap_or(Res::Null);
    let cards: Vec<CardId> = first.cards().to_vec();
    for (i, c) in cards.iter().enumerate() {
        if let Some(&s) = slots.get(i) {
            g.run_fx(Effect::PlayPokemonFromDeck { p: p as u8, card: *c, target: SlotRef::new(p, s) })?;
        }
    }
    shuffle_deck(g, p);
    Ok(())
}
