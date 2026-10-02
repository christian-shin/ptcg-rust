//! Azumarill ex (MC): Bubble Gathering — as often as you like during your
//! turn, move an Energy from 1 of your other Pokémon to this Pokémon.
//! Energy Balloon — 60+; 40 more for each [P] Energy card attached to this
//! Pokémon.
//!
//! Twinleaf: the MoveEnergyPrompt (cancellable, 0..1) callback throws
//! INVALID_TARGET unless every transfer targets the slot holding this card;
//! ABILITY_USED runs even for an empty answer. Twinleaf bug replicated:
//! Energy Balloon tests `provides?.includes('P')` / `energyType === 'P'` on
//! numeric `CardType` enums, so it never counts anything and the attack
//! always does its base 60 damage (no handler needed).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Azumarillex", mask: mask(&[k::POWER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_power_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Power { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let mut slots = SVec::new();
        slots.push(SlotType::Active as u8);
        slots.push(SlotType::Bench as u8);
        let filter = Filter::super_type(SuperType::Energy);
        let o = MoveOpts { allow_cancel: true, min: 0, max: Some(1), ..Default::default() };
        let id = g.player_id(p);
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        g.prompt(id, "MOVE_ENERGY_CARDS", PromptKind::MoveEnergy { player_type: PlayerType::BottomPlayer, slots, filter, o }, Cont::Card { card: me, frame: f });
        return Ok(());
    }

    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let transfers = match results.first().copied().unwrap_or(Res::Null) {
        Res::Transfers(t) => t,
        _ => SVec::new(),
    };
    for (from, to, card) in transfers.iter() {
        let dst = get_target(&g.st, p, *to)?;
        if g.st.locate(me) != Some(dst.list()) {
            bail!("INVALID_TARGET");
        }
        let src = get_target(&g.st, p, *from)?;
        move_cards(g, src.list(), dst.list(), &[*card], me)?;
    }
    ability_used(g, p, me);
    Ok(())
}
