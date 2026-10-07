//! Hydrapple ex (SCR): Ripening Charge — once during your turn, attach a
//! Basic [G] Energy card from your hand to 1 of your Pokémon and heal 30
//! damage from it. Syrup Storm — 30+, 30 more for each [G] Energy attached
//! to all of your Pokémon.
//!
//! Fixed (R1-18): cancelling the attach prompt doesn't use the Ability up (the
//! once-per-turn marker and the ABILITY_USED board effect used to be set
//! anyway). Twinleaf quirk kept: only the first transfer is processed. Syrup Storm counts `provides` entries equal to
//! [G] or ANY across every CheckProvidedEnergy map.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "Hydrappleex",
    mask: mask(&[k::PLAY_POKEMON, k::END_TURN, k::POWER, k::ATTACK]),
    reduce,
    resume: Some(resume),
    coin: None,
    can_play: None,
};

fn ripe() -> crate::markers::MarkerName {
    crate::marker!("RIPE_CHARGE_MARKER")
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::PlayPokemon { p, card, .. } = *g.e(e) {
        if card == me {
            g.st.players[p as usize].marker.remove_from(ripe(), me);
        }
    }

    if let Effect::EndTurn { p } = *g.e(e) {
        g.st.players[p as usize].marker.remove_from(ripe(), me);
    }

    if was_power_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Power { p, .. } => p as usize,
            _ => return Ok(()),
        };
        if g.st.players[p].marker.has_from(ripe(), me) {
            bail!("POWER_ALREADY_USED");
        }
        let has = g.st.players[p].hand.iter().any(|c| {
            let d = g.st.cdef(c);
            d.is_energy() && d.energy_type == EnergyType::Basic as u8 && d.provides.contains(&ct::GRASS)
        });
        if !has {
            bail!("CANNOT_USE_POWER");
        }
        let mut slots = SVec::new();
        slots.push(SlotType::Active as u8);
        slots.push(SlotType::Bench as u8);
        let mut o = AttachOpts::new(g.st.players[p].hand.len() as u8);
        o.min = 1;
        o.max = 1;
        o.allow_cancel = true;
        let filter = Filter {
            super_type: Some(SuperType::Energy as u8),
            energy_type: Some(EnergyType::Basic as u8),
            name: Some("Grass Energy"),
            ..Filter::none()
        };
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        let id = g.player_id(p);
        g.prompt(
            id,
            "ATTACH_ENERGY_CARDS",
            PromptKind::AttachEnergy { cards: ListRef::Hand(p as u8), player_type: PlayerType::BottomPlayer, slots, filter, o },
            Cont::Card { card: me, frame: f },
        );
        return Ok(());
    }

    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let mut count = 0i32;
        for (s, _, _) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
            let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p: p as u8, source: SlotRef::new(p, s), energy_map: SVec::new() })?;
            if let Effect::CheckProvidedEnergy { energy_map, .. } = pe {
                for em in energy_map.iter() {
                    count += em.provides.iter().filter(|t| **t == ct::GRASS || **t == ct::ANY).count() as i32;
                }
            }
        }
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage += count * 30;
        }
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let transfers: SVec<(CardTarget, CardId), 64> = match results.first() {
        Some(Res::Attach(t)) => *t,
        _ => SVec::new(),
    };
    // R1-18: declining the attachment doesn't use the Ability up.
    if transfers.is_empty() {
        return Ok(());
    }
    g.st.players[p].marker.add(ripe(), me, crate::markers::SourceType::None, crate::markers::TargetScope::None);
    ability_used(g, p, me);
    if let Some((to, c)) = transfers.iter().copied().next() {
        let target = get_target(&g.st, p, to)?;
        g.run_fx(Effect::AttachEnergy { p: p as u8, card: c, target })?;
        g.run_fx(Effect::Heal { p: p as u8, target, damage: 30 })?;
    }
    Ok(())
}
