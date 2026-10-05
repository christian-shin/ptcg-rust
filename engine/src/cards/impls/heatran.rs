//! Heatran (TWM 123): Incandescent Body — if this Pokémon is in the Active
//! Spot and is damaged by an attack from your opponent's Pokémon (even if it
//! is Knocked Out), the Attacking Pokémon is now Burned. Steel Burst — 50x;
//! discard all [M] Energy from this Pokémon, 50 damage for each card
//! discarded.
//!
//! Twinleaf: Steel Burst discards the cards of the Active's
//! CheckProvidedEnergy map whose entry provides [M] in one DiscardCardsEffect
//! and adds `(listed - 1) * 50` (counted before the discard resolves).
//! Fixed (phase 4b, R3): it used to discard and count every attached Energy
//! card, whatever its type. Incandescent Body reacts to AfterDamageEffect on
//! any list holding this card: the lock probe runs for the *attacking*
//! player, and the burn is a direct `source.addSpecialCondition` during the
//! attack phase.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Heatran", mask: mask(&[k::ATTACK, k::AFTER_DAMAGE]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let n = discard_all_metal_energy_from_active(g, e)? as i32;
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage += (n - 1) * 50;
        }
    }
    if let Effect::AfterDamage { b, damage } = *g.e(e) {
        let t = b.target;
        if !g.st.slot(t.p as usize, t.s).cards.contains(me) {
            return Ok(());
        }
        let player = b.player as usize;
        let owner = t.p as usize;
        if damage <= 0 || player == owner || g.st.players[owner].active != t.s {
            return Ok(());
        }
        if is_ability_blocked(g, player, me, None) {
            return Ok(());
        }
        if g.st.phase == GamePhase::Attack {
            let src = b.source;
            crate::engine::phase::add_condition(&mut g.st.players[src.p as usize].slots[src.s as usize], SpecialCondition::Burned);
        }
    }
    Ok(())
}

/// `CheckProvidedEnergyEffect(player)` on the Active, then one
/// DiscardCardsEffect of the mapped cards whose entry provides [M] aimed at
/// `player.active`. Returns how many cards the effect listed.
fn discard_all_metal_energy_from_active(g: &mut Game, e: EffId) -> R<usize> {
    let (p, opp, attack, source) = match *g.e(e) {
        Effect::Attack { p, opp, attack, source, .. } => (p, opp, attack, source),
        _ => return Ok(0),
    };
    let pu = p as usize;
    let active = SlotRef::new(pu, g.st.players[pu].active);
    let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p, source: active, energy_map: SVec::new() })?;
    let mut cards: SVec<CardId, 16> = SVec::new();
    if let Effect::CheckProvidedEnergy { energy_map, .. } = pe {
        for m in energy_map.iter() {
            if m.provides.contains(&ct::METAL) {
                cards.push(m.card);
            }
        }
    }
    let n = cards.len();
    let target = SlotRef::new(pu, g.st.players[pu].active);
    let b = AtkBase { attack_effect: e, player: p, opponent: opp, attack, source, target };
    g.run_fx(Effect::DiscardCards { b, cards })?;
    Ok(n)
}
