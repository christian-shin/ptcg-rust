//! Repel (SUM / MEG): your opponent switches their Active Pokémon with 1 of
//! their Benched Pokémon.
//!
//! Twinleaf: throws CANNOT_PLAY_THIS_CARD without an opposing Bench, then
//! SWITCH_OUT_OPPONENT_ACTIVE_POKEMON (no sourceEffect, no cancel): the
//! *opponent* answers a ChoosePokemonPrompt over their own Bench and
//! `opponent.switchPokemon(selected[0], store, state)` runs in the callback.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Repel", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Some(p) = trainer_played(g, e, me) {
        let o = 1 - p;
        let pl = &g.st.players[o];
        if !pl.bench.iter().any(|b| !pl.slots[*b as usize].cards.is_empty()) {
            bail!("CANNOT_PLAY_THIS_CARD");
        }
        let mut slots = SVec::new();
        slots.push(SlotType::Bench as u8);
        let mut f = CardFrame::at(1);
        f.a[0] = o as i32;
        let id = g.player_id(o);
        g.prompt(
            id,
            "CHOOSE_POKEMON_TO_SWITCH",
            PromptKind::ChoosePokemon { player_type: PlayerType::BottomPlayer, slots, min: 1, max: 1, allow_cancel: false, blocked: SVec::new() },
            Cont::Card { card: me, frame: f },
        );
    }
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let o = f.a[0] as usize;
    let sel = results.first().map(|r| r.slots().to_vec()).unwrap_or_default();
    if sel.is_empty() {
        return Ok(());
    }
    if sel[0].p as usize == o {
        crate::engine::turn::switch_pokemon(g, o, sel[0].s)?;
    }
    Ok(())
}
