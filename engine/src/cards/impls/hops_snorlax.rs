//! Hop's Snorlax (JTG): Extra Helpings — attacks used by your Hop's Pokémon
//! do 30 more damage to your opponent's Active Pokémon (before Weakness and
//! Resistance); doesn't stack. Dynamic Press — 140; 80 damage to itself.
//!
//! Twinleaf: any DealDamageEffect whose attacker (`effect.player`) has this
//! card as the top Pokémon of a slot; after the ability-lock probe, adds 30
//! when the attacker's current Active is a Hop's Pokémon, the target is the
//! opponent's Active and the effect's `damageIncreased` flag is unset (then
//! sets it). The self-damage DealDamageEffect also passes through here.
use crate::cards::prelude::*;
use crate::game::fx_flag;

pub static IMPL: CardImpl = CardImpl { class: "HopsSnorlax", mask: mask(&[k::ATTACK, k::DEAL_DAMAGE]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        if let Effect::Attack { p, opp, attack, source, .. } = *g.e(e) {
            let target = SlotRef::new(p as usize, g.st.players[p as usize].active);
            let b = AtkBase { attack_effect: e, player: p, opponent: opp, attack, source, target };
            g.run_fx(Effect::DealDamage { b, damage: 80 })?;
        }
        return Ok(());
    }
    if let Effect::DealDamage { b, .. } = *g.e(e) {
        let p = b.player as usize;
        if !for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().any(|(_, c, _)| *c == me) {
            return Ok(());
        }
        let o = 1 - p;
        let hops = g.st.active_pokemon(p);
        // snorlaxCount >= 1 here (this card is in play).
        if is_ability_blocked(g, p, me, None) {
            return Ok(());
        }
        if let Some(h) = hops {
            if g.st.cdef(h).has_tag(tag::HOPS)
                && b.target.p as usize == o
                && b.target.s == g.st.players[o].active
                && g.fx_flags(e) & fx_flag::DAMAGE_INCREASED == 0
            {
                if let Effect::DealDamage { damage, .. } = g.e_mut(e) {
                    *damage += 30;
                }
                g.set_fx_flag(e, fx_flag::DAMAGE_INCREASED);
            }
        }
    }
    Ok(())
}
