//! Blaziken ex (JTG): Seething Spirit — once during your turn, attach a Basic
//! Energy card from your discard pile to 1 of your Pokémon. Burning Assault
//! — 200; during your next turn, this Pokémon can't attack.
//!
//! Twinleaf: the marker (OVERFLOWING_SPIRIT_MARKER) and the ABILITY_USED
//! board effect are set in the prompt callback; the marker is removed on any
//! EndTurnEffect for the ending player.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Blazikenex", mask: mask(&[k::END_TURN, k::POWER, k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn spirit() -> crate::markers::MarkerName {
    crate::marker!("OVERFLOWING_SPIRIT_MARKER")
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::EndTurn { p } = *g.e(e) {
        let m = &mut g.st.players[p as usize].marker;
        if m.has_from(spirit(), me) {
            m.remove_from(spirit(), me);
        }
    }

    if was_power_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Power { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let has_energy = g.st.players[p].discard.iter().any(|c| {
            let d = g.st.cdef(c);
            d.is_energy() && d.energy_type == EnergyType::Basic as u8
        });
        if !has_energy {
            bail!("CANNOT_USE_POWER");
        }
        if g.st.players[p].marker.has_from(spirit(), me) {
            bail!("CANNOT_USE_POWER");
        }
        let mut o = AttachOpts::new(g.st.players[p].discard.len() as u8);
        o.allow_cancel = false;
        o.min = 1;
        o.max = 1;
        let filter = Filter { super_type: Some(SuperType::Energy as u8), energy_type: Some(EnergyType::Basic as u8), ..Filter::none() };
        let mut slots = SVec::new();
        slots.push(SlotType::Bench as u8);
        slots.push(SlotType::Active as u8);
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        let id = g.player_id(p);
        g.prompt(
            id,
            "ATTACH_ENERGY_TO_ACTIVE",
            PromptKind::AttachEnergy { cards: ListRef::Discard(p as u8), player_type: PlayerType::BottomPlayer, slots, filter, o },
            Cont::Card { card: me, frame: f },
        );
    }

    if was_attack_used(g, e, 0, me) {
        if let Effect::Attack { p, .. } = *g.e(e) {
            let p = p as usize;
            let a = g.st.players[p].active;
            g.st.players[p].slots[a as usize].cannot_attack_next_turn_pending = true;
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
    g.st.players[p].marker.add(spirit(), me, crate::markers::SourceType::None, crate::markers::TargetScope::None);
    ability_used(g, p, me);
    for (to, c) in transfers.iter().copied() {
        let target = get_target(&g.st, p, to)?;
        move_cards(g, ListRef::Discard(p as u8), target.list(), &[c], me)?;
    }
    Ok(())
}
