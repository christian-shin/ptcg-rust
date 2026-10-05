//! Team Rocket's Giovanni (DRI): switch your Active Team Rocket's Pokémon
//! with 1 of your Benched Team Rocket's Pokémon. If you do, switch in 1 of
//! your opponent's Benched Pokémon to the Active Spot.
//!
//! Twinleaf: the card goes to the supporter pile (and `rocketSupporter` is
//! set) before the Active / Bench checks throw. Each switch clears the old
//! Active's effects first.
//!
//! Fixed in phase 4b (R4): playable when the opponent has no Benched Pokémon
//! (only your own switch happens; the opponent's is the "if you do" part and is
//! skipped), and both switches dispatch MovedToActive / MovedFromActiveToBench
//! (they were silent: Yanmega ex Buzz Boost, Palafin Zero to Hero and the
//! ability-lock activation order never saw them).
//!
//! R7C: `rocket_supporter` is not set when used as the effect of an attack (ruling 1727).
use crate::cards::prelude::*;
use crate::engine::game_effect::clear_effects;
use crate::engine::turn::switch_pokemon;

pub static IMPL: CardImpl = CardImpl { class: "TeamRocketsGiovanni", mask: mask(&[k::TRAINER, k::END_TURN]), reduce, resume: Some(resume), coin: None, can_play: None };

fn bench_prompt(g: &mut Game, me: CardId, p: usize, pt: PlayerType, blocked: TargetList, stage: u8) {
    let mut slots = SVec::new();
    slots.push(SlotType::Bench as u8);
    let id = g.player_id(p);
    let mut f = CardFrame::at(stage);
    f.a[0] = p as i32;
    g.prompt(
        id,
        "CHOOSE_POKEMON_TO_SWITCH",
        PromptKind::ChoosePokemon { player_type: pt, slots, min: 1, max: 1, allow_cancel: false, blocked },
        Cont::Card { card: me, frame: f },
    );
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Some(p) = trainer_played(g, e, me) {
        if g.st.players[p].supporter_turn > 0 {
            bail!("SUPPORTER_ALREADY_PLAYED");
        }
        // Using the effect of a Supporter as the effect of an attack is not playing it from the hand.
        if !trainer_via_attack(g, e) {
            g.st.players[p].rocket_supporter = true;
        }
        move_cards(g, ListRef::Hand(p as u8), ListRef::Supporter(p as u8), &[me], me)?;
        g.set_prevent(e, true);
        match g.st.active_pokemon(p) {
            Some(c) if g.st.cdef(c).has_tag(tag::TEAM_ROCKET) => {}
            _ => bail!("CANNOT_PLAY_THIS_CARD"),
        }
        let mut blocked: TargetList = SVec::new();
        let mut rocket_bench = 0;
        for (_, c, t) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
            if t.slot == SlotType::Bench {
                if !g.st.cdef(c).has_tag(tag::TEAM_ROCKET) {
                    blocked.push(t);
                } else {
                    rocket_bench += 1;
                }
            }
        }
        if rocket_bench == 0 {
            bail!("CANNOT_PLAY_THIS_CARD");
        }
        bench_prompt(g, me, p, PlayerType::BottomPlayer, blocked, 1);
        return Ok(());
    }
    if let Effect::EndTurn { p } = *g.e(e) {
        if g.st.players[p as usize].rocket_supporter {
            g.st.players[p as usize].rocket_supporter = false;
        }
    }
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
            let a = g.st.players[p].active;
            clear_effects(&mut g.st.players[p].slots[a as usize]);
            if t.p as usize == p {
                switch_pokemon(g, p, t.s)?;
            }
            // If you do, switch in 1 of the opponent's Benched Pokémon (nothing more to do without one)
            let opl = &g.st.players[1 - p];
            if !opl.bench.iter().any(|s| !opl.slots[*s as usize].cards.is_empty()) {
                return Ok(());
            }
            bench_prompt(g, me, p, PlayerType::TopPlayer, SVec::new(), 2);
            Ok(())
        }
        2 => {
            let o = 1 - p;
            let a = g.st.players[o].active;
            clear_effects(&mut g.st.players[o].slots[a as usize]);
            if t.p as usize == o {
                switch_pokemon(g, o, t.s)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
