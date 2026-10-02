//! Spectrier (ASC): Spooky Shot — 30. Phantasmal Barrage — discard all Energy
//! from this Pokémon and place 12 damage counters on 1 of your opponent's
//! Pokémon.
//!
//! Twinleaf: DISCARD_ALL_ENERGY_FROM_POKEMON (one DiscardCardsEffect with the
//! Active's CheckProvidedEnergy map), then a non-cancellable ChoosePokemonPrompt
//! whose callback reduces a PutCountersEffect (120) on the chosen Pokémon.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Spectrier", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 1, me) {
        super::zapdos::discard_all_energy_from_active(g, e)?;
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let mut slots = SVec::new();
        slots.push(SlotType::Active as u8);
        slots.push(SlotType::Bench as u8);
        g.retain_fx(e);
        let mut f = CardFrame::at(1);
        f.e[0] = e;
        let id = g.player_id(p);
        g.prompt(
            id,
            "CHOOSE_POKEMON_TO_DAMAGE",
            PromptKind::ChoosePokemon { player_type: PlayerType::TopPlayer, slots, min: 1, max: 1, allow_cancel: false, blocked: SVec::new() },
            Cont::Card { card: me, frame: f },
        );
    }
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let atk = f.e[0];
    let r = (|| -> R {
        let sel = results.first().copied().unwrap_or(Res::Null);
        let t = match sel.slots().first() {
            Some(t) => *t,
            None => return Ok(()),
        };
        let (p, opp, attack, source) = match *g.e(atk) {
            Effect::Attack { p, opp, attack, source, .. } => (p, opp, attack, source),
            _ => return Ok(()),
        };
        let b = AtkBase { attack_effect: atk, player: p, opponent: opp, attack, source, target: t };
        g.run_fx(Effect::PutCounters { b, damage: 120 })?;
        Ok(())
    })();
    g.release_fx(atk);
    r
}
