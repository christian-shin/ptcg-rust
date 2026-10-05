//! Hisuian Growlithe (TWM): Blazing Destruction — discard a Stadium in play.
//! Take Down — 40, this Pokémon also does 10 damage to itself.
//!
//! Twinleaf: with no Stadium in play the attack does nothing (phase 4b: it used
//! to throw CANNOT_USE_ATTACK, but an attack can be used with no effect);
//! the Stadium goes to its owner's discard (MOVE_CARDS of the whole list).
//! Take Down's recoil is a DealDamageEffect aimed at the attacker's Active.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "HisuianGrowlithe", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

/// `MOVE_CARDS(findCardList(stadium), findOwner(list).discard, { sourceCard })`.
pub fn discard_stadium(g: &mut Game, stadium: CardId, me: CardId) -> R {
    let src = match g.st.locate(stadium) {
        Some(l) => l,
        None => bail!("INVALID_GAME_STATE"),
    };
    let owner = match src.owner() {
        Some(o) => o,
        None => bail!("INVALID_GAME_STATE"),
    };
    g.run_fx(Effect::MoveCards {
        source: src,
        destination: ListRef::Discard(owner as u8),
        cards: None,
        count: None,
        to_top: false,
        to_bottom: false,
        skip_cleanup: false,
        source_card: me,
    })?;
    Ok(())
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let stadium = match g.st.stadium_card() {
            Some(c) => c,
            None => return Ok(()),
        };
        return discard_stadium(g, stadium, me);
    }
    if was_attack_used(g, e, 1, me) {
        if let Effect::Attack { p, opp, attack, source, .. } = *g.e(e) {
            let pp = p as usize;
            let target = SlotRef::new(pp, g.st.players[pp].active);
            let b = AtkBase { attack_effect: e, player: p, opponent: opp, attack, source, target };
            g.run_fx(Effect::DealDamage { b, damage: 10 })?;
        }
    }
    Ok(())
}
