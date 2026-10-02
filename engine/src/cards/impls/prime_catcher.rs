//! Prime Catcher (TEF): switch in 1 of your opponent's Benched Pokémon to the
//! Active Spot. If you do, switch your Active Pokémon with 1 of your Benched
//! Pokémon.
//!
//! Twinleaf: with an empty opposing Bench the play fails (undefined state).
use crate::cards::prelude::*;
use crate::engine::game_effect::clear_effects;
use crate::engine::turn::switch_pokemon_silent;

pub static IMPL: CardImpl = CardImpl { class: "PrimeCatcher", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn has_bench(g: &Game, p: usize) -> bool {
    let pl = &g.st.players[p];
    pl.bench.iter().any(|b| !pl.slots[*b as usize].cards.is_empty())
}

fn bench_prompt(g: &mut Game, me: CardId, p: usize, pt: PlayerType, stage: u8) {
    let mut slots = SVec::new();
    slots.push(SlotType::Bench as u8);
    let id = g.player_id(p);
    let mut f = CardFrame::at(stage);
    f.a[0] = p as i32;
    g.prompt(
        id,
        "CHOOSE_POKEMON_TO_SWITCH",
        PromptKind::ChoosePokemon { player_type: pt, slots, min: 1, max: 1, allow_cancel: false, blocked: SVec::new() },
        Cont::Card { card: me, frame: f },
    );
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    if !has_bench(g, 1 - p) {
        // The generator returns undefined, so reduceEffect hands back an
        // undefined state and the play crashes in Twinleaf.
        bail!("TypeError: undefined state");
    }
    bench_prompt(g, me, p, PlayerType::TopPlayer, 1);
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    let sel = results.first().copied().unwrap_or(Res::Null);
    let t = match sel.slots().first() {
        Some(t) => *t,
        None => return Ok(()),
    };
    match f.stage {
        1 => {
            let o = 1 - p;
            let a = g.st.players[o].active;
            clear_effects(&mut g.st.players[o].slots[a as usize]);
            if t.p as usize == o {
                switch_pokemon_silent(g, o, t.s)?;
            }
            if !has_bench(g, p) {
                return Ok(());
            }
            bench_prompt(g, me, p, PlayerType::BottomPlayer, 2);
            Ok(())
        }
        2 => {
            let a = g.st.players[p].active;
            clear_effects(&mut g.st.players[p].slots[a as usize]);
            if t.p as usize == p {
                switch_pokemon_silent(g, p, t.s)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
