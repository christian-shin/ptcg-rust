//! Shuckle (MEG): Fermented Juice — once during your turn, if this Pokémon
//! has any [G] Energy attached, heal 30 damage from 1 of your Pokémon.
//! Rollout — 30.
//!
//! Twinleaf: throws CANNOT_USE_POWER without [G] provided or without a
//! damaged Pokémon, then USE_ABILITY_ONCE_PER_TURN (POWER_ALREADY_USED) and
//! ABILITY_USED before a non-cancellable ChoosePokemonPrompt (undamaged
//! Pokémon blocked) and a HealEffect of 30.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Shuckle@MEG", mask: mask(&[k::POWER, k::END_TURN]), reduce, resume: Some(resume), coin: None, can_play: None };

fn juice() -> crate::markers::MarkerName {
    crate::marker!("FERMENTED_JUICE_MARKER")
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_power_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Power { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let (sp, ss) = match g.st.find_pokemon_slot(me) {
            Some(x) => x,
            None => bail!("TypeError: findCardList"),
        };
        let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p: p as u8, source: SlotRef::new(sp, ss), energy_map: SVec::new() })?;
        let has_grass = match &pe {
            Effect::CheckProvidedEnergy { energy_map, .. } => crate::energy::check_enough_energy(energy_map.as_slice(), &[ct::GRASS]),
            _ => false,
        };
        if !has_grass {
            bail!("CANNOT_USE_POWER");
        }
        let mut blocked = TargetList::new();
        let mut has_target = false;
        for (s, _, t) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter() {
            if g.st.slot(p, *s).damage == 0 {
                blocked.push(*t);
            } else {
                has_target = true;
            }
        }
        if !has_target {
            bail!("CANNOT_USE_POWER");
        }
        use_ability_once_per_turn(g, p, juice(), me)?;
        ability_used(g, p, me);
        let mut slots = SVec::new();
        slots.push(SlotType::Active as u8);
        slots.push(SlotType::Bench as u8);
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        let id = g.player_id(p);
        g.prompt(
            id,
            "CHOOSE_POKEMON_TO_HEAL",
            PromptKind::ChoosePokemon { player_type: PlayerType::BottomPlayer, slots, min: 1, max: 1, allow_cancel: false, blocked },
            Cont::Card { card: me, frame: f },
        );
        return Ok(());
    }
    remove_marker_at_end_of_turn(g, e, juice(), me);
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let first = results.first().copied().unwrap_or(Res::Null);
    if let Some(t) = first.slots().first().copied() {
        g.run_fx(Effect::Heal { p: p as u8, target: t, damage: 30 })?;
    }
    Ok(())
}
