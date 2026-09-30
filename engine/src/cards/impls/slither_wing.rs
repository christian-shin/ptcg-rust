//! Slither Wing (SFA): Iron Buster — 20+; 120 more if your opponent has a
//! Future Pokémon in play. Smashing Wings — 130, discard 2 Energy from this
//! Pokémon.
//!
//! DISCARD_X_ENERGY_FROM_THIS_POKEMON(2): ChooseEnergyPrompt over the
//! Active's CheckProvidedEnergy map for [C][C] (no cancel), then a
//! DiscardCardsEffect aimed at the attacker's Active.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "SlitherWing", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

/// DISCARD_X_ENERGY_FROM_THIS_POKEMON(store, state, effect, amount): resume
/// `stage` with the chosen energy through [`discard_energy_chosen`].
pub fn discard_x_energy_from_this_pokemon(g: &mut Game, me: CardId, e: EffId, amount: usize, stage: u8) -> R {
    let p = match *g.e(e) {
        Effect::Attack { p, .. } => p as usize,
        _ => return Ok(()),
    };
    let a = g.st.players[p].active;
    let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p: p as u8, source: SlotRef::new(p, a), energy_map: SVec::new() })?;
    let energy = match pe {
        Effect::CheckProvidedEnergy { energy_map, .. } => energy_map,
        _ => SVec::new(),
    };
    let mut cost = SVec::new();
    for _ in 0..amount {
        cost.push(ct::COLORLESS);
    }
    g.retain_fx(e);
    let mut f = CardFrame::at(stage);
    f.e[0] = e;
    let id = g.player_id(p);
    g.prompt(id, "CHOOSE_ENERGIES_TO_DISCARD", PromptKind::ChooseEnergy { energy, cost, allow_cancel: false }, Cont::Card { card: me, frame: f });
    Ok(())
}

/// The ChooseEnergyPrompt callback: DiscardCardsEffect on `player.active`.
pub fn discard_energy_chosen(g: &mut Game, f: CardFrame, results: &[Res]) -> R {
    let atk = f.e[0];
    let first = results.first().copied().unwrap_or(Res::Null);
    let r = (|| -> R {
        let mut cards = SVec::new();
        if let Res::Energy(c) = first {
            for x in c.iter() {
                cards.push(*x);
            }
        }
        if let Effect::Attack { p, opp, attack, source, .. } = *g.e(atk) {
            let pp = p as usize;
            let target = SlotRef::new(pp, g.st.players[pp].active);
            g.run_fx(Effect::DiscardCards { b: AtkBase { attack_effect: atk, player: p, opponent: opp, attack, source, target }, cards })?;
        }
        Ok(())
    })();
    g.release_fx(atk);
    r
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let opp = match *g.e(e) {
            Effect::Attack { opp, .. } => opp as usize,
            _ => return Ok(()),
        };
        let future = for_each_pokemon(g, opp, PlayerType::TopPlayer).iter().any(|(_, c, _)| g.st.cdef(*c).has_tag(tag::FUTURE));
        if future {
            if let Effect::Attack { damage, .. } = g.e_mut(e) {
                *damage += 120;
            }
        }
    }
    if was_attack_used(g, e, 1, me) {
        discard_x_energy_from_this_pokemon(g, me, e, 2, 1)?;
    }
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage == 1 {
        return discard_energy_chosen(g, f, results);
    }
    Ok(())
}
