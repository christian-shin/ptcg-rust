//! Gastly (EVO): Little Grudge — during your opponent's next turn, if this
//! Pokémon is Knocked Out by damage from an attack, discard an Energy
//! attached to the Attacking Pokémon. Nightmare — 20; flip a coin, if heads
//! your opponent's Active Pokémon is now Asleep.
//!
//! Twinleaf: Little Grudge arms the slot fields through a
//! DiscardAttackerEnergyIfKnockedOut EffectOfAttack (resolved in the
//! KnockOutEffect reducer); Nightmare flips on AFTER_ATTACK and applies an
//! AddSpecialConditionsPowerEffect.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Gastly", mask: mask(&[k::ATTACK, k::AFTER_ATTACK]), reduce, resume: None, coin: Some(coin), can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        return discard_attacker_energy_if_knocked_out(g, e, me);
    }
    if after_attack_used(g, e, 1, me) {
        let (p, o) = match *g.e(e) {
            Effect::AfterAttack { p, opp, .. } => (p as usize, opp as usize),
            _ => return Ok(()),
        };
        let mut f = CardFrame::at(1);
        f.a[0] = o as i32;
        g.coin_flip(p, CoinCb::Card { card: me, frame: f })?;
    }
    Ok(())
}

fn coin(g: &mut Game, me: CardId, f: CardFrame, heads: bool) -> R {
    if f.stage == 1 && heads {
        add_special_conditions_to_player_active(g, f.a[0] as usize, me, &[SpecialCondition::Asleep])?;
    }
    Ok(())
}
