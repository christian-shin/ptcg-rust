//! Koraidon ex (ASC, Tera): Orichalcum Fang - 50+, 120 more if any of your
//! Pokémon were Knocked Out by attack damage during your opponent's last
//! turn. Impact Blow - 200; can't use Impact Blow during your next turn.
//!
//! Twinleaf has two unrelated `Koraidonex` classes (ASC and TEF); this port
//! is registered for the ASC one only.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Koraidonex@ASC", mask: mask(&[k::ATTACK, k::PUT_DAMAGE]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        // WAS_POKEMON_KNOCKED_OUT_DURING_OPPONENTS_LAST_TURN(player, { byAttackDamage: true })
        if g.st.players[p].pokemon_knocked_out_by_attack_during_opponents_last_turn {
            if let Effect::Attack { damage, .. } = g.e_mut(e) {
                *damage += 120;
            }
        }
        return Ok(());
    }
    if was_attack_used(g, e, 1, me) {
        if let Effect::Attack { p, .. } = *g.e(e) {
            let p = p as usize;
            let a = g.st.players[p].active;
            g.st.players[p].slots[a as usize].cannot_use_attacks_next_turn_pending.push("Impact Blow");
        }
    }
    tera_rule(g, e, me);
    Ok(())
}
