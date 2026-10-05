//! Chansey (TWM): Lucky Attachment — attach a Basic Energy card from your
//! hand to 1 of your Pokémon. Boundless Power — 80; during your next turn,
//! this Pokémon can't attack.
//!
//! Lucky Attachment does nothing with no Basic Energy in hand (fixed in
//! R1-13: it used to throw CANNOT_USE_ATTACK, so the attack was not offered).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Chansey", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let has = g.st.players[p].hand.iter().any(|c| {
            let d = g.st.cdef(c);
            d.is_energy() && d.energy_type == EnergyType::Basic as u8
        });
        if !has {
            return Ok(());
        }
        let mut o = AttachOpts::new(g.st.players[p].hand.len() as u8);
        o.allow_cancel = false;
        o.min = 1;
        o.max = 1;
        let mut slots = SVec::new();
        slots.push(SlotType::Active as u8);
        slots.push(SlotType::Bench as u8);
        let mut filter = Filter::super_type(SuperType::Energy);
        filter.energy_type = Some(EnergyType::Basic as u8);
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        let id = g.player_id(p);
        g.prompt(
            id,
            "ATTACH_ENERGY_TO_BENCH",
            PromptKind::AttachEnergy { cards: ListRef::Hand(p as u8), player_type: PlayerType::BottomPlayer, slots, filter, o },
            Cont::Card { card: me, frame: f },
        );
        return Ok(());
    }
    if was_attack_used(g, e, 1, me) {
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
    let transfers: SVec<(CardTarget, CardId), 16> = match results.first() {
        Some(Res::Attach(t)) => *t,
        _ => SVec::new(),
    };
    for (to, c) in transfers.iter().copied() {
        let target = get_target(&g.st, p, to)?;
        move_cards(g, ListRef::Hand(p as u8), target.list(), &[c], me)?;
    }
    Ok(())
}
