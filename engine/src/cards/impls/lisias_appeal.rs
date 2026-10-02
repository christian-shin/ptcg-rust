//! Lisia's Appeal (SSP, supporter): switch in 1 of your opponent's Benched
//! Basic Pokémon to the Active Spot; the new Active Pokémon is now Confused.
//!
//! Twinleaf: no supporter-turn check in `reduceEffect`; fails when the
//! opponent has no Bench. SWITCH_IN_OPPONENT_BENCHED_POKEMON with the
//! non-Basic Bench spots blocked (no cancel); `opponent.switchPokemon` with
//! `store, state`; then, unless a TrainerTargetEffect on the new Active is
//! blocked, Confused is added directly (`addSpecialCondition`).
use crate::cards::prelude::*;
use crate::engine::phase::add_condition;

pub static IMPL: CardImpl = CardImpl { class: "LisiasAppeal", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    let o = 1 - p;
    let has_bench = {
        let pl = &g.st.players[o];
        pl.bench.iter().any(|b| !pl.slots[*b as usize].cards.is_empty())
    };
    if !has_bench {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    let mut blocked = TargetList::new();
    for (s, c, target) in for_each_pokemon(g, o, PlayerType::TopPlayer).iter() {
        let _ = s;
        if target.slot != SlotType::Bench {
            continue;
        }
        if g.st.cdef(*c).stage != Stage::Basic as u8 {
            blocked.push(*target);
        }
    }
    let mut slots = SVec::new();
    slots.push(SlotType::Bench as u8);
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    let id = g.player_id(p);
    g.prompt(
        id,
        "CHOOSE_POKEMON_TO_SWITCH",
        PromptKind::ChoosePokemon { player_type: PlayerType::TopPlayer, slots, min: 1, max: 1, allow_cancel: false, blocked },
        Cont::Card { card: me, frame: f },
    );
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let o = 1 - p;
    let sel = match results.first() {
        Some(r) => r.slots(),
        None => return Ok(()),
    };
    if sel.is_empty() {
        return Ok(());
    }
    if sel[0].p as usize == o {
        crate::engine::turn::switch_pokemon(g, o, sel[0].s)?;
    }
    let target = SlotRef::new(o, g.st.players[o].active);
    let (t, prevented) = g.run_fx(Effect::TrainerTarget { p: p as u8, card: me, target: Some(target) })?;
    let blocked = prevented || matches!(t, Effect::TrainerTarget { target: None, .. });
    if !blocked {
        let a = g.st.players[o].active;
        add_condition(&mut g.st.players[o].slots[a as usize], SpecialCondition::Confused);
    }
    Ok(())
}
