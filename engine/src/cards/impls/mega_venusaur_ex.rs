//! Mega Venusaur ex (M1L 3): Solar Trans — as many times as you like during
//! your turn, move a Basic [G] Energy from one of your Pokémon to another.
//! Jungle Dump — 240; heal 30 damage from this Pokémon.
//!
//! Twinleaf: the Ability throws CANNOT_USE_POWER only when none of your
//! Pokémon has an Energy card attached; otherwise a MoveEnergyPrompt
//! (min 0, no max, not cancellable) for Basic energy named 'Grass Energy',
//! each transfer a MOVE_CARDS with this card as source. Jungle Dump reduces
//! a HealEffect (not HealTargetEffect) on `player.active`.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "MegaVenusaurEx", mask: mask(&[k::POWER, k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_power_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Power { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let with_energy = for_each_pokemon(g, p, PlayerType::BottomPlayer)
            .iter()
            .filter(|(s, _, _)| g.st.slot(p, *s).cards.iter().any(|c| g.st.cdef(c).is_energy()))
            .count();
        if with_energy == 0 {
            bail!("CANNOT_USE_POWER");
        }
        let mut slots = SVec::new();
        slots.push(SlotType::Active as u8);
        slots.push(SlotType::Bench as u8);
        let filter = Filter {
            super_type: Some(SuperType::Energy as u8),
            energy_type: Some(EnergyType::Basic as u8),
            name: Some("Grass Energy"),
            ..Filter::none()
        };
        let o = MoveOpts { allow_cancel: false, min: 0, max: None, ..Default::default() };
        let id = g.player_id(p);
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        g.prompt(id, "MOVE_ENERGY_CARDS", PromptKind::MoveEnergy { player_type: PlayerType::BottomPlayer, slots, filter, o }, Cont::Card { card: me, frame: f });
        return Ok(());
    }
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let a = g.st.players[p].active;
        g.run_fx(Effect::Heal { p: p as u8, target: SlotRef::new(p, a), damage: 30 })?;
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
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
