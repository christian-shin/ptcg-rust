//! Pecharunt (SVP): Toxic Subjugation - while this Pokémon is Active, your
//! opponent's Poisoned Pokémon takes 5 more damage counters during Checkup.
//! Poison Chain - 10; the opponent's Active is now Poisoned and can't
//! retreat during their next turn.
//!
//! Twinleaf only looks at `active.cards[0]` (the bottom card of the stack).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Pecharunt", mask: mask(&[k::BETWEEN_TURNS, k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::BetweenTurns { p: current, .. } = *g.e(e) {
        let current = current as usize;
        let mut owner = None;
        for q in [current, 1 - current] {
            let a = g.st.players[q].active;
            if g.st.slot(q, a).cards.get(0) == Some(me) {
                owner = Some(q);
            }
        }
        let owner = match owner {
            Some(o) => o,
            None => return Ok(()),
        };
        if is_ability_blocked(g, owner, me, None) {
            return Ok(());
        }
        let victim = 1 - owner;
        let va = g.st.players[victim].active;
        if current == victim && g.st.slot(victim, va).special_conditions.contains(&(SpecialCondition::Poisoned as u8)) {
            if let Effect::BetweenTurns { poison_damage, .. } = g.e_mut(e) {
                *poison_damage += 50;
            }
        }
        return Ok(());
    }
    if was_attack_used(g, e, 0, me) {
        add_special_conditions_to_opponent_active(g, e, &[SpecialCondition::Poisoned])?;
        block_retreat(g, e)?;
    }
    Ok(())
}
