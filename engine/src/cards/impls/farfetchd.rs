//! Farfetch'd (TWM): Impromptu Carrier — when you put this card from your
//! hand onto your Bench, you may search your deck for a Pokémon Tool and
//! attach it to this Pokémon, then shuffle. Mach Cut — 30; discard a Special
//! Energy from your opponent's Active Pokémon (fixed in R1-8: Twinleaf had no
//! handler for it; now the attacker picks one Special Energy attached to the
//! opponent's Active with a ChooseCardsPrompt (min 1, max 1, no cancel,
//! nothing when there is none) and a DiscardCardsEffect follows, like
//! Hawlucha FST's Flying Stomp).
//!
//! Twinleaf: the prompt is created during PlayPokemonEffect propagation
//! (before the card is benched); the callback finds this card's bench index
//! (0 if not found), MOVE_CARDS the Tool there and pushes it onto `tools`
//! directly (no AttachPokemonToolEffect, no max-tools check), then shuffles
//! with no trailing wait. Fixed (W1-C): MOVE_CARDS also left the Tool in the
//! slot's `cards`, so it was in two places; it is now taken out of `cards`
//! (a Tool lives in `tools` only).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Farfetchd", mask: mask(&[k::PLAY_POKEMON, k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    // Mach Cut
    if was_attack_used(g, e, 0, me) {
        let (p, o) = match *g.e(e) {
            Effect::Attack { p, opp, .. } => (p as usize, opp as usize),
            _ => return Ok(()),
        };
        let a = g.st.players[o].active;
        let has_special = g.st.slot(o, a).cards.iter().any(|c| {
            let d = g.st.cdef(c);
            d.is_energy() && d.energy_type == EnergyType::Special as u8
        });
        if !has_special {
            return Ok(());
        }
        g.retain_fx(e);
        let mut f = CardFrame::at(2);
        f.e[0] = e;
        f.l[0] = o as u8;
        f.l[1] = a;
        let mut filter = Filter::super_type(SuperType::Energy);
        filter.energy_type = Some(EnergyType::Special as u8);
        choose_cards(g, p, "CHOOSE_CARD_TO_DISCARD", ListRef::Slot(o as u8, a), filter, ChooseCardsOpts::new(1, 1, false), Cont::Card { card: me, frame: f });
        return Ok(());
    }
    if let Effect::PlayPokemon { card, target, .. } = *g.e(e) {
        if card != me {
            return Ok(());
        }
        let p = target.p as usize;
        if is_ability_blocked(g, p, me, None) {
            return Ok(());
        }
        // An Ability can't be used for no effect: the number of cards in a deck is public (Advanced Rulebook E-06,
        // rulings 244, 782).
        if g.st.players[p].deck.is_empty() {
            return Ok(());
        }
        let filter = Filter { super_type: Some(SuperType::Trainer as u8), trainer_type: Some(TrainerType::Tool as u8), ..Filter::none() };
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        choose_cards(g, p, "CHOOSE_CARD_TO_HAND", ListRef::Deck(p as u8), filter, ChooseCardsOpts::new(0, 1, false), Cont::Card { card: me, frame: f });
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage == 2 {
        return super::trubbish::discard_chosen(g, f, results);
    }
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let first = results.first().copied().unwrap_or(Res::Null);
    let mut bench_idx = 0usize;
    for (i, &b) in g.st.players[p].bench.iter().enumerate() {
        if g.st.slot_pokemon(p, b) == Some(me) {
            bench_idx = i;
        }
    }
    if let Some(&c) = first.cards().first() {
        if g.st.cdef(c).is_trainer() {
            let s = g.st.players[p].bench.as_slice()[bench_idx];
            move_cards(g, ListRef::Deck(p as u8), ListRef::Slot(p as u8, s), &[c], me)?;
            g.st.players[p].slots[s as usize].cards.remove(c);
            g.st.players[p].slots[s as usize].tools.push(c);
        }
    }
    let id = g.player_id(p);
    g.prompt(id, "", PromptKind::ShuffleDeck, Cont::ShuffleApplyNoWait { p: p as u8 });
    Ok(())
}
