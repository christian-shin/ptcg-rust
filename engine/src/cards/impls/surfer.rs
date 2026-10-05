//! Surfer (SSP / ASC): switch your Active Pokémon with 1 of your Benched
//! Pokémon; if you do, draw cards until you have 5 cards in your hand.
//!
//! Twinleaf: throws SUPPORTER_ALREADY_PLAYED when a Supporter was played;
//! fixed (phase 4b): throws CANNOT_PLAY_THIS_CARD with an empty Bench (the
//! prompt would have no valid answer); moves the card to the supporter list
//! itself and prevents the default;
//! the ChoosePokemonPrompt (no cancel)
//! is followed by `player.switchPokemon(cardList, store, state)` (fixed in
//! phase 4b, R4: it was the silent form without the move effects: Yanmega ex
//! Buzz Boost, Palafin Zero to Hero and the ability-lock order never saw the
//! switch) and a loop of
//! single-card MOVE_CARDS (count 1) until the hand has 5 cards or the deck
//! is empty.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Surfer", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Some(p) = trainer_played(g, e, me) {
        if g.st.players[p].supporter_turn > 0 {
            bail!("SUPPORTER_ALREADY_PLAYED");
        }
        let has_bench = g.st.players[p].bench.iter().any(|&b| !g.st.players[p].slots[b as usize].cards.is_empty());
        if !has_bench {
            bail!("CANNOT_PLAY_THIS_CARD");
        }
        move_cards(g, ListRef::Hand(p as u8), ListRef::Supporter(p as u8), &[me], me)?;
        g.set_prevent(e, true);
        let mut slots = SVec::new();
        slots.push(SlotType::Bench as u8);
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        let id = g.player_id(p);
        g.prompt(
            id,
            "CHOOSE_POKEMON_TO_SWITCH",
            PromptKind::ChoosePokemon { player_type: PlayerType::BottomPlayer, slots, min: 1, max: 1, allow_cancel: false, blocked: SVec::new() },
            Cont::Card { card: me, frame: f },
        );
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let t = match results.first().and_then(|r| r.slots().first().copied()) {
        Some(t) => t,
        // `result[0]` of a cancelled prompt: TypeError.
        None => bail!("TypeError: Cannot read properties of null"),
    };
    if t.p as usize == p {
        crate::engine::turn::switch_pokemon(g, p, t.s)?;
    }
    while g.st.players[p].hand.len() < 5 {
        if g.st.players[p].deck.is_empty() {
            break;
        }
        move_count_from(g, ListRef::Deck(p as u8), ListRef::Hand(p as u8), 1, me)?;
    }
    Ok(())
}
