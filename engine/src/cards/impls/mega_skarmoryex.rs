//! Mega Skarmory ex (POR / M3): Sonic Ripper — shuffle all Energy from this
//! Pokémon into your deck; 220 damage to 1 of your opponent's Pokémon (no
//! Weakness/Resistance for the Bench).
//!
//! Fixed (W1-A): the Energy used to be moved out of the Active's `energies`
//! list only (the cards stayed in the slot's `cards` too, so they were
//! duplicated into the deck); it is now moved from the slot itself.
//! R7A (ruling 1580): the Energy goes back into the deck, and the deck is shuffled, after the damage (`move_cards_after_damage`,
//! `shuffle_deck_after_damage`); the shuffle uses SHUFFLE_DECK (with its animation wait).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "MegaSkarmoryex", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !was_attack_used(g, e, 0, me) {
        return Ok(());
    }
    let p = match *g.e(e) {
        Effect::Attack { p, .. } => p as usize,
        _ => return Ok(()),
    };
    let a = g.st.players[p].active;
    let energies: Vec<CardId> = g.st.players[p].slots[a as usize].energies.iter().collect();
    if !energies.is_empty() {
        move_cards_after_damage(g, e, ListRef::Slot(p as u8, a), ListRef::Deck(p as u8), &energies, me)?;
    }
    shuffle_deck_after_damage(g, e, p);
    let id = g.player_id(p);
    let mut slots = SVec::new();
    slots.push(SlotType::Active as u8);
    slots.push(SlotType::Bench as u8);
    g.retain_fx(e);
    let mut f = CardFrame::at(1);
    f.e[0] = e;
    g.prompt(
        id,
        "CHOOSE_POKEMON_TO_DAMAGE",
        PromptKind::ChoosePokemon { player_type: PlayerType::TopPlayer, slots, min: 1, max: 1, allow_cancel: false, blocked: SVec::new() },
        Cont::Card { card: me, frame: f },
    );
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let atk = f.e[0];
    let targets: Vec<SlotRef> = results.first().map(|r| r.slots().to_vec()).unwrap_or_default();
    let r = damage_opponent_pokemon(g, atk, 220, &targets);
    g.release_fx(atk);
    r
}
