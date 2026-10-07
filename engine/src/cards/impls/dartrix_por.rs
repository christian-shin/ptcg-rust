//! Dartrix (POR / M3): Leafage — 10. Feather Shot — discard all Energy from
//! this Pokémon, and this attack does 90 damage to 1 of your opponent's
//! Pokémon.
//!
//! Twinleaf: DISCARD_ALL_ENERGY_FROM_POKEMON (CheckProvidedEnergyEffect on
//! the player's Active, then a DiscardCardsEffect of the map's cards on this
//! card's slot), then a non-cancellable ChoosePokemonPrompt and
//! DAMAGE_OPPONENT_POKEMON(90) on the choice.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Dartrix@POR", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !was_attack_used(g, e, 1, me) {
        return Ok(());
    }
    let (p, opp, attack, source) = match *g.e(e) {
        Effect::Attack { p, opp, attack, source, .. } => (p, opp, attack, source),
        _ => return Ok(()),
    };
    let pu = p as usize;
    let o = opp as usize;
    let (mp, ms) = match g.st.find_pokemon_slot(me) {
        Some(x) => x,
        None => bail!("INVALID_TARGET"),
    };
    let active = SlotRef::new(pu, g.st.players[pu].active);
    let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p, source: active, energy_map: SVec::new() })?;
    let mut cards: SVec<CardId, 64> = SVec::new();
    if let Effect::CheckProvidedEnergy { energy_map, .. } = pe {
        for em in energy_map.iter() {
            cards.push(em.card);
        }
    }
    let target = SlotRef::new(mp, ms);
    g.run_fx(Effect::DiscardCards { b: AtkBase { attack_effect: e, player: p, opponent: opp, attack, source, target }, cards })?;

    let pl = &g.st.players[o];
    let has = !pl.slots[pl.active as usize].cards.is_empty() || pl.bench.iter().any(|b| !pl.slots[*b as usize].cards.is_empty());
    if !has {
        return Ok(());
    }
    let mut slots = SVec::new();
    slots.push(SlotType::Active as u8);
    slots.push(SlotType::Bench as u8);
    g.retain_fx(e);
    let mut f = CardFrame::at(1);
    f.e[0] = e;
    let id = g.player_id(pu);
    g.prompt(
        id,
        "CHOOSE_POKEMON_TO_DAMAGE",
        PromptKind::ChoosePokemon { player_type: PlayerType::TopPlayer, slots, min: 1, max: 1, allow_cancel: false, blocked: SVec::new() },
        Cont::Card { card: me, frame: f },
    );
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let atk = f.e[0];
    let first = results.first().copied().unwrap_or(Res::Null);
    let sel: Vec<SlotRef> = first.slots().to_vec();
    let r = damage_opponent_pokemon(g, atk, 90, &sel);
    g.release_fx(atk);
    r
}
