//! Jellicent ex (WHT / SV11W): Oceanic Curse - while this Pokémon is your
//! Active, your opponent can't play Item cards or attach Pokémon Tools.
//! Power Press - 80+; 80 more with at least 2 extra Energy.
//!
//! Twinleaf: the extra Energy sums the provided Energy of the Active minus the
//! checked attack cost. The lock throws BLOCKED_BY_ABILITY when the
//! ability probe for the *opponent of the player* passes (i.e. the Ability is not blocked).
use crate::cards::prelude::*;
use crate::effects::Cost;

pub static IMPL: CardImpl = CardImpl {
    class: "Jellicentex",
    mask: mask(&[k::ATTACK, k::PLAY_ITEM, k::ATTACH_POKEMON_TOOL]),
    reduce,
    resume: None,
    coin: None,
    can_play: None,
};

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p,
            _ => return Ok(()),
        };
        let attack = my_attack(g, me, 0);
        let mut cost: Cost = SVec::new();
        for &c in crate::engine::attack::attack_def(g, attack).cost {
            cost.push(c);
        }
        let (ce, _) = g.run_fx(Effect::CheckAttackCost { p, attack, cost })?;
        let cost_len = match ce {
            Effect::CheckAttackCost { cost, .. } => cost.len() as i32,
            _ => 0,
        };
        let pu = p as usize;
        let src = SlotRef::new(pu, g.st.players[pu].active);
        let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p, source: src, energy_map: SVec::new() })?;
        let total: i32 = match pe {
            Effect::CheckProvidedEnergy { energy_map, .. } => energy_map.iter().map(|m| m.provides.len() as i32).sum(),
            _ => 0,
        };
        if total - cost_len >= 2 {
            if let Effect::Attack { damage, .. } = g.e_mut(e) {
                *damage += 80;
            }
        }
    }

    let p = match *g.e(e) {
        Effect::PlayItem { p, .. } | Effect::AttachPokemonTool { p, .. } => p as usize,
        _ => return Ok(()),
    };
    let opp = 1 - p;
    if g.st.active_pokemon(opp) == Some(me) && !is_ability_blocked(g, opp, me, None) {
        bail!("BLOCKED_BY_ABILITY");
    }
    Ok(())
}
