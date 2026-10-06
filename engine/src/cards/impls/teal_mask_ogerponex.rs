//! Teal Mask Ogerpon ex (TWM, Tera): Teal Dance - once during your turn,
//! attach a Basic [G] Energy card from your hand to this Pokémon; if you
//! did, draw a card. Myriad Leaf Shower - 30 + 30 for each Energy attached
//! to both Active Pokémon. Tera: no attack damage while on the Bench.
//!
//! Fixed (phase 4b, F1): the prompt took 0 cards; a "you may" Ability is declined by not
//! using it, so the attach is exactly 1 Energy (ruling 1778).
//!
//! Twinleaf counts the energy of each player's Active via
//! `CheckProvidedEnergyEffect(player)` (the Active by default).
use crate::cards::prelude::*;
use crate::marker;

pub static IMPL: CardImpl = CardImpl {
    class: "TealMaskOgerponex",
    mask: mask(&[k::PLAY_POKEMON, k::POWER, k::END_TURN, k::ATTACK, k::PUT_DAMAGE]),
    reduce,
    resume: Some(resume),
    coin: None,
    can_play: None,
};

fn teal_dance() -> crate::markers::MarkerName {
    marker!("TEAL_DANCE_MARKER")
}

fn energy_units(g: &mut Game, p: usize) -> R<i32> {
    let a = g.st.players[p].active;
    let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p: p as u8, source: SlotRef::new(p, a), energy_map: SVec::new() })?;
    Ok(match pe {
        Effect::CheckProvidedEnergy { energy_map, .. } => energy_map.iter().map(|e| e.provides.len() as i32).sum(),
        _ => 0,
    })
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::PlayPokemon { p, card, .. } = *g.e(e) {
        if card == me {
            g.st.players[p as usize].marker.remove_from(teal_dance(), me);
        }
    }

    if was_power_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Power { p, .. } => p as usize,
            _ => return Ok(()),
        };
        if g.st.players[p].marker.has_from(teal_dance(), me) {
            bail!("POWER_ALREADY_USED");
        }
        let has = g.st.players[p].hand.iter().any(|c| {
            let d = g.st.cdef(c);
            d.is_energy() && d.energy_type == EnergyType::Basic as u8 && d.provides.contains(&ct::GRASS)
        });
        if !has {
            bail!("CANNOT_USE_POWER");
        }
        let (sp, ss) = match g.st.find_pokemon_slot(me) {
            Some(x) => x,
            None => return Ok(()),
        };
        let mut filter = Filter::super_type(SuperType::Energy);
        filter.energy_type = Some(EnergyType::Basic as u8);
        filter.name = Some("Grass Energy");
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        f.a[1] = sp as i32;
        f.l[0] = ss;
        choose_cards(g, p, "CHOOSE_CARD_TO_ATTACH", ListRef::Hand(p as u8), filter, ChooseCardsOpts::new(1, 1, false), Cont::Card { card: me, frame: f });
        return Ok(());
    }

    if let Effect::EndTurn { p } = *g.e(e) {
        let m = &mut g.st.players[p as usize].marker;
        if m.has_from(teal_dance(), me) {
            m.remove_from(teal_dance(), me);
        }
    }

    if was_attack_used(g, e, 0, me) {
        let (p, o) = match *g.e(e) {
            Effect::Attack { p, opp, .. } => (p as usize, opp as usize),
            _ => return Ok(()),
        };
        let mine = energy_units(g, p)?;
        let theirs = energy_units(g, o)?;
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage += (mine + theirs) * 30;
        }
    }

    tera_rule(g, e, me);
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let slot = ListRef::Slot(f.a[1] as u8, f.l[0]);
    let cards: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
    if !cards.is_empty() {
        g.st.players[p].marker.add(teal_dance(), me, crate::markers::SourceType::None, crate::markers::TargetScope::None);
        ability_used(g, p, me);
        move_cards(g, ListRef::Hand(p as u8), slot, &cards, me)?;
        g.run_fx(Effect::MoveCards {
            source: ListRef::Deck(p as u8),
            destination: ListRef::Hand(p as u8),
            cards: None,
            count: Some(1),
            to_top: false,
            to_bottom: false,
            skip_cleanup: false,
            source_card: me,
        })?;
    }
    Ok(())
}
