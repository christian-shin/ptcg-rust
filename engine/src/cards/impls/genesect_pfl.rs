//! Genesect (M2 / PFL 8): Bug's Cannon — choose 1 of your opponent's
//! Pokémon; 20 damage to it for each [G] Energy attached to this Pokémon.
//! Speed Attack — 110.
//!
//! Twinleaf: counts GRASS / ANY `provides` on `player.active`, then a
//! non-cancellable ChoosePokemonPrompt (min 1, max 1) and
//! DAMAGE_OPPONENT_POKEMON (DealDamage on the Active, PutDamage on the
//! Bench), even for 0 damage. Two `Genesect` classes exist; this port is
//! bound to PFL.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Genesect@PFL", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !was_attack_used(g, e, 0, me) {
        return Ok(());
    }
    let p = match *g.e(e) {
        Effect::Attack { p, .. } => p as usize,
        _ => return Ok(()),
    };
    let a = g.st.players[p].active;
    let n = super::mega_meganiumex::grass_energy_count(g, p, a)?;
    let mut slots = SVec::new();
    slots.push(SlotType::Active as u8);
    slots.push(SlotType::Bench as u8);
    g.retain_fx(e);
    let mut f = CardFrame::at(1);
    f.e[0] = e;
    f.a[0] = n * 20;
    let id = g.player_id(p);
    g.prompt(
        id,
        "CHOOSE_POKEMON_TO_DAMAGE",
        PromptKind::ChoosePokemon { player_type: PlayerType::TopPlayer, slots, min: 1, max: 1, allow_cancel: false, blocked: SVec::new() },
        Cont::Card { card: me, frame: f },
    );
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let atk = f.e[0];
    let targets: Vec<SlotRef> = results.first().map(|r| r.slots().to_vec()).unwrap_or_default();
    let r = damage_opponent_pokemon(g, atk, f.a[0], &targets);
    g.release_fx(atk);
    r
}
