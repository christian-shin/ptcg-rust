//! Alakazam (TWM): Strange Hacking — your opponent's Active Pokémon is now
//! Confused; you may move any number of damage counters from your opponent's
//! Pokémon to their other Pokémon in any way you like. Psychic — 10+, 50 more
//! for each Energy attached to your opponent's Active Pokémon.
//!
//! Twinleaf (twilight-masquerade file): Strange Hacking reduces an
//! AddSpecialConditionsEffect (Confused), builds maxAllowedDamage from a
//! CheckHpEffect per opponent Pokémon and opens a MoveDamagePrompt (opponent's
//! Active + Bench, cancellable, defaults otherwise); each transfer moves 10
//! damage directly if the source has at least 10. Psychic counts
//! `provides` of the opponent's CheckProvidedEnergyEffect (their Active).
//!
//! Fixed (phase 4b, R7F-6; ruling 1665): the transfers bypassed Mist Energy
//! and Repelling Veil. Each one now probes the source and the destination with
//! a PutCountersEffect of 0 counters: a protected source keeps its counter, a protected
//! destination loses the counter that leaves the source.
//!
//! The prompt answers one transfer per damage counter, 20-30 of them for the
//! bot (any number is valid): `Res::DamageTransfers` is run-length encoded and
//! `damage_transfers` expands it (Y2-3; it held 16 transfers before).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Alakazam@TWM", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let (p, o) = match *g.e(e) {
            Effect::Attack { p, opp, .. } => (p as usize, opp as usize),
            _ => return Ok(()),
        };
        add_special_conditions_to_opponent_active(g, e, &[SpecialCondition::Confused])?;
        let mut max_allowed: SVec<(CardTarget, i32), 16> = SVec::new();
        for (s, _, t) in for_each_pokemon(g, o, PlayerType::TopPlayer).iter().copied() {
            let hp = crate::engine::check::check_hp(g, o, s)?;
            max_allowed.push((t, hp));
        }
        let mut slots = SVec::new();
        slots.push(SlotType::Active as u8);
        slots.push(SlotType::Bench as u8);
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        f.e[0] = e;
        g.retain_fx(e);
        let id = g.player_id(p);
        g.prompt(
            id,
            "MOVE_DAMAGE",
            PromptKind::MoveDamage {
                player_type: PlayerType::TopPlayer,
                slots,
                max_allowed,
                o: MoveOpts::default(),
                single_source: false,
                single_destination: false,
                damage_multiple: 10,
            },
            Cont::Card { card: me, frame: f },
        );
        return Ok(());
    }

    if was_attack_used(g, e, 1, me) {
        let o = match *g.e(e) {
            Effect::Attack { opp, .. } => opp as usize,
            _ => return Ok(()),
        };
        let active = SlotRef::new(o, g.st.players[o].active);
        let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p: o as u8, source: active, energy_map: SVec::new() })?;
        let n: i32 = match pe {
            Effect::CheckProvidedEnergy { energy_map, .. } => energy_map.iter().map(|m| m.provides.len() as i32).sum(),
            _ => 0,
        };
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage += n * 50;
        }
    }
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let atk = f.e[0];
    let r = (|| -> R {
        let p = f.a[0] as usize;
        let transfers = match results.first() {
            Some(Res::DamageTransfers(t)) => *t,
            _ => return Ok(()),
        };
        let (opp, attack, asource) = match *g.e(atk) {
            Effect::Attack { opp, attack, source, .. } => (opp, attack, source),
            _ => return Ok(()),
        };
        for (from, to) in damage_transfers(transfers.as_slice()) {
            let source = get_target(&g.st, p, from)?;
            let target = get_target(&g.st, p, to)?;
            if g.st.slot(source.p as usize, source.s).damage >= 10 {
                let b = AtkBase { attack_effect: atk, player: p as u8, opponent: opp, attack, source: asource, target: source };
                let (_, from_prevented) = g.run_fx(Effect::PutCounters { b, damage: 0 })?;
                if from_prevented {
                    continue;
                }
                g.st.players[source.p as usize].slots[source.s as usize].damage -= 10;
                let b = AtkBase { attack_effect: atk, player: p as u8, opponent: opp, attack, source: asource, target };
                let (_, to_prevented) = g.run_fx(Effect::PutCounters { b, damage: 0 })?;
                if !to_prevented {
                    g.st.players[target.p as usize].slots[target.s as usize].damage += 10;
                }
            }
        }
        Ok(())
    })();
    g.release_fx(atk);
    r
}
