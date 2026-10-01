//! Mega Slowbro ex (MEP): Shellnado Spin — 180; during your opponent's next
//! turn, if this Pokémon is damaged by an attack (even if Knocked Out), place
//! 12 damage counters on the Attacking Pokémon.
//!
//! Twinleaf: THIS_POKEMON_RETALIATES_ON_DAMAGE_DURING_OPPONENTS_NEXT_TURN
//! arms `retaliateOnDamageNextTurnPending = { damage: 120, attack, sourceCard,
//! attackerPlayerId }` on the attacker's Active (see `attack.rs`, AfterDamage).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "MegaSlowbroex", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        if let Effect::Attack { p, opp, attack, source, .. } = *g.e(e) {
            let b = AtkBase { attack_effect: e, player: p, opponent: opp, attack, source, target: source };
            g.run_fx(Effect::RetaliateOnDamage { b, damage: 120, source_card: me })?;
        }
    }
    Ok(())
}
