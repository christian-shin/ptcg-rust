//! Mega Hawlucha ex (M2a / ASC 116): Tenacious Body — if this Pokémon would
//! be Knocked Out by damage from an attack, flip a coin; heads, it survives
//! with 10 HP. Somersault Dive — 120+; 140 more if a Stadium is in play,
//! then discard that Stadium.
//!
//! Twinleaf quirk kept (as Annihilape M5's Durable Body): the coin's
//! callback sets `surviveOnTenHPReason` after the flip's wait prompt, i.e.
//! after the PutDamageEffect was already applied, so the flip happens (when
//! `effect.damage >=` the CheckHpEffect HP, ignoring existing damage) but
//! never saves the Pokémon. On heads the callback reads
//! `this.powers[0].name`, which throws for a power-less copycat.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "MegaHawluchaex",
    mask: mask(&[k::PUT_DAMAGE, k::ATTACK]),
    reduce,
    resume: None,
    coin: Some(coin),
    can_play: None,
};

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::PutDamage { b, damage, .. } = *g.e(e) {
        let t = b.target;
        if g.st.slot(t.p as usize, t.s).cards.contains(me) {
            let owner = t.p as usize;
            if is_ability_blocked(g, owner, me, None) {
                return Ok(());
            }
            let hp = crate::engine::check::check_hp(g, owner, t.s)?;
            if damage >= hp {
                g.coin_flip(owner, CoinCb::Card { card: me, frame: CardFrame::at(1) })?;
                return Ok(());
            }
        }
    }

    if was_attack_used(g, e, 0, me) {
        if let Some(stadium) = g.st.stadium_card() {
            if let Effect::Attack { damage, .. } = g.e_mut(e) {
                *damage += 140;
            }
            super::hisuian_growlithe::discard_stadium(g, stadium, me)?;
        }
    }
    Ok(())
}

fn coin(g: &mut Game, me: CardId, _f: CardFrame, heads: bool) -> R {
    // `effect.surviveOnTenHPReason = this.powers[0].name` (no rules effect).
    if heads && g.st.cdef(me).powers.is_empty() {
        bail!("Cannot read properties of undefined (reading 'name')");
    }
    Ok(())
}
