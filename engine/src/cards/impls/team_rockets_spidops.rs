//! Team Rocket's Spidops (DRI): Charge Up — once during your turn, attach a
//! Basic Energy from your discard pile to this Pokémon. Rocket Rush — 30
//! damage for each of your Team Rocket's Pokémon in play.
//!
//! Twinleaf: the once-per-turn marker (player marker sourced by this card)
//! is removed at every EndTurnEffect for the ending player; it is only added
//! (with the MOVE_CARDS) once a card is chosen. The discard check precedes
//! the marker check. No ABILITY_USED board effect.
use crate::cards::prelude::*;
use crate::marker;

pub static IMPL: CardImpl = CardImpl { class: "TeamRocketsSpidops", mask: mask(&[k::END_TURN, k::POWER, k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn charge_up() -> crate::markers::MarkerName {
    marker!("CHARGE_UP_MARKER")
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::EndTurn { p } = *g.e(e) {
        g.st.players[p as usize].marker.remove_from(charge_up(), me);
    }
    if was_power_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Power { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let has = g.st.players[p].discard.iter().any(|c| {
            let d = g.st.cdef(c);
            d.is_energy() && d.energy_type == EnergyType::Basic as u8
        });
        if !has {
            bail!("CANNOT_USE_POWER");
        }
        if g.st.players[p].marker.has_from(charge_up(), me) {
            bail!("CANNOT_USE_POWER");
        }
        let list = match g.st.locate(me) {
            Some(l @ ListRef::Slot(..)) => l,
            _ => return Ok(()),
        };
        let (lp, ls) = match list {
            ListRef::Slot(lp, ls) => (lp, ls),
            _ => unreachable!(),
        };
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        f.a[1] = lp as i32;
        f.a[2] = ls as i32;
        let filter = Filter { super_type: Some(SuperType::Energy as u8), energy_type: Some(EnergyType::Basic as u8), ..Default::default() };
        choose_cards(g, p, "CHOOSE_CARD_TO_ATTACH", ListRef::Discard(p as u8), filter, ChooseCardsOpts::new(1, 1, false), Cont::Card { card: me, frame: f });
        return Ok(());
    }
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let n = for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().filter(|(_, c, _)| g.st.cdef(*c).has_tag(tag::TEAM_ROCKET)).count() as i32;
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage = 30 * n;
        }
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let dst = ListRef::Slot(f.a[1] as u8, f.a[2] as SlotId);
    let cards: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
    if !cards.is_empty() {
        g.st.players[p].marker.add(charge_up(), me, crate::markers::SourceType::None, crate::markers::TargetScope::None);
        move_cards(g, ListRef::Discard(p as u8), dst, &cards, me)?;
    }
    Ok(())
}
