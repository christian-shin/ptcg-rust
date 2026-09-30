//! Energy Switch (SVI): move a Basic Energy from 1 of your Pokémon to
//! another of your Pokémon.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "EnergySwitch", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    let pl = &g.st.players[p];
    let mut count = 0;
    let mut has_basic = false;
    for s in pl.all_slots().iter() {
        if g.st.slot_pokemon(p, *s).is_none() {
            continue;
        }
        count += 1;
        if g.st.slot(p, *s).cards.iter().any(|c| {
            let d = g.st.cdef(c);
            d.is_energy() && d.energy_type == EnergyType::Basic as u8
        }) {
            has_basic = true;
        }
    }
    if !has_basic || count <= 1 {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    g.set_prevent(e, true);
    let mut slots = SVec::new();
    slots.push(SlotType::Active as u8);
    slots.push(SlotType::Bench as u8);
    let mut filter = Filter::super_type(SuperType::Energy);
    filter.energy_type = Some(EnergyType::Basic as u8);
    let o = MoveOpts { allow_cancel: false, min: 1, max: Some(1), ..Default::default() };
    let id = g.player_id(p);
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    g.prompt(id, "MOVE_ENERGY_CARDS", PromptKind::MoveEnergy { player_type: PlayerType::BottomPlayer, slots, filter, o }, Cont::Card { card: me, frame: f });
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    let transfers = match results.first() {
        Some(Res::Transfers(t)) => *t,
        _ => return Ok(()),
    };
    for (from, to, card) in transfers.iter() {
        let src = get_target(&g.st, p, *from)?;
        let dst = get_target(&g.st, p, *to)?;
        move_cards(g, src.list(), dst.list(), &[*card], me)?;
    }
    Ok(())
}
