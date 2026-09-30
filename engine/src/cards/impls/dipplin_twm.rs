//! Dipplin (TWM 18): Festival Lead — if Festival Grounds is in play, this
//! Pokémon may use an attack twice. Do the Wave — 20 damage for each of your
//! Benched Pokémon.
//!
//! Twinleaf: Festival Lead is the attack's `barrage` flag, written on the
//! card object whenever Do the Wave's AttackEffect is reduced and the
//! Ability isn't blocked (so it is sticky while blocked). The damage is only
//! recomputed when the opponent's Active holds a Pokémon.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Dipplin@Dipplin TWM1", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

/// Festival Lead: `this.attacks[0].barrage = stadium is 'Festival Grounds'`
/// unless the Ability is blocked. `pristine_has_key`: the printed attack
/// object already has `barrage: false` (Dipplin), so only `true` differs
/// from the printed card in the canonical state; otherwise any write does.
pub fn festival_lead(g: &mut Game, p: usize, me: CardId, pristine_has_key: bool) {
    if is_ability_blocked(g, p, me, None) {
        return;
    }
    let fg = g.st.stadium_card().map(|s| g.st.cdef(s).name == "Festival Grounds").unwrap_or(false);
    let inst = &mut g.st.cards[me as usize];
    if fg {
        inst.attack_barrage |= 1;
    } else {
        inst.attack_barrage &= !1;
    }
    if fg || !pristine_has_key {
        inst.attack_barrage_shown |= 1;
    } else {
        inst.attack_barrage_shown &= !1;
    }
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !was_attack_used(g, e, 0, me) {
        return Ok(());
    }
    let (p, opp) = match *g.e(e) {
        Effect::Attack { p, opp, .. } => (p as usize, opp as usize),
        _ => return Ok(()),
    };
    if g.st.active_pokemon(opp).is_some() {
        let pl = &g.st.players[p];
        let benched = pl.bench.iter().filter(|b| !pl.slots[**b as usize].cards.is_empty()).count() as i32;
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage = benched * 20;
        }
    }
    festival_lead(g, p, me, true);
    Ok(())
}
