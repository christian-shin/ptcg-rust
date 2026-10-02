//! Scramble Switch (PLS, ACE SPEC): switch your Active Pokémon with 1 of your
//! Benched Pokémon. Then, you may move as many Energy attached to the old
//! Active Pokémon to the new Active Pokémon as you like.
//!
//! Twinleaf: throws without a Benched Pokémon; the card moves to the
//! Supporter area (not the discard pile) and the play is prevented; a
//! ChoosePokemonPrompt over the Bench (cancellable: stops there). When the
//! Active has Energy, a ChooseCardsPrompt over the Active's cards (Energy,
//! 0..all, no cancel) moves the chosen Energy to the chosen Benched Pokémon;
//! then the silent `switchPokemon(target)` runs.
use crate::cards::prelude::*;
use crate::engine::turn::switch_pokemon_silent;

pub static IMPL: CardImpl = CardImpl { class: "ScrambleSwitch", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    let has_bench = g.st.players[p].bench.iter().any(|b| !g.st.players[p].slots[*b as usize].cards.is_empty());
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
        PromptKind::ChoosePokemon { player_type: PlayerType::BottomPlayer, slots, min: 1, max: 1, allow_cancel: true, blocked: SVec::new() },
        Cont::Card { card: me, frame: f },
    );
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    match f.stage {
        1 => {
            let target = match results.first().and_then(|r| r.slots().first().copied()) {
                Some(t) => t,
                None => return Ok(()),
            };
            let a = g.st.players[p].active;
            let has_energy = g.st.slot(p, a).cards.iter().any(|c| g.st.cdef(c).is_energy());
            if has_energy {
                let n = g.st.slot(p, a).cards.len().min(255) as u8;
                let mut nf = CardFrame::at(2);
                nf.a[0] = p as i32;
                nf.a[1] = target.s as i32;
                choose_cards(
                    g,
                    p,
                    "ATTACH_ENERGY_TO_BENCH",
                    ListRef::Slot(p as u8, a),
                    Filter::super_type(SuperType::Energy),
                    ChooseCardsOpts::new(0, n, false),
                    Cont::Card { card: me, frame: nf },
                );
                return Ok(());
            }
            switch_pokemon_silent(g, p, target.s)
        }
        2 => {
            let target = f.a[1] as SlotId;
            let selected: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
            let a = g.st.players[p].active;
            move_cards(g, ListRef::Slot(p as u8, a), ListRef::Slot(p as u8, target), &selected, me)?;
            switch_pokemon_silent(g, p, target)
        }
        _ => Ok(()),
    }
}
