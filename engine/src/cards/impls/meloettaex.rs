//! Meloetta ex (SV11B): Live Debut — if you go first, this Pokémon can attack
//! on your first turn. Echoed Voice — 30; during your next turn, this
//! Pokémon's Echoed Voice attack does 80 more damage.
//!
//! Twinleaf: on every UseAttackEffect whose player has this card in the
//! Active slot on turn 1 (and the Ability is not blocked) it sets
//! `effect.attack.canUseOnFirstTurn`; NEXT_TURN_ATTACK_BONUS runs on every
//! AttackEffect whose attacker is this card (see Metagross TEF).
use crate::cards::prelude::*;
use crate::state::NextTurnAttackDamageBonus;

pub static IMPL: CardImpl = CardImpl { class: "Meloettaex", mask: mask(&[k::USE_ATTACK, k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

const ECHOED_VOICE: &str = "Echoed Voice";

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::UseAttack { p, .. } = *g.e(e) {
        let a = g.st.players[p as usize].active;
        if g.st.slot(p as usize, a).cards.contains(me) && g.st.turn == 1 {
            if is_ability_blocked(g, p as usize, me, None) {
                return Ok(());
            }
            if let Effect::UseAttack { attack, .. } = *g.e(e) {
                g.st.cards[attack.card as usize].attack_first_turn |= 1u8 << attack.idx();
            }
        }
        return Ok(());
    }

    // NEXT_TURN_ATTACK_BONUS
    let (attack, source) = match *g.e(e) {
        Effect::Attack { attack, source, .. } => (attack, source),
        _ => return Ok(()),
    };
    if g.st.slot_pokemon(source.p as usize, source.s) != Some(me) {
        return Ok(());
    }
    let full_name = g.st.cdef(me).full_name;
    let attack_name = g.st.cdef(attack.card).attacks[attack.idx()].name;
    let slot = &g.st.players[source.p as usize].slots[source.s as usize];
    let bonus = match slot.next_turn_attack_damage_bonus {
        Some(b) if b.source_card_name == full_name && (b.attack_name == "*" || b.attack_name == attack_name) => b.bonus_damage,
        _ => 0,
    };
    if bonus != 0 {
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage += bonus;
        }
    }
    if attack_name != ECHOED_VOICE {
        return Ok(());
    }
    g.st.players[source.p as usize].slots[source.s as usize].next_turn_attack_damage_bonus_pending =
        Some(NextTurnAttackDamageBonus { attack_name: ECHOED_VOICE, bonus_damage: 80, source_card_name: full_name });
    Ok(())
}
