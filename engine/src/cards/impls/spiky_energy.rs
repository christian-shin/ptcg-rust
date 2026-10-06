//! Spiky Energy (JTG): provides [C]. If the Pokémon this card is attached to
//! is in the Active Spot and is damaged by an attack from your opponent's
//! Pokémon (even if it is Knocked Out), put 2 damage counters on the
//! Attacking Pokémon.
//!
//! Twinleaf: the block check is made for the attacking player; the counters
//! are a PutCountersEffect on the attacker.
//!
//! Fixed (phase 4b, R7F-15): it reacted to DealDamageEffect (before any damage
//! is put, whatever its amount), so it also fired when the damage was
//! prevented (Crustle's Mysterious Rock Inn, ...) or reduced to 0 although the
//! Pokémon was not damaged. It now reacts to AfterDamageEffect, like Punk
//! Helmet and Lucky Helmet (text: "is damaged by an attack"; rulings 1646,
//! 1839: it stacks and works wherever the Pokémon end up).
//!
//! Step 7 of the attack flow chart (F1): the damage records the trigger (`Game::attack_trigger`) and it resolves
//! after every effect of the attack's own text and its prompts (AttackTrigger): it needs this card to be still
//! attached to the damaged Pokémon (an attack that discards it stops it, ruling 1649), the Special Energy not
//! blocked, and the Attacking Pokémon still in play, wherever it is (rulings 530, 1839).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "SpikyEnergy", mask: mask(&[k::AFTER_DAMAGE, k::ATTACK_TRIGGER]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    match *g.e(e) {
        Effect::AfterDamage { b, damage } => {
            let t = b.target;
            if !g.st.slot(t.p as usize, t.s).cards.contains(me) || g.st.phase != GamePhase::Attack {
                return Ok(());
            }
            if t.p == b.player || g.st.players[t.p as usize].active != t.s {
                return Ok(());
            }
            g.attack_trigger(b, damage, me, None, false)
        }
        Effect::AttackTrigger { attack_effect, p, opp, attack, card, target, source, source_in_play, retaliate: None, .. } if card == me => {
            if !g.st.slot(target.p as usize, target.s).cards.contains(me) {
                return Ok(());
            }
            if is_special_energy_blocked(g, p as usize, me, target, false) {
                return Ok(());
            }
            if !source_in_play {
                return Ok(());
            }
            let pb = AtkBase { attack_effect, player: p, opponent: opp, attack, source, target: source };
            g.run_fx(Effect::PutCounters { b: pb, damage: 20 })?;
            Ok(())
        }
        _ => Ok(()),
    }
}
