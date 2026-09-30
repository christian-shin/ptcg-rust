//! Mega Darkrai ex (M5 / PBL 48): Dusk Raid — 110+; 110 more if any of your
//! Benched Pokémon has damage counters. Abyss Eye — if the opponent's Active
//! is affected by a Special Condition, it is Knocked Out.
//!
//! Twinleaf clears the Active's Special Conditions before reducing the
//! (Mist-blockable) KnockOutOpponentEffect.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "MegaDarkraiex", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        if let Effect::Attack { p, .. } = *g.e(e) {
            let pl = &g.st.players[p as usize];
            let damaged = pl.bench.iter().any(|b| !pl.slots[*b as usize].cards.is_empty() && pl.slots[*b as usize].damage > 0);
            if damaged {
                if let Effect::Attack { damage, .. } = g.e_mut(e) {
                    *damage += 110;
                }
            }
        }
    }
    if was_attack_used(g, e, 1, me) {
        if let Effect::Attack { p, opp, attack, source, .. } = *g.e(e) {
            let o = opp as usize;
            let a = g.st.players[o].active;
            if !g.st.slot(o, a).special_conditions.is_empty() {
                g.st.players[o].slots[a as usize].special_conditions.clear();
                let b = AtkBase { attack_effect: e, player: p, opponent: opp, attack, source, target: SlotRef::new(o, a) };
                g.run_fx(Effect::KnockOutOpponent { b, knocked_out: false, prize_count: 0 })?;
            }
        }
    }
    Ok(())
}
