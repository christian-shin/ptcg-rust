//! Haxorus (SV11B / BLK 70): Cross-Cut — 80+; 80 more if the opponent's
//! Active is not a Basic Pokémon. Axe Bomber — if the opponent's Active is a
//! Basic Pokémon it is Knocked Out (Mist-blockable KnockOutOpponentEffect).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Haxorus@Haxorus SV11B", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        if let Effect::Attack { opp, .. } = *g.e(e) {
            if let Some(c) = g.st.active_pokemon(opp as usize) {
                if g.st.cdef(c).stage != Stage::Basic as u8 {
                    if let Effect::Attack { damage, .. } = g.e_mut(e) {
                        *damage += 80;
                    }
                }
            }
        }
        return Ok(());
    }
    if was_attack_used(g, e, 1, me) {
        if let Effect::Attack { p, opp, attack, source, .. } = *g.e(e) {
            let o = opp as usize;
            if let Some(c) = g.st.active_pokemon(o) {
                if g.st.cdef(c).stage == Stage::Basic as u8 {
                    let a = g.st.players[o].active;
                    let b = AtkBase { attack_effect: e, player: p, opponent: opp, attack, source, target: SlotRef::new(o, a) };
                    g.run_fx(Effect::KnockOutOpponent { b, knocked_out: false, prize_count: 0 })?;
                }
            }
        }
    }
    Ok(())
}
