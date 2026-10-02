//! Powerglass (SFA, tool; class PowerHourglass): at the end of your turn, if
//! the Pokémon this card is attached to is in the Active Spot, you may attach
//! a Basic Energy from your discard pile to that Pokémon.
//!
//! Twinleaf: on EndTurnEffect when the player's Active holds this tool: the
//! tool probe (a bare ToolEffect), then a prompt only if the discard pile
//! holds a Basic Energy (0 or 1, no cancel, targets the Active slot only).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "PowerHourglass", mask: mask(&[k::END_TURN]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match *g.e(e) {
        Effect::EndTurn { p } => p as usize,
        _ => return Ok(()),
    };
    let a = g.st.players[p].active;
    if !g.st.slot(p, a).tools.contains(me) {
        return Ok(());
    }
    if g.run_fx(Effect::Tool { p: p as u8, card: me }).is_err() {
        return Ok(());
    }
    let has = g.st.players[p].discard.iter().any(|c| {
        let d = g.st.cdef(c);
        d.is_energy() && d.energy_type == EnergyType::Basic as u8
    });
    if !has {
        return Ok(());
    }
    let mut o = AttachOpts::new(1);
    o.allow_cancel = false;
    o.min = 0;
    o.max = 1;
    let mut slots = SVec::new();
    slots.push(SlotType::Active as u8);
    let mut filter = Filter::super_type(SuperType::Energy);
    filter.energy_type = Some(EnergyType::Basic as u8);
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    let id = g.player_id(p);
    g.prompt(
        id,
        "ATTACH_ENERGY_TO_ACTIVE",
        PromptKind::AttachEnergy { cards: ListRef::Discard(p as u8), player_type: PlayerType::BottomPlayer, slots, filter, o },
        Cont::Card { card: me, frame: f },
    );
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
        move_cards(g, ListRef::Discard(p as u8), target.list(), &[c], me)?;
    }
    Ok(())
}
