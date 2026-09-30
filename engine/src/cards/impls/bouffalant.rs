//! Bouffalant (SCR): Curly Wall — if you have any other Bouffalant in play,
//! each of your Basic [C] Pokémon takes 60 less damage from your opponent's
//! attacks (after Weakness and Resistance); only 1 Curly Wall applies.
//! Boundless Power — 130 (the "can't attack" text is not implemented).
//!
//! Twinleaf quirks kept: every copy reacts from any zone (its owner is the
//! owner of the list holding it); it needs 2+ Pokémon named Bouffalant in
//! that owner's play, then an ability-lock probe; any PutDamageEffect on the
//! owner's Basic [C] Pokémon is reduced (also self-damage), once per effect
//! via the `nonstackingDamageReducers` source 'Curly Wall'.
use crate::cards::prelude::*;
use crate::game::fx_flag;

pub static IMPL: CardImpl = CardImpl { class: "Bouffalant", mask: mask(&[k::PUT_DAMAGE]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let b = match *g.e(e) {
        Effect::PutDamage { b, .. } => b,
        _ => return Ok(()),
    };
    let owner = match g.st.locate(me).and_then(|l| l.owner()) {
        Some(o) => o,
        None => return Ok(()),
    };
    let count = for_each_pokemon(g, owner, PlayerType::BottomPlayer).iter().filter(|(_, c, _)| g.st.cdef(*c).name == "Bouffalant").count();
    if count < 2 {
        return Ok(());
    }
    if is_ability_blocked(g, owner, me, None) {
        return Ok(());
    }
    let t = b.target;
    if let Some(tc) = g.st.slot_pokemon(t.p as usize, t.s) {
        let d = g.st.cdef(tc);
        if d.card_type.contains(&ct::COLORLESS) && d.stage == Stage::Basic as u8 && t.p as usize == owner {
            if g.fx_flags(e) & fx_flag::CURLY_WALL != 0 {
                return Ok(());
            }
            if let Effect::PutDamage { damage, .. } = g.e_mut(e) {
                *damage = (*damage - 60).max(0);
            }
            g.set_fx_flag(e, fx_flag::CURLY_WALL);
        }
    }
    Ok(())
}
