//! Patrat (M4): Watchful Eye — damage counters on each Pokémon can't be
//! moved to other Pokémon. Bite — 10.
//!
//! Twinleaf: any MoveDamageCountersEffect is prevented while any Patrat is
//! in play on either side (checked from any zone, no ability-lock check).
//! MoveCountersAttackEffect is never created by a pool card.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Patrat", mask: mask(&[k::MOVE_DAMAGE_COUNTERS]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, _me: CardId, e: EffId) -> R {
    if let Effect::MoveDamageCounters { .. } = *g.e(e) {
        let has = (0..2).any(|p| for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().any(|(_, c, _)| g.st.cdef(*c).name == "Patrat"));
        if has {
            g.set_prevent(e, true);
        }
    }
    Ok(())
}
