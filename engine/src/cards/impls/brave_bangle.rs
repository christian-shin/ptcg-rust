//! Brave Bangle (SV11W, tool): if the holder doesn't have a Rule Box, its
//! attacks do 30 more damage to the opponent's Active Pokémon ex (before W/R).
//!
//! Twinleaf: the tool block probe is a bare ToolEffect; the Rule Box check
//! is on the attacking slot's cards; the attack's printed damage must be
//! above 0.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "BraveBangle", mask: mask(&[k::DEAL_DAMAGE]), reduce, resume: None, coin: None, can_play: None };

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
    if g.run_fx(Effect::Tool { p: p as u8, card: me }).is_err() {
        return Ok(());
    }
    let t = b.target;
    let on_opp_active = t.p as usize == o && t.s == g.st.players[o].active;
    let on_my_active = t.p as usize == p && t.s == g.st.players[p].active;
    if !on_opp_active && !on_my_active {
        return Ok(());
    }
    let rule_box = g.st.slot(b.source.p as usize, b.source.s).cards.iter().any(|c| g.st.cdef(c).has_rule_box());
    if rule_box {
        return Ok(());
    }
    let target_ex = g.st.slot_pokemon(t.p as usize, t.s).map(|c| g.st.cdef(c).has_tag(tag::POKEMON_EX_LOWER)).unwrap_or(false);
    if target_ex && crate::engine::attack::attack_def(g, b.attack).damage > 0 && on_opp_active {
        if let Effect::DealDamage { damage, .. } = g.e_mut(e) {
            *damage += 30;
        }
    }
    Ok(())
}
