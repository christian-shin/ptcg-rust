//! Azumarill ex (MC): Bubble Gathering — as often as you like during your
//! turn, move an Energy from 1 of your other Pokémon to this Pokémon.
//! Energy Balloon — 60+; 40 more for each [P] Energy card attached to this
//! Pokémon.
//!
//! Twinleaf: the MoveEnergyPrompt (cancellable, 0..1) callback throws
//! INVALID_TARGET unless every transfer targets the slot holding this card;
//! ABILITY_USED runs even for an empty answer. Fixed in phase 4b: the prompt
//! now has `blockedFrom` = this Pokémon's slot and `blockedTo` = every other
//! slot (random answers used to reach the INVALID_TARGET throw). Energy
//! Balloon (also fixed: it compared numeric `CardType` enums to 'P', so it
//! never counted anything) runs CheckProvidedEnergyEffect on the Active and
//! adds 40 for every energy map entry that provides [P].
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Azumarillex", mask: mask(&[k::POWER, k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_power_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Power { p, .. } => p as usize,
            _ => return Ok(()),
        };
        // Phase 4b R7E (ruling 12): an Ability can't be used for no effect: no Energy on the other Pokémon.
        let energy_on_others = for_each_pokemon(g, p, PlayerType::BottomPlayer)
            .iter()
            .any(|(s, c, _)| *c != me && g.st.slot(p, *s).cards.iter().any(|x| g.st.cdef(x).is_energy()));
        if !energy_on_others {
            bail!("CANNOT_USE_POWER");
        }
        let mut slots = SVec::new();
        slots.push(SlotType::Active as u8);
        slots.push(SlotType::Bench as u8);
        let filter = Filter::super_type(SuperType::Energy);
        let mut o = MoveOpts { allow_cancel: true, min: 0, max: Some(1), ..Default::default() };
        for (_, c, t) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
            if c == me {
                o.blocked_from.push(t);
            } else {
                o.blocked_to.push(t);
            }
        }
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
        let mut psychic = 0;
        let a = g.st.players[p].active;
        let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p: p as u8, source: SlotRef::new(p, a), energy_map: SVec::new() })?;
        if let Effect::CheckProvidedEnergy { energy_map, .. } = pe {
            for m in energy_map.iter() {
                if m.provides.iter().any(|t| *t == ct::PSYCHIC || *t == ct::ANY) {
                    psychic += 1;
                }
            }
        }
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage += 40 * psychic;
        }
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
