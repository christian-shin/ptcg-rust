//! Ethan's Ho-Oh ex (DRI / ASC): Golden Flame — once during your turn, you
//! may attach up to 2 Basic [R] Energy from your hand to 1 of your Benched
//! Ethan's Pokémon. Shining Feather — 160; heal 50 damage from each of your
//! Pokémon.
//!
//! Twinleaf quirks kept: the marker check throws BLOCKED_BY_EFFECT; the
//! prompt filter is `name: 'Fire Energy'` (not validated), blockedTo lists
//! every non-Ethan's Pokémon, min 0; the marker and the ability animation
//! happen when the prompt resolves, even with 0 Energy chosen; each Energy
//! is its own MOVE_CARDS (no AttachEnergyEffect). The attack heals each of
//! your Pokémon with a HealEffect.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "EthansHoOhex",
    mask: mask(&[k::PLAY_POKEMON, k::POWER, k::END_TURN, k::ATTACK]),
    reduce,
    resume: Some(resume),
    coin: None,
    can_play: None,
};

fn feather() -> crate::markers::MarkerName {
    crate::marker!("SHINING_FEATHER_MARKER")
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::PlayPokemon { p, card, .. } = *g.e(e) {
        if card == me {
            g.st.players[p as usize].marker.remove_from(feather(), me);
        }
    }

    if was_power_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Power { p, .. } => p as usize,
            _ => return Ok(()),
        };
        if g.st.players[p].marker.has_from(feather(), me) {
            bail!("BLOCKED_BY_EFFECT");
        }
        let has_energy = g.st.players[p].hand.iter().any(|c| {
            let d = g.st.cdef(c);
            d.is_energy() && d.energy_type == EnergyType::Basic as u8 && d.provides.contains(&ct::FIRE)
        });
        if !has_energy {
            bail!("CANNOT_USE_POWER");
        }
        let mut o = AttachOpts::new(g.st.players[p].hand.len() as u8);
        o.allow_cancel = false;
        o.same_target = true;
        o.min = 0;
        o.max = 2;
        for (_, c, t) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
            if !g.st.cdef(c).has_tag(tag::ETHANS) {
                o.blocked_to.push(t);
            }
        }
        let filter = Filter {
            super_type: Some(SuperType::Energy as u8),
            energy_type: Some(EnergyType::Basic as u8),
            name: Some("Fire Energy"),
            ..Filter::none()
        };
        let mut slots = SVec::new();
        slots.push(SlotType::Bench as u8);
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        let id = g.player_id(p);
        g.prompt(
            id,
            "ATTACH_ENERGY_TO_ACTIVE",
            PromptKind::AttachEnergy { cards: ListRef::Hand(p as u8), player_type: PlayerType::BottomPlayer, slots, filter, o },
            Cont::Card { card: me, frame: f },
        );
    }

    remove_marker_at_end_of_turn(g, e, feather(), me);

    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        for (s, _, _) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
            g.run_fx(Effect::Heal { p: p as u8, target: SlotRef::new(p, s), damage: 50 })?;
        }
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let transfers: SVec<(CardTarget, CardId), 16> = match results.first() {
        Some(Res::Attach(t)) => *t,
        _ => SVec::new(),
    };
    g.st.players[p].marker.add(feather(), me, crate::markers::SourceType::None, crate::markers::TargetScope::None);
    ability_used(g, p, me);
    for (to, c) in transfers.iter().copied() {
        let target = get_target(&g.st, p, to)?;
        move_cards(g, ListRef::Hand(p as u8), target.list(), &[c], me)?;
    }
    Ok(())
}
