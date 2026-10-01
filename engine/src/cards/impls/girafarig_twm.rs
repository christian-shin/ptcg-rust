//! Girafarig (TWM): Dual Headbutt — 30; this attack also does 10 damage to 1
//! of your Benched Pokémon.
//!
//! Twinleaf (twilight-masquerade file): no Benched Pokémon → nothing;
//! otherwise ChoosePokemonPrompt (your Bench, no cancel) and a
//! PutDamageEffect(10) on the chosen Pokémon.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Girafarig@TWM", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !was_attack_used(g, e, 0, me) {
        return Ok(());
    }
    let p = match *g.e(e) {
        Effect::Attack { p, .. } => p as usize,
        _ => return Ok(()),
    };
    let pl = &g.st.players[p];
    if !pl.bench.iter().any(|b| !pl.slots[*b as usize].cards.is_empty()) {
        return Ok(());
    }
    let mut slots = SVec::new();
    slots.push(SlotType::Bench as u8);
    g.retain_fx(e);
    let mut f = CardFrame::at(1);
    f.e[0] = e;
    let id = g.player_id(p);
    g.prompt(
        id,
        "CHOOSE_POKEMON_TO_DAMAGE",
        PromptKind::ChoosePokemon { player_type: PlayerType::BottomPlayer, slots, min: 1, max: 1, allow_cancel: false, blocked: SVec::new() },
        Cont::Card { card: me, frame: f },
    );
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let atk = f.e[0];
    let target = results.first().and_then(|r| r.slots().first().copied());
    let r = match target {
        Some(t) => put_damage(g, atk, 10, t),
        None => Ok(()),
    };
    g.release_fx(atk);
    r
}
