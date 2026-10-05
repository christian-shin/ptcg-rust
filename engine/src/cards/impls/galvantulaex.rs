//! Galvantula ex (SCR, Tera): Charged Web — 110+, 110 more if your
//! opponent's Active Pokémon is a Pokémon ex or Pokémon V. Fulgurite — 180;
//! discard all Energy from this Pokémon; during your opponent's next turn
//! they can't play Item cards.
//!
//! Twinleaf: the V check covers V / VSTAR / VMAX tags (not V-UNION).
//! Fulgurite: CheckProvidedEnergyEffect(player) on the Active, a
//! DiscardCardsEffect of the map's cards on the Active, then
//! OPPONENT_CANNOT_PLAY_ITEM_CARDS (PlayLockEffect). Tera bench protection.
//!
//! Fixed (phase 4b, R2): the Energy was discarded in the attack handler,
//! before the damage (Voltaic Lightning Energy's +20 was lost); it is now
//! discarded in AfterAttackEffect. The Item lock stays in the attack handler.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Galvantulaex", mask: mask(&[k::ATTACK, k::AFTER_ATTACK, k::PUT_DAMAGE]), reduce, resume: None, coin: None, can_play: None };

/// CheckProvidedEnergyEffect(player) on the Active, then a DiscardCardsEffect
/// of every mapped card with `target = player.active`.
pub fn discard_all_active_energy(g: &mut Game, e: EffId) -> R {
    let (p, opp, attack, source) = match *g.e(e) {
        Effect::Attack { p, opp, attack, source, .. } => (p, opp, attack, source),
        // `new AttackEffect(player, opponent, attack)`: its source is the Active.
        Effect::AfterAttack { p, opp, attack } => (p, opp, attack, SlotRef::new(p as usize, g.st.players[p as usize].active)),
        _ => return Ok(()),
    };
    let active = SlotRef::new(p as usize, g.st.players[p as usize].active);
    let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p, source: active, energy_map: SVec::new() })?;
    let mut cards: SVec<CardId, 16> = SVec::new();
    if let Effect::CheckProvidedEnergy { energy_map, .. } = pe {
        for em in energy_map.iter() {
            cards.push(em.card);
        }
    }
    g.run_fx(Effect::DiscardCards { b: AtkBase { attack_effect: e, player: p, opponent: opp, attack, source, target: active }, cards })?;
    Ok(())
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let o = match *g.e(e) {
            Effect::Attack { opp, .. } => opp as usize,
            _ => return Ok(()),
        };
        if let Some(c) = g.st.active_pokemon(o) {
            let d = g.st.cdef(c);
            if d.has_tag(tag::POKEMON_V) || d.has_tag(tag::POKEMON_VSTAR) || d.has_tag(tag::POKEMON_VMAX) || d.has_tag(tag::POKEMON_EX_LOWER) {
                if let Effect::Attack { damage, .. } = g.e_mut(e) {
                    *damage += 110;
                }
            }
        }
    }
    if after_attack_used(g, e, 1, me) {
        discard_all_active_energy(g, e)?;
    }
    if was_attack_used(g, e, 1, me) {
        return opponent_cannot_play_cards(g, e, crate::effects::play_lock::ITEM);
    }
    tera_rule(g, e, me);
    Ok(())
}
