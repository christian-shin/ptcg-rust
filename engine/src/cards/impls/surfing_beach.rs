//! Surfing Beach (M1S / CRI, stadium): once during each player's turn, that
//! player may switch their Active [W] Pokémon with 1 of their Benched [W]
//! Pokémon.
//!
//! Twinleaf quirks kept: the `blocked` list gets the Active and every Bench
//! slot that is not [W] (the `else` binds to the second `if`); the card
//! throws CANNOT_USE_STADIUM without an Active and a Benched [W] Pokémon or
//! when stadium effects on the Active are blocked; the callback re-checks the
//! chosen slot, then switches silently (`player.switchPokemon(target)`).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "SurfingBeach", mask: mask(&[k::USE_STADIUM]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match *g.e(e) {
        Effect::UseStadium { p, .. } if g.st.stadium_card() == Some(me) => p as usize,
        _ => return Ok(()),
    };
    let mut has_bench_water = false;
    let mut has_active_water = false;
    let mut blocked: TargetList = SVec::new();
    let active = g.st.players[p].active;
    for (s, _, t) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
        let target = SlotRef::new(p, s);
        let types = crate::engine::game_effect::pokemon_types(g, target);
        let (te, _) = g.run_fx(Effect::CheckPokemonType { target, card_types: types })?;
        let water = match te {
            Effect::CheckPokemonType { card_types, .. } => card_types.contains(&ct::WATER),
            _ => false,
        };
        if s == active && water {
            has_active_water = true;
        }
        if s != active && water {
            has_bench_water = true;
        } else {
            blocked.push(t);
        }
    }
    if !(has_active_water && has_bench_water) {
        bail!("CANNOT_USE_STADIUM");
    }
    if is_stadium_effect_blocked(g, p, SlotRef::new(p, active), me) {
        bail!("CANNOT_USE_STADIUM");
    }
    let mut slots = SVec::new();
    slots.push(SlotType::Bench as u8);
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    let id = g.player_id(p);
    g.prompt(
        id,
        "CHOOSE_NEW_ACTIVE_POKEMON",
        PromptKind::ChoosePokemon { player_type: PlayerType::BottomPlayer, slots, min: 1, max: 1, allow_cancel: false, blocked },
        Cont::Card { card: me, frame: f },
    );
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let t = match results.first().and_then(|r| r.slots().first().copied()) {
        Some(t) => t,
        None => return Ok(()),
    };
    if is_stadium_effect_blocked(g, p, t, me) {
        return Ok(());
    }
    if t.p as usize == p {
        crate::engine::turn::switch_pokemon_silent(g, p, t.s)?;
    }
    Ok(())
}
