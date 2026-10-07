//! Nighttime Mine (M2a): attacks used by each Tera Pokémon in play cost [C] more.
//!
//! Twinleaf: only the Active Pokémon of the effect's player is checked; the
//! extra [C] is inserted before the first existing [C] (else pushed last).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "NightMine", mask: mask(&[k::CHECK_ATTACK_COST, k::USE_STADIUM]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    // Automatically active: a Stadium without "that player may" can't be announced and used (Advanced Rulebook B-04).
    if let Effect::UseStadium { .. } = *g.e(e) {
        if g.st.stadium_card() == Some(me) {
            bail!("CANNOT_USE_STADIUM");
        }
    }
    let p = match *g.e(e) {
        Effect::CheckAttackCost { p, .. } if g.st.stadium_card() == Some(me) => p as usize,
        _ => return Ok(()),
    };
    let active = g.st.players[p].active;
    if is_stadium_effect_blocked(g, p, SlotRef::new(p, active), me) {
        return Ok(());
    }
    let tera = g.st.slot_pokemon(p, active).map(|c| g.st.cdef(c).has_tag(tag::POKEMON_TERA)).unwrap_or(false);
    if tera {
        if let Effect::CheckAttackCost { cost, .. } = g.e_mut(e) {
            match cost.iter().position(|t| *t == ct::COLORLESS) {
                Some(i) => cost.insert(i, ct::COLORLESS),
                None => cost.push(ct::COLORLESS),
            }
        }
    }
    Ok(())
}
