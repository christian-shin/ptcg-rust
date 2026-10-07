//! Magmortar (JTG): Magma Surge — during Pokémon Checkup, put 3 more damage
//! counters on your opponent's Burned Pokémon. Searing Flame — 90; flip a
//! coin, if heads the opponent's Active Pokémon is now Burned.
//!
//! Twinleaf: on each BetweenTurnsEffect the owner is found by scanning
//! [player, opponent] (last match wins), a generic ability probe is run for
//! the owner, and `burnDamage += 30` when the ending player is the owner's
//! opponent and its Active is Burned.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Magmortar", mask: mask(&[k::BETWEEN_TURNS, k::AFTER_ATTACK]), reduce, resume: None, coin: Some(coin), can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::BetweenTurns { p: current, .. } = *g.e(e) {
        let current = current as usize;
        let mut owner = None;
        for q in [current, 1 - current] {
            for (_, c, _) in for_each_pokemon(g, q, PlayerType::BottomPlayer).iter().copied() {
                if c == me {
                    owner = Some(q);
                }
            }
        }
        let owner = match owner {
            Some(o) => o,
            None => return Ok(()),
        };
        if is_ability_blocked(g, owner, me, None) {
            return Ok(());
        }
        let victim = 1 - owner;
        let va = g.st.players[victim].active;
        if current == victim && g.st.slot(victim, va).special_conditions.contains(&(SpecialCondition::Burned as u8)) {
            if let Effect::BetweenTurns { burn_damage, .. } = g.e_mut(e) {
                *burn_damage += 30;
            }
        }
    }

    if after_attack_used(g, e, 0, me) {
        let e = real_attack(g, e);
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        g.retain_fx(e);
        let mut f = CardFrame::at(1);
        f.e[0] = e;
        if let Err(err) = g.coin_flip(p, CoinCb::Card { card: me, frame: f }) {
            g.release_fx(e);
            return Err(err);
        }
    }
    Ok(())
}

fn coin(g: &mut Game, _me: CardId, f: CardFrame, heads: bool) -> R {
    let atk = f.e[0];
    let r = if heads { add_special_conditions_to_opponent_active(g, atk, &[SpecialCondition::Burned]) } else { Ok(()) };
    g.release_fx(atk);
    r
}
