//! Mega Hawlucha ex (M2a / ASC 116): Tenacious Body — if this Pokémon would
//! be Knocked Out by damage from an attack, flip a coin; heads, it survives
//! with 10 HP. Somersault Dive — 120+; 140 more if a Stadium is in play,
//! then discard that Stadium.
//!
//! Fixed (phase 4b, R6): the Stadium used to be discarded in the attack
//! handler, before the damage (so a Stadium such as Neutralization Zone or
//! Lively Stadium no longer affected the damage of this attack); it is now
//! discarded in AFTER_ATTACK, after the damage.
//!
//! Fixed (phase 4b, X1-1): as Annihilape M5's Durable Body, the coin's
//! callback used to set `surviveOnTenHPReason` after the PutDamageEffect was
//! applied (and the would-KO test ignored the damage already on the Pokémon),
//! so the flip never saved the Pokémon. The flip is now read right away
//! (SURVIVE_ON_TEN_ON_COIN_FLIP).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "MegaHawluchaex",
    mask: mask(&[k::PUT_DAMAGE, k::ATTACK, k::AFTER_ATTACK]),
    reduce,
    resume: None,
    coin: None,
    can_play: None,
};

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::PutDamage { b, .. } = *g.e(e) {
        let t = b.target;
        if g.st.slot(t.p as usize, t.s).cards.contains(me) {
            let owner = t.p as usize;
            if is_ability_blocked(g, owner, me, None) {
                return Ok(());
            }
            crate::prefabs::survive_on_ten_on_coin_flip(g, e, owner)?;
        }
    }

    if was_attack_used(g, e, 0, me) {
        if g.st.stadium_card().is_some() {
            if let Effect::Attack { damage, .. } = g.e_mut(e) {
                *damage += 140;
            }
        }
    }

    if after_attack_used(g, e, 0, me) {
        if let Some(stadium) = g.st.stadium_card() {
            super::hisuian_growlithe::discard_stadium(g, stadium, me)?;
        }
    }
    Ok(())
}
