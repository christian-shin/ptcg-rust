//! Slakoth (SSP): Take It Easy — heal 60 damage from this Pokémon. During
//! your next turn, this Pokémon can't retreat.
//!
//! Twinleaf: HEAL_X_DAMAGE_FROM_THIS_POKEMON (a HealTargetEffect on
//! `player.active`) then BLOCK_SELF_RETREAT (a SelfPreventRetreatEffect whose
//! target is the attacker since phase 4b, so Mist Energy on the Defending
//! Pokémon no longer blocks it).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Slakoth", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !was_attack_used(g, e, 0, me) {
        return Ok(());
    }
    let (p, opp, attack, source) = match *g.e(e) {
        Effect::Attack { p, opp, attack, source, .. } => (p, opp, attack, source),
        _ => return Ok(()),
    };
    let a = g.st.players[p as usize].active;
    let b = AtkBase { attack_effect: e, player: p, opponent: opp, attack, source, target: SlotRef::new(p as usize, a) };
    g.run_fx(Effect::HealTarget { b, damage: 60 })?;
    block_self_retreat(g, e)
}
