//! Okidogi (TWM): Adrena-Power — if this Pokémon has any [D] Energy
//! attached, it gets +100 HP, and its attacks do 100 more damage to the
//! opponent's Active Pokémon (before Weakness and Resistance). Good Punch — 70.
//!
//! Twinleaf: on a DealDamageEffect whose source slot holds this card
//! (non-zero damage, target = the attacker's opponent's Active) and on a
//! CheckHpEffect whose target slot holds it; the lock check uses the
//! effect's player; [D] counts when an energy-map entry provides DARK or ANY.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Okidogi", mask: mask(&[k::DEAL_DAMAGE, k::CHECK_HP]), reduce, resume: None, coin: None, can_play: None };

fn dark_provided(g: &mut Game, p: usize, slot: SlotRef) -> R<bool> {
    let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p: p as u8, source: slot, energy_map: SVec::new() })?;
    Ok(match pe {
        Effect::CheckProvidedEnergy { energy_map, .. } => energy_map.iter().any(|x| x.provides.contains(&ct::DARK) || x.provides.contains(&ct::ANY)),
        _ => false,
    })
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    match *g.e(e) {
        Effect::DealDamage { b, damage } => {
            if !g.st.slot(b.source.p as usize, b.source.s).cards.contains(me) {
                return Ok(());
            }
            let p = b.player as usize;
            let o = 1 - p;
            if damage == 0 || b.target != SlotRef::new(o, g.st.players[o].active) {
                return Ok(());
            }
            if is_ability_blocked(g, p, me, None) {
                return Ok(());
            }
            if dark_provided(g, p, b.source)? {
                if let Effect::DealDamage { damage, .. } = g.e_mut(e) {
                    *damage += 100;
                }
            }
        }
        Effect::CheckHp { p, target, card } => {
            if !g.st.slot(target.p as usize, target.s).cards.contains(me) {
                return Ok(());
            }
            let p = p as usize;
            if is_ability_blocked(g, p, me, None) {
                return Ok(());
            }
            if dark_provided(g, p, target)? && card.is_some() {
                // `effect.hp += 100`: the setter writes hpBonus only when a Pokémon was captured.
                g.st.players[target.p as usize].slots[target.s as usize].hp_bonus += 100;
            }
        }
        _ => {}
    }
    Ok(())
}
