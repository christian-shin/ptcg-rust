//! Mega Lucario ex (M1L): Aura Jab — 130. Attach up to 3 Basic [F] Energy
//! cards from your discard pile to your Benched Pokémon in any way you like.
//! Mega Brave — 270. During your next turn, this Pokémon can't use Mega Brave.
//!
//! Twinleaf: the AttachEnergyPrompt is created even with nothing to attach;
//! each transfer is a MOVE_CARDS discard → target.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "MegaLucarioex", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let mut slots = SVec::new();
        slots.push(SlotType::Bench as u8);
        let mut o = AttachOpts::new(g.st.players[p].discard.len() as u8);
        o.allow_cancel = false;
        o.min = 0;
        o.max = 3;
        let filter = Filter {
            super_type: Some(SuperType::Energy as u8),
            energy_type: Some(EnergyType::Basic as u8),
            name: Some("Fighting Energy"),
            ..Filter::none()
        };
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        let id = g.player_id(p);
        g.prompt(
            id,
            "ATTACH_ENERGY_TO_BENCH",
            PromptKind::AttachEnergy { cards: ListRef::Discard(p as u8), player_type: PlayerType::BottomPlayer, slots, filter, o },
            Cont::Card { card: me, frame: f },
        );
    }
    if was_attack_used(g, e, 1, me) {
        if let Effect::Attack { p, .. } = *g.e(e) {
            super::riolu_meg::push_pending(g, p as usize, "Mega Brave");
        }
    }
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
    for (to, c) in ts.iter().copied() {
        let target = get_target(&g.st, p, to)?;
        move_cards(g, ListRef::Discard(p as u8), target.list(), &[c], me)?;
    }
    Ok(())
}
