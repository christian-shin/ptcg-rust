//! Binding Mochi (SFA, tool): attacks used by the Poisoned Pokémon this card
//! is attached to do 40 more damage to your opponent's Active Pokémon.
//!
//! Twinleaf quirks kept: the Poisoned check reads the attacking player's
//! Active (not the tool's holder); the tool block probe is a bare ToolEffect
//! (no `stadiumAndToolHaveNoEffectTurnsRemaining` check).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "BindingMochi", mask: mask(&[k::DEAL_DAMAGE]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let b = match *g.e(e) {
        Effect::DealDamage { b, .. } => b,
        _ => return Ok(()),
    };
    if !g.st.slot(b.source.p as usize, b.source.s).tools.contains(me) {
        return Ok(());
    }
    let p = b.player as usize;
    let o = 1 - p;
    let t = b.target;
    let is_active = (t.p as usize == p && t.s == g.st.players[p].active) || (t.p as usize == o && t.s == g.st.players[o].active);
    if !is_active {
        return Ok(());
    }
    if g.run_fx(Effect::Tool { p: p as u8, card: me }).is_err() {
        return Ok(());
    }
    let pa = g.st.players[p].active;
    if g.st.slot(p, pa).special_conditions.contains(&(SpecialCondition::Poisoned as u8)) {
        if let Some(oc) = g.st.active_pokemon(o) {
            if g.st.slot(t.p as usize, t.s).cards.contains(oc) {
                if let Effect::DealDamage { damage, .. } = g.e_mut(e) {
                    *damage += 40;
                }
            }
        }
    }
    Ok(())
}
