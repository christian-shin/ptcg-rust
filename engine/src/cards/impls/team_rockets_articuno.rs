//! Team Rocket's Articuno (DRI): Repelling Veil — prevent all effects of the
//! opponent's attacks done to your Basic Team Rocket's Pokémon. Dark Frost —
//! 60+; 60 more if this Pokémon has Team Rocket's Energy attached.
//!
//! Twinleaf quirks kept: the Ability only prevents PutCountersEffect (during
//! the attack phase, when this card is the top card of one of the attacked
//! player's slots and the target is a Basic Team Rocket's Pokémon); it has
//! no Ability-lock check. Dark Frost looks for an Energy named
//! "Team Rocket's Energy" on the attacker's Active (fixed in phase 4b: it
//! compared against "Team Rocket Energy", so the bonus never applied).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "TeamRocketsArticuno", mask: mask(&[k::PUT_COUNTERS, k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::PutCounters { b, .. } = *g.e(e) {
        let o = b.opponent as usize;
        let in_play = for_each_pokemon(g, o, PlayerType::BottomPlayer).iter().any(|(_, c, _)| *c == me);
        if !in_play {
            return Ok(());
        }
        if g.st.phase == GamePhase::Attack {
            let t = b.target;
            if let Some(c) = g.st.slot_pokemon(t.p as usize, t.s) {
                let d = g.st.cdef(c);
                if d.stage == Stage::Basic as u8 && d.has_tag(tag::TEAM_ROCKET) {
                    g.set_prevent(e, true);
                }
            }
        }
    }
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let a = g.st.players[p].active;
        let has = g.st.slot(p, a).cards.iter().any(|c| {
            let d = g.st.cdef(c);
            d.is_energy() && d.name == "Team Rocket's Energy"
        });
        if has {
            if let Effect::Attack { damage, .. } = g.e_mut(e) {
                *damage += 60;
            }
        }
    }
    Ok(())
}
