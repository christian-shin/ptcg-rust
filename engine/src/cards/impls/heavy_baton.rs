//! Heavy Baton (TEF / PAR, tool): if the Pokémon this card is attached to has
//! a Retreat Cost of exactly 4, is in the Active Spot, and is Knocked Out
//! by damage from an attack from your opponent's Pokémon, move up to 3 Basic
//! Energy cards from that Pokémon to your Benched Pokémon in any way you
//! like.
//!
//! Twinleaf: a KnockOutEffect of a slot holding this tool during the
//! opponent's ATTACK phase, where the slot is the owner's Active Spot and the
//! owner carries DAMAGE_DEALT_MARKER (Knocked Out by damage from an attack;
//! fixed in phase 4b, it used to trigger on any KO), and the current Retreat
//! Cost (CheckRetreatCostEffect) is exactly 4 (phase 4b: it used to be a
//! printed Retreat Cost of 4 or more). The Energy list is a copy of the Basic
//! Energy on the slot, but the transfers move the cards from the owner's
//! discard pile (the core has already discarded the Pokémon); the slot marker
//! is removed when the prompt resolves (cancel allowed, up to 3, any Benched
//! Pokémon in any combination: no sameTarget since phase 4b).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "HeavyBaton", mask: mask(&[k::KNOCK_OUT]), reduce, resume: Some(resume), coin: None, can_play: None };

fn baton() -> crate::markers::MarkerName {
    crate::marker!("HEAVY_BATON_MARKER")
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let (p, t) = match *g.e(e) {
        Effect::KnockOut { p, target, .. } => (p as usize, target),
        _ => return Ok(()),
    };
    if !g.st.slot(t.p as usize, t.s).tools.contains(me) {
        return Ok(());
    }
    let opponent = 1 - p;
    if is_tool_blocked(g, p, me) {
        return Ok(());
    }
    if g.st.phase != GamePhase::Attack || g.st.active_player as usize != opponent {
        return Ok(());
    }
    if g.st.slot(t.p as usize, t.s).marker.has(baton()) {
        return Ok(());
    }
    // Only the Active Spot, and only when Knocked Out by damage from an attack
    if g.st.players[p].active != t.s || !g.st.players[p].marker.has(crate::markers::DAMAGE_DEALT_MARKER) {
        return Ok(());
    }
    if g.st.slot_pokemon(t.p as usize, t.s).is_none() {
        return Ok(());
    }
    // Retreat Cost of exactly 4 (the current one: CheckRetreatCostEffect)
    let cost = crate::engine::retreat::check_retreat_cost_base(g, p);
    let (rc, _) = g.run_fx(Effect::CheckRetreatCost { p: p as u8, cost, no_cost: false })?;
    if !matches!(rc, Effect::CheckRetreatCost { cost, .. } if cost.len() == 4) {
        return Ok(());
    }
    let energy: Vec<CardId> = g
        .st
        .slot(t.p as usize, t.s)
        .cards
        .iter()
        .filter(|c| {
            let d = g.st.cdef(*c);
            d.is_energy() && d.energy_type == EnergyType::Basic as u8
        })
        .collect();
    if energy.is_empty() {
        return Ok(());
    }
    g.st.players[t.p as usize].slots[t.s as usize].marker.add(baton(), me, crate::markers::SourceType::None, crate::markers::TargetScope::None);
    let temp = g.alloc_temp(&energy);
    let mut o = AttachOpts::new(energy.len() as u8);
    o.allow_cancel = true;
    o.min = 0;
    o.max = 3;
    let filter = Filter { super_type: Some(SuperType::Energy as u8), energy_type: Some(EnergyType::Basic as u8), ..Filter::none() };
    let mut slots = SVec::new();
    slots.push(SlotType::Bench as u8);
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    f.a[1] = t.s as i32;
    let id = g.player_id(p);
    g.prompt(
        id,
        "ATTACH_ENERGY_TO_BENCH",
        PromptKind::AttachEnergy { cards: temp, player_type: PlayerType::BottomPlayer, slots, filter, o },
        Cont::Card { card: me, frame: f },
    );
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let s = f.a[1] as SlotId;
    let transfers: SVec<(CardTarget, CardId), 16> = match results.first() {
        Some(Res::Attach(t)) => *t,
        _ => SVec::new(),
    };
    g.st.players[p].slots[s as usize].marker.remove(baton());
    for (to, c) in transfers.iter().copied() {
        let target = get_target(&g.st, p, to)?;
        move_cards(g, ListRef::Discard(p as u8), target.list(), &[c], NO_CARD)?;
    }
    Ok(())
}
