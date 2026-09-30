//! Farfetch'd (TWM): Impromptu Carrier — when you put this card from your
//! hand onto your Bench, you may search your deck for a Pokémon Tool and
//! attach it to this Pokémon, then shuffle. Mach Cut — 30 (text not
//! implemented in Twinleaf).
//!
//! Twinleaf: the prompt is created during PlayPokemonEffect propagation
//! (before the card is benched); the callback finds this card's bench index
//! (0 if not found), MOVE_CARDS the Tool there and pushes it onto `tools`
//! directly (no AttachPokemonToolEffect, no max-tools check), then shuffles
//! with no trailing wait.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Farfetchd", mask: mask(&[k::PLAY_POKEMON]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::PlayPokemon { card, target, .. } = *g.e(e) {
        if card != me {
            return Ok(());
        }
        let p = target.p as usize;
        if is_ability_blocked(g, p, me, None) {
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
            g.st.players[p].slots[s as usize].tools.push(c);
        }
    }
    let id = g.player_id(p);
    g.prompt(id, "", PromptKind::ShuffleDeck, Cont::ShuffleApplyNoWait { p: p as u8 });
    Ok(())
}
