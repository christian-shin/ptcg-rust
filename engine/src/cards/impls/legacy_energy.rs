//! Legacy Energy (TWM, ACE SPEC): provides every type of Energy but only 1
//! at a time. If the Pokémon this card is attached to is Knocked Out by
//! damage from an attack from your opponent's Pokémon, that player takes 1
//! fewer Prize card (once per game).
//!
//! Twinleaf: provides [ANY] on every CheckProvidedEnergyEffect of its slot.
//! On a KnockOutEffect of its slot during the ATTACK phase of the KO'd
//! Pokémon's opponent (any KO then, not only from damage), unless the
//! special energy is blocked, `prizeCount -= 1` once per game
//! (`player.legacyEnergyUsed`).
use crate::cards::prelude::*;
use crate::effects::EnergyEntry;

pub static IMPL: CardImpl = CardImpl {
    class: "LegacyEnergy",
    mask: mask(&[k::CHECK_PROVIDED_ENERGY, k::KNOCK_OUT]),
    reduce,
    resume: None,
    coin: None,
    can_play: None,
};

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    match *g.e(e) {
        Effect::CheckProvidedEnergy { source, .. } => {
            if g.st.slot(source.p as usize, source.s).cards.contains(me) {
                let mut provides = SVec::new();
                provides.push(ct::ANY);
                if let Effect::CheckProvidedEnergy { energy_map, .. } = g.e_mut(e) {
                    energy_map.push(EnergyEntry { card: me, provides });
                }
            }
        }
        Effect::KnockOut { p, target, .. } => {
            if !g.st.slot(target.p as usize, target.s).cards.contains(me) {
                return Ok(());
            }
            let p = p as usize;
            if g.st.phase != GamePhase::Attack || g.st.active_player as usize != 1 - p {
                return Ok(());
            }
            if is_special_energy_blocked(g, p, me, target, false) {
                return Ok(());
            }
            if !g.st.players[p].legacy_energy_used {
                if let Effect::KnockOut { prize_count, .. } = g.e_mut(e) {
                    *prize_count -= 1;
                }
                g.st.players[p].legacy_energy_used = true;
            }
        }
        _ => {}
    }
    Ok(())
}
