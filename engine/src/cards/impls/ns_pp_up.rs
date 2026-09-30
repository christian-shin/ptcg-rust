//! N's PP Up (JTG): attach 1 Basic Energy from your discard pile to 1 of
//! your Benched N's Pokémon.
//!
//! Twinleaf: a Bench slot counts as "N's" if any card in its stack has the
//! tag; `blockedTo` lists every in-play Pokémon (Active included) whose top
//! card lacks the tag. The card is discarded by the prompt callback.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "NsPPUp", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    let pl = &g.st.players[p];
    let has_energy = pl.discard.iter().any(|c| {
        let d = g.st.cdef(c);
        d.is_energy() && d.energy_type == EnergyType::Basic as u8
    });
    if !has_energy {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    let has_ns = pl.bench.iter().any(|b| pl.slots[*b as usize].cards.iter().any(|c| g.st.cdef(c).has_tag(tag::NS)));
    if !has_ns {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    let mut o = AttachOpts::new(g.st.players[p].discard.len() as u8);
    for (_, c, t) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter() {
        if !g.st.cdef(*c).has_tag(tag::NS) {
            o.blocked_to.push(*t);
        }
    }
    g.set_prevent(e, true);
    o.allow_cancel = false;
    o.min = 1;
    o.max = 1;
    let mut slots = SVec::new();
    slots.push(SlotType::Bench as u8);
    let filter = Filter { super_type: Some(SuperType::Energy as u8), energy_type: Some(EnergyType::Basic as u8), ..Filter::none() };
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    let id = g.player_id(p);
    g.prompt(
        id,
        "ATTACH_ENERGY_TO_BENCH",
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
    let ts = match results.first().copied().unwrap_or(Res::Null) {
        Res::Attach(ts) => ts,
        _ => return Ok(()),
    };
    if ts.is_empty() {
        return Ok(());
    }
    for (to, c) in ts.iter() {
        let target = get_target(&g.st, p, *to)?;
        move_cards(g, ListRef::Discard(p as u8), target.list(), &[*c], me)?;
    }
    move_cards(g, ListRef::Supporter(p as u8), ListRef::Discard(p as u8), &[me], me)
}
