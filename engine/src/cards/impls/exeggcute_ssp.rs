//! Exeggcute (SSP 1): Precocious Evolution (usable on the first turn) —
//! search your deck for a card that evolves from this Pokémon and put it
//! onto this Pokémon to evolve it, then shuffle.
//!
//! Two `Exeggcute` classes exist; this port is bound to SSP. Twinleaf: a
//! non-cancellable ChooseCardsPrompt (min 0, max 1); an empty deck skips everything (no shuffle); the evolution is a
//! plain MOVE_CARDS onto the player's Active followed by `clearEffects()` and
//! `pokemonPlayedTurn = turn` (no EvolveEffect); the shuffle has no wait.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Exeggcute@SSP", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !was_attack_used(g, e, 0, me) {
        return Ok(());
    }
    let (p, source) = match *g.e(e) {
        Effect::Attack { p, source, .. } => (p as usize, source),
        _ => return Ok(()),
    };
    if g.st.players[p].deck.is_empty() {
        return Ok(());
    }
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    f.l[0] = source.s;
    let filter = Filter { super_type: Some(SuperType::Pokemon as u8), evolves_from: Some("Exeggcute"), ..Filter::none() };
    choose_cards(g, p, "CHOOSE_CARD_TO_EVOLVE", ListRef::Deck(p as u8), filter, ChooseCardsOpts::new(0, 1, false), Cont::Card { card: me, frame: f });
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let cards: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
    if !cards.is_empty() {
        let source_card = g.st.slot_pokemon(p, f.l[0]).unwrap_or(NO_CARD);
        let a = g.st.players[p].active;
        move_cards(g, ListRef::Deck(p as u8), ListRef::Slot(p as u8, a), &cards, source_card)?;
        let a = g.st.players[p].active;
        let turn = g.st.turn;
        let slot = &mut g.st.players[p].slots[a as usize];
        crate::engine::game_effect::clear_effects(slot);
        slot.pokemon_played_turn = turn;
    }
    let id = g.player_id(p);
    g.prompt(id, "", PromptKind::ShuffleDeck, Cont::ShuffleApplyNoWait { p: p as u8 });
    Ok(())
}
