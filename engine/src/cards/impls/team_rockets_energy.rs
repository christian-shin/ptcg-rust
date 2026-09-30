//! Team Rocket's Energy (DRI, Special): can only be attached to a Team
//! Rocket's Pokémon (discarded from anything else); provides 2 in any
//! combination of [P] and [D].
//!
//! Twinleaf: attaching to a slot whose top card is not a Team Rocket's
//! Pokémon throws. The table-state discard skips copies whose
//! SpecialEnergyEffect probe (for the slot's owner) is blocked. The two
//! [P]/[D] entries are only added if the EnergyEffect probe (for the checked
//! player) passes; otherwise the core adds the printed [C].
use crate::cards::prelude::*;
use crate::effects::EnergyEntry;

pub static IMPL: CardImpl = CardImpl {
    class: "TeamRocketsEnergy",
    mask: mask(&[k::ATTACH_ENERGY, k::CHECK_TABLE_STATE, k::CHECK_PROVIDED_ENERGY]),
    reduce,
    resume: None,
    coin: None,
    can_play: None,
};

fn is_rocket(g: &Game, t: SlotRef) -> bool {
    g.st.slot_pokemon(t.p as usize, t.s).map(|c| g.st.cdef(c).has_tag(tag::TEAM_ROCKET)).unwrap_or(false)
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    match *g.e(e) {
        Effect::AttachEnergy { card, target, .. } => {
            if card == me && !is_rocket(g, target) {
                bail!("CANNOT_PLAY_THIS_CARD");
            }
        }
        Effect::CheckTableState { .. } => {
            for p in 0..2usize {
                for (s, _, _) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
                    let t = SlotRef::new(p, s);
                    if !g.st.slot(p, s).cards.contains(me) || is_special_energy_blocked(g, p, me, t, false) {
                        continue;
                    }
                    if g.st.slot_pokemon(p, s).is_some() && !is_rocket(g, t) {
                        move_cards(g, t.list(), ListRef::Discard(p as u8), &[me], me)?;
                    }
                }
            }
        }
        Effect::CheckProvidedEnergy { p, source, .. } => {
            if !g.st.slot(source.p as usize, source.s).cards.contains(me) {
                return Ok(());
            }
            if g.run_fx(Effect::Energy { p, card: me }).is_err() {
                return Ok(());
            }
            let mut provides = SVec::new();
            provides.push(ct::PSYCHIC);
            provides.push(ct::DARK);
            if let Effect::CheckProvidedEnergy { energy_map, .. } = g.e_mut(e) {
                energy_map.push(EnergyEntry { card: me, provides });
                energy_map.push(EnergyEntry { card: me, provides });
            }
        }
        _ => {}
    }
    Ok(())
}
