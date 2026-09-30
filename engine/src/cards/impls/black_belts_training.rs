//! Black Belt's Training (PRE / JTG): during this turn, attacks used by your
//! Pokémon do 40 more damage to your opponent's Active Pokémon ex (before
//! Weakness and Resistance).
//!
//! Twinleaf: the card moves itself to the supporter pile and marks the
//! player; each copy only honors its own marker. The bonus applies to any
//! DealDamageEffect of a marked player whose target is an Active Pokémon ex.
use crate::cards::prelude::*;
use crate::marker;

pub static IMPL: CardImpl = CardImpl {
    class: "BlackBeltsTraining",
    mask: mask(&[k::TRAINER, k::DEAL_DAMAGE, k::END_TURN]),
    reduce,
    resume: None,
    coin: None,
    can_play: None,
};

fn bbt() -> crate::markers::MarkerName {
    marker!("BLACK_BELTS_TRAINING_MARKER")
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Some(p) = trainer_played(g, e, me) {
        if g.st.players[p].supporter_turn > 0 {
            bail!("SUPPORTER_ALREADY_PLAYED");
        }
        move_cards(g, ListRef::Hand(p as u8), ListRef::Supporter(p as u8), &[me], me)?;
        g.st.players[p].marker.add(bbt(), me, crate::markers::SourceType::None, crate::markers::TargetScope::None);
    }

    if let Effect::DealDamage { b, damage } = *g.e(e) {
        let p = b.player as usize;
        if g.st.players[p].marker.has_from(bbt(), me) && damage > 0 {
            let t = b.target;
            if let Some(c) = g.st.slot_pokemon(t.p as usize, t.s) {
                if g.st.cdef(c).has_tag(tag::POKEMON_EX_LOWER) {
                    let o = 1 - p;
                    let is_active = (t.p as usize == p && t.s == g.st.players[p].active) || (t.p as usize == o && t.s == g.st.players[o].active);
                    if !is_active {
                        return Ok(());
                    }
                    if let Effect::DealDamage { damage, .. } = g.e_mut(e) {
                        *damage += 40;
                    }
                }
            }
        }
    }

    if let Effect::EndTurn { p } = *g.e(e) {
        let m = &mut g.st.players[p as usize].marker;
        if m.has_from(bbt(), me) {
            m.remove_from(bbt(), me);
        }
    }
    Ok(())
}
