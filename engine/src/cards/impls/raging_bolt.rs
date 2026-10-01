//! Raging Bolt (SCR): Thunderburst Storm — 30 damage to 1 of the opponent's
//! Pokémon for each Energy attached to this Pokémon. Dragon Headbutt — 130.
//!
//! Twinleaf: ChoosePokemonPrompt (Active + Bench, no cancel); in the callback
//! CheckProvidedEnergyEffect(player, player.active) is summed over every
//! provided type (Special Energy providing 2 count twice), then
//! DAMAGE_OPPONENT_POKEMON (DealDamage on the Active, PutDamage on the Bench).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "RagingBolt", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let mut slots = SVec::new();
        slots.push(SlotType::Active as u8);
        slots.push(SlotType::Bench as u8);
        g.retain_fx(e);
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        f.e[0] = e;
        let id = g.player_id(p);
        g.prompt(
            id,
            "CHOOSE_POKEMON_TO_DAMAGE",
            PromptKind::ChoosePokemon { player_type: PlayerType::TopPlayer, slots, min: 1, max: 1, allow_cancel: false, blocked: SVec::new() },
            Cont::Card { card: me, frame: f },
        );
    }
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let atk = f.e[0];
    let targets: Vec<SlotRef> = results.first().map(|r| r.slots().to_vec()).unwrap_or_default();
    let r = (|| -> R {
        let a = g.st.players[p].active;
        let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p: p as u8, source: SlotRef::new(p, a), energy_map: SVec::new() })?;
        let provided: usize = match pe {
            Effect::CheckProvidedEnergy { energy_map, .. } => energy_map.iter().map(|m| m.provides.len()).sum(),
            _ => 0,
        };
        damage_opponent_pokemon(g, atk, provided as i32 * 30, &targets)
    })();
    g.release_fx(atk);
    r
}
