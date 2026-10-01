//! Battle Cage ("Battle Colosseum M2", PFL, stadium): prevent all damage
//! counters from being placed on Benched Pokémon (both yours and your
//! opponent's) by effects of attacks and Abilities from the opponent's
//! Pokémon.
//!
//! Twinleaf: while this is the stadium in play,
//! * every MoveDamageCountersEffect whose player is the non-active player is
//!   prevented (no bench / source check, no stadium-block probe);
//! * a PutCountersEffect (attack) or PlaceDamageCountersEffect (whose source
//!   card is still in play) on a Benched Pokémon from its owner's opponent
//!   is prevented unless the stadium effect is blocked for that target.
//!   The stadium can be "used" (nothing happens).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "BattleColosseum",
    mask: mask(&[k::MOVE_DAMAGE_COUNTERS, k::PUT_COUNTERS, k::PLACE_DAMAGE_COUNTERS]),
    reduce,
    resume: None,
    coin: None,
    can_play: None,
};

/// `StateUtils.findPokemonSlot(state, card)`: owner of the in-play slot
/// holding `card` (cards, energies or tools).
fn find_slot_owner(g: &Game, card: CardId) -> Option<usize> {
    for p in 0..2usize {
        let pl = &g.st.players[p];
        for s in std::iter::once(pl.active).chain(pl.bench.iter().copied()) {
            let slot = g.st.slot(p, s);
            if slot.cards.contains(card) || slot.energies.contains(card) || slot.tools.contains(card) {
                return Some(p);
            }
        }
    }
    None
}

fn bench_target_prevented(g: &mut Game, me: CardId, source_owner: usize, t: SlotRef) -> bool {
    let owner = t.p as usize;
    if source_owner != 1 - owner {
        return false;
    }
    // `target !== targetOpponent.active` always holds (slot ids are per player).
    if t.s == g.st.players[owner].active {
        return false;
    }
    !is_stadium_effect_blocked(g, owner, t, me)
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if g.st.stadium_card() != Some(me) {
        return Ok(());
    }
    match *g.e(e) {
        Effect::MoveDamageCounters { p } => {
            let active = g.st.active_player as usize;
            if p as usize == 1 - active {
                g.set_prevent(e, true);
            }
        }
        Effect::PutCounters { b, .. } => {
            if bench_target_prevented(g, me, b.source.p as usize, b.target) {
                g.set_prevent(e, true);
            }
        }
        Effect::PlaceDamageCounters { target, source, .. } => {
            if source == NO_CARD {
                return Ok(());
            }
            if let Some(so) = find_slot_owner(g, source) {
                if bench_target_prevented(g, me, so, target) {
                    g.set_prevent(e, true);
                }
            }
        }
        _ => {}
    }
    Ok(())
}
