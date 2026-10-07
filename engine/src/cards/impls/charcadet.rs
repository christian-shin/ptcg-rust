//! Charcadet (M2 / PFL): Gather Power — search your deck for up to 2 Basic
//! Energy cards and put them into your hand. Chop — 10.
//!
//! Fixed (phase 4b): an empty deck does nothing (it threw CANNOT_USE_ATTACK,
//! making the attack unusable); the up-to-2 choice (cancel not allowed) is
//! moved to hand, revealed to the opponent, and the deck is shuffled (both
//! were missing).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Charcadet@Charcadet M2", mask: mask(&[k::AFTER_ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if after_attack_used(g, e, 0, me) {
        let e = real_attack(g, e);
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        if g.st.players[p].deck.is_empty() {
            return Ok(());
        }
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        let filter = Filter { super_type: Some(SuperType::Energy as u8), energy_type: Some(EnergyType::Basic as u8), ..Filter::none() };
        choose_cards(g, p, "CHOOSE_CARD_TO_HAND", ListRef::Deck(p as u8), filter, ChooseCardsOpts::new(0, 2, false), Cont::Card { card: me, frame: f });
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let cards: Vec<CardId> = results.first().copied().unwrap_or(Res::Null).cards().to_vec();
    if !cards.is_empty() {
        move_cards(g, ListRef::Deck(p as u8), ListRef::Hand(p as u8), &cards, me)?;
        let id = g.player_id(1 - p);
        g.prompt(id, "CARDS_SHOWED_BY_THE_OPPONENT", PromptKind::ShowCards, Cont::Noop);
    }
    shuffle_deck(g, p);
    Ok(())
}
