//! Deoxys (M4 32): Psyspear - 120; if this Pokémon has at least 2 extra
//! Energy attached, also 120 damage to 1 of your opponent's Benched Pokémon.
//!
//! Twinleaf: only with an occupied opponent Bench, a CheckAttackCostEffect is
//! reduced (result unused), then the provided Energy of the Active is summed
//! and compared with a hard-coded cost of 3; the bench target is a mandatory
//! ChoosePokemonPrompt and gets a PutDamageEffect (no Weakness/Resistance).
use crate::cards::prelude::*;
use crate::effects::Cost;

pub static IMPL: CardImpl = CardImpl { class: "Deoxys2", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !was_attack_used(g, e, 0, me) {
        return Ok(());
    }
    let (p, opp) = match *g.e(e) {
        Effect::Attack { p, opp, .. } => (p, opp as usize),
        _ => return Ok(()),
    };
    let has_benched = g.st.players[opp].bench.iter().any(|s| !g.st.players[opp].slots[*s as usize].cards.is_empty());
    if !has_benched {
        return Ok(());
    }
    let attack = my_attack(g, me, 0);
    let mut cost: Cost = SVec::new();
    for &c in crate::engine::attack::attack_def(g, attack).cost {
        cost.push(c);
    }
    g.run_fx(Effect::CheckAttackCost { p, attack, cost, set_cost: None, ignore_colorless: false, reduction: 0, any_reduction: false })?;
    let pu = p as usize;
    let src = SlotRef::new(pu, g.st.players[pu].active);
    let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p, source: src, energy_map: SVec::new() })?;
    let total: i32 = match pe {
        Effect::CheckProvidedEnergy { energy_map, .. } => energy_map.iter().map(|m| m.provides.len() as i32).sum(),
        _ => 0,
    };
    if total - 3 >= 2 {
        let mut slots = SVec::new();
        slots.push(SlotType::Bench as u8);
        g.retain_fx(e);
        let mut f = CardFrame::at(1);
        f.e[0] = e;
        let id = g.player_id(pu);
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
    let atk = f.e[0];
    let targets: Vec<SlotRef> = results.first().map(|r| r.slots().to_vec()).unwrap_or_default();
    let mut r = Ok(());
    for t in targets {
        r = put_damage(g, atk, 120, t);
        if r.is_err() {
            break;
        }
    }
    g.release_fx(atk);
    r
}
