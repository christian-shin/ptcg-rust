//! Munkidori (TWM): Adrena-Brain — once during your turn, if this Pokémon has
//! any [D] Energy attached, move up to 3 damage counters from 1 of your
//! Pokémon to 1 of your opponent's Pokémon. Mind Bend — Confused.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "Munkidori",
    mask: mask(&[k::ATTACK, k::PLAY_POKEMON, k::END_TURN, k::POWER]),
    reduce,
    resume: Some(resume),
    coin: None,
    can_play: None,
};

fn adrena() -> crate::markers::MarkerName {
    crate::marker!("ADRENA_BRAIN_MARKER")
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        if let Effect::Attack { p, opp, attack, source, .. } = *g.e(e) {
            let target = SlotRef::new(opp as usize, g.st.players[opp as usize].active);
            let b = AtkBase { attack_effect: e, player: p, opponent: opp, attack, source, target };
            let mut conditions = SVec::new();
            conditions.push(SpecialCondition::Confused as u8);
            g.run_fx(Effect::AddSpecialConditions { b, conditions, poison_damage: None, burn_damage: None, confusion_damage: None })?;
        }
    }

    if let Effect::PlayPokemon { p, card, .. } = *g.e(e) {
        if card == me {
            g.st.players[p as usize].marker.remove_from(adrena(), me);
        }
    }

    if let Effect::EndTurn { p } = *g.e(e) {
        let m = &mut g.st.players[p as usize].marker;
        if m.has_from(adrena(), me) {
            m.remove_from(adrena(), me);
        }
    }

    if was_power_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Power { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let o = 1 - p;
        if g.st.players[p].marker.has_from(adrena(), me) {
            bail!("CANNOT_USE_POWER");
        }
        let mine = for_each_pokemon(g, p, PlayerType::BottomPlayer);
        if !mine.iter().any(|(s, _, _)| g.st.slot(p, *s).damage > 0) {
            bail!("CANNOT_USE_POWER");
        }
        let mut max_allowed: SVec<(CardTarget, i32), 16> = SVec::new();
        for (s, _, t) in mine.iter().copied() {
            let hp = crate::engine::check::check_hp(g, p, s)?;
            max_allowed.push((t, hp));
        }
        let mut o_opts = MoveOpts { allow_cancel: false, min: 1, max: Some(3), ..Default::default() };
        for (_, _, t) in mine.iter().copied() {
            o_opts.blocked_to.push(t);
        }
        for (_, _, t) in for_each_pokemon(g, o, PlayerType::TopPlayer).iter().copied() {
            o_opts.blocked_from.push(t);
        }
        for (s, c, _) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
            if c != me {
                continue;
            }
            let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p: p as u8, source: SlotRef::new(p, s), energy_map: SVec::new() })?;
            let has_dark = match pe {
                Effect::CheckProvidedEnergy { energy_map, .. } => {
                    energy_map.iter().any(|em| em.provides.contains(&ct::ANY) || em.provides.contains(&ct::DARK))
                }
                _ => false,
            };
            if !has_dark {
                bail!("CANNOT_USE_POWER");
            }
            let mut slots = SVec::new();
            slots.push(SlotType::Active as u8);
            slots.push(SlotType::Bench as u8);
            let mut f = CardFrame::at(1);
            f.a[0] = p as i32;
            let id = g.player_id(p);
            g.prompt(
                id,
                "MOVE_DAMAGE",
                PromptKind::RemoveDamage { player_type: PlayerType::Any, slots, max_allowed, o: o_opts, same_target: true },
                Cont::Card { card: me, frame: f },
            );
        }
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let transfers = match results.first() {
        Some(Res::DamageTransfers(t)) => *t,
        _ => return Ok(()),
    };
    let mut total = 0;
    for (from, to) in transfers.iter().copied() {
        let source = get_target(&g.st, p, from)?;
        let target = get_target(&g.st, p, to)?;
        g.st.players[p].marker.add(adrena(), me, crate::markers::SourceType::None, crate::markers::TargetScope::None);
        ability_used(g, p, me);
        let src_damage = g.st.slot(source.p as usize, source.s).damage;
        let damage_to_move = (30 - total).min(10.min(src_damage));
        if damage_to_move > 0 {
            let (_, prevented) = g.run_fx(Effect::MoveDamageCounters { p: p as u8 })?;
            if prevented {
                continue;
            }
            g.st.players[source.p as usize].slots[source.s as usize].damage -= damage_to_move;
            g.run_fx(Effect::PlaceDamageCounters { p: p as u8, target, damage: damage_to_move, source: me })?;
            total += damage_to_move;
        }
        if total >= 30 {
            break;
        }
    }
    Ok(())
}
