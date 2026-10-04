//! Volcanion ex (JTG): Scorching Steam — once during your turn, if this
//! Pokémon is Active, your opponent's Active Pokémon is now Burned. Heat
//! Cyclone — 160; move an Energy from this Pokémon to 1 of your Benched
//! Pokémon.
//!
//! Twinleaf quirks kept: the AddSpecialConditionsPowerEffect is built with
//! the OPPONENT as its player and reduced twice (same effect object, once
//! before and once after the marker is added).
//!
//! Fixed (phase 4b, W4): Heat Cyclone's AttachEnergyPrompt used
//! `PlayerType.TOP_PLAYER`, so the Energy went to the opponent's Benched
//! Pokémon (while the bench check looked at your own); it now uses
//! `BOTTOM_PLAYER`, your own Bench.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Volcanionex", mask: mask(&[k::PLAY_POKEMON, k::POWER, k::ATTACK, k::END_TURN]), reduce, resume: Some(resume), coin: None, can_play: None };

fn steam() -> crate::markers::MarkerName {
    crate::marker!("SCORCHING_STEAM")
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::PlayPokemon { p, card, .. } = *g.e(e) {
        if card == me {
            let m = &mut g.st.players[p as usize].marker;
            if m.has_from(steam(), me) {
                m.remove_from(steam(), me);
            }
        }
    }

    if was_power_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Power { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let o = 1 - p;
        let a = g.st.players[p].active;
        if g.st.slot_pokemon(p, a) != Some(me) {
            bail!("CANNOT_USE_POWER");
        }
        if g.st.players[p].marker.has_from(steam(), me) {
            bail!("CANNOT_USE_POWER");
        }
        let target = SlotRef::new(o, g.st.players[o].active);
        let mut cs = SVec::new();
        cs.push(SpecialCondition::Burned as u8);
        let id = g.new_fx(Effect::AddSpecialConditionsPower {
            p: o as u8,
            source: me,
            target,
            conditions: cs,
            poison_damage: 10,
            burn_damage: 20,
            sleep_flips: 1,
            confusion_damage: 30,
        });
        if let Err(err) = g.reduce_effect(id) {
            g.release_fx(id);
            return Err(err);
        }
        g.st.players[p].marker.add(steam(), me, crate::markers::SourceType::None, crate::markers::TargetScope::None);
        let r = g.reduce_effect(id);
        g.release_fx(id);
        return r;
    }

    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let has_bench = g.st.players[p].bench.iter().any(|s| !g.st.players[p].slots[*s as usize].cards.is_empty());
        if !has_bench {
            return Ok(());
        }
        let a = g.st.players[p].active;
        let mut o = AttachOpts::new(g.st.slot(p, a).cards.len() as u8);
        o.allow_cancel = false;
        o.min = 1;
        o.max = 1;
        let filter = Filter::super_type(SuperType::Energy);
        let mut slots = SVec::new();
        slots.push(SlotType::Bench as u8);
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        let id = g.player_id(p);
        g.prompt(
            id,
            "ATTACH_ENERGY_TO_BENCH",
            PromptKind::AttachEnergy { cards: ListRef::Slot(p as u8, a), player_type: PlayerType::BottomPlayer, slots, filter, o },
            Cont::Card { card: me, frame: f },
        );
        return Ok(());
    }

    if let Effect::EndTurn { p } = *g.e(e) {
        let m = &mut g.st.players[p as usize].marker;
        if m.has_from(steam(), me) {
            m.remove_from(steam(), me);
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
    for (to, c) in transfers.iter().copied() {
        let target = get_target(&g.st, p, to)?;
        let a = g.st.players[p].active;
        move_cards(g, ListRef::Slot(p as u8, a), target.list(), &[c], me)?;
    }
    Ok(())
}
