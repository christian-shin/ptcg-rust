//! Milotic (TWM 50): Mentally Calm — your opponent's Pokémon in play and all
//! attached cards can't be put into your opponent's hand. Hydro Splash — 100.
//!
//! Twinleaf: reacts to every MoveCardsEffect whose source is a Pokémon slot
//! (any card of any copy): this card must be the top Pokémon of a slot (found
//! with findCardList; any failure returns), the destination must be the
//! hand of its owner's opponent, the source slot must be one of that
//! opponent's occupied Pokémon slots, and the owner's generic Ability probe
//! must pass; then preventDefault.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "MiloticTWMPool", mask: mask(&[k::MOVE_CARDS]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let (source, destination) = match *g.e(e) {
        Effect::MoveCards { source, destination, .. } => (source, destination),
        _ => return Ok(()),
    };
    let (sq, ss) = match source {
        ListRef::Slot(q, s) => (q as usize, s),
        _ => return Ok(()),
    };
    let (mp, ms) = match g.st.locate(me) {
        Some(ListRef::Slot(q, s)) => (q as usize, s),
        _ => return Ok(()),
    };
    if g.st.slot_pokemon(mp, ms) != Some(me) {
        return Ok(());
    }
    let owner = mp;
    let opp = 1 - owner;
    if destination != ListRef::Hand(opp as u8) {
        return Ok(());
    }
    let from_opponents_play = sq == opp && g.st.slot_pokemon(opp, ss).is_some();
    if !from_opponents_play {
        return Ok(());
    }
    if is_ability_blocked(g, owner, me, None) {
        return Ok(());
    }
    g.set_prevent(e, true);
    Ok(())
}
