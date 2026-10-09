//! Game-state invariants, checked by the replay and self-play
//! tools at every turn decision.
//!
//! Every check here must hold in any reachable state of a correct game. A
//! violation that the oracle shares (the state hash still matches) is a card
//! bug in both engines, e.g. a card duplicated into two zones.

use crate::game::{Game, Pending};
use crate::list::CardList;
use crate::state::MAX_CARDS;
use crate::types::GamePhase;

/// All invariants; `Err` names the first one violated.
pub fn check(g: &Game) -> Result<(), String> {
    let st = &g.st;
    let n = st.n_cards as usize;
    let mut seen = [0u8; MAX_CARDS];
    let mut count = |cards: &[u8], zone: &str| -> Result<(), String> {
        for &c in cards {
            if c as usize >= n {
                return Err(format!("card id {} out of range in {}", c, zone));
            }
            seen[c as usize] += 1;
        }
        Ok(())
    };
    for (p, pl) in st.players.iter().enumerate() {
        count(pl.deck.as_slice(), "deck")?;
        count(pl.hand.as_slice(), "hand")?;
        count(pl.discard.as_slice(), "discard")?;
        count(pl.lostzone.as_slice(), "lostzone")?;
        count(pl.stadium.as_slice(), "stadium")?;
        count(pl.supporter.as_slice(), "supporter")?;
        let mut prizes = 0;
        for pr in pl.prizes.iter() {
            count(pr.as_slice(), "prizes")?;
            prizes += pr.len();
        }
        if prizes > 6 {
            return Err(format!("player {} has {} Prize cards", p, prizes));
        }
        for s in pl.all_slots().iter() {
            let slot = &pl.slots[*s as usize];
            count(slot.cards.as_slice(), "slot")?;
            // `energies` is a view of the Energy cards in `cards`.
            if let Some(e) = slot.energies.as_slice().iter().find(|e| !slot.cards.contains(**e)) {
                return Err(format!("player {} slot {}: attached Energy {} is not in the slot's cards", p, s, e));
            }
            count(slot.tools.as_slice(), "slot tools")?;
            if slot.damage < 0 {
                return Err(format!("player {} slot {} has damage {}", p, s, slot.damage));
            }
        }
        let occupied = pl.bench.iter().filter(|b| !pl.slots[**b as usize].cards.is_empty()).count();
        if occupied > pl.bench.len() {
            return Err(format!("player {} has {} Benched Pokémon on {} spots", p, occupied, pl.bench.len()));
        }
        if matches!(st.phase, GamePhase::PlayerTurn) && pl.slots[pl.active as usize].cards.is_empty() {
            return Err(format!("player {} has no Active Pokémon during a turn", p));
        }
    }
    for c in 0..n {
        match seen[c] {
            1 => {}
            0 => return Err(format!("card {} ({}) is in no zone", c, st.cdef(c as u8).full_name)),
            k => return Err(format!("card {} ({}) is in {} zones", c, st.cdef(c as u8).full_name, k)),
        }
    }
    if matches!(g.pending(), Pending::Turn(_)) && !g.fx.is_empty() {
        return Err(format!("{} effects on the stack at a turn decision", g.fx.len()));
    }
    Ok(())
}
