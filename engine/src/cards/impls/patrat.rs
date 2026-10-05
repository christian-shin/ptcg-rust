//! Patrat (M4): Watchful Eye — damage counters on each Pokémon can't be
//! moved to other Pokémon. Bite — 10.
//!
//! Twinleaf: any MoveDamageCountersEffect or MoveCountersAttackEffect (Cofagrigus
//! WHT's Extended Damagriiigus) is prevented while any Patrat is in play on
//! either side (checked from any zone).
//!
//! Fixed (phase 4b, R2): a Patrat whose Ability is blocked (Team Rocket's
//! Watchtower hits [C] Pokémon) still prevented the move; each Patrat now
//! counts only when `IS_ABILITY_BLOCKED` is false for it.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Patrat", mask: mask(&[k::MOVE_DAMAGE_COUNTERS, k::MOVE_COUNTERS]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, _me: CardId, e: EffId) -> R {
    if let Effect::MoveDamageCounters { .. } | Effect::MoveCounters { .. } = *g.e(e) {
        let mut has = false;
        for p in 0..2 {
            for (_, c, _) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
                if g.st.cdef(c).name == "Patrat" && !is_ability_blocked(g, p, c, None) {
                    has = true;
                }
            }
        }
        if has {
            g.set_prevent(e, true);
        }
    }
    Ok(())
}
