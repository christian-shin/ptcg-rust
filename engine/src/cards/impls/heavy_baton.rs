//! Heavy Baton (TEF / PAR, tool): if the Pokémon this card is attached to has
//! a Retreat Cost of 4 or higher, is in the Active Spot, and is Knocked Out
//! by damage from an attack from your opponent's Pokémon, move up to 3 Basic
//! Energy cards from that Pokémon to your Benched Pokémon in any way you
//! like.
//!
//! Twinleaf quirks kept: any KnockOutEffect of a slot holding this tool
//! during the opponent's ATTACK phase triggers it (Active Spot and damage
//! are not checked, the retreat cost is the printed one); the Energy
//! list is a copy of the Basic Energy on the slot, but the transfers move
//! the cards from the owner's discard pile; the slot marker is removed when
//! the prompt resolves (cancel allowed, same target only).
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
    let pokemon = match g.st.slot_pokemon(t.p as usize, t.s) {
        Some(c) => c,
        None => return Ok(()),
    };
    if g.st.cdef(pokemon).retreat.len() < 4 {
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
    o.same_target = true;
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
