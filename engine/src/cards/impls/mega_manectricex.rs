//! Mega Manectric ex (M1S): Flash Ray — 120; during your opponent's next
//! turn, prevent all damage done to this Pokémon by attacks from Basic
//! Pokémon. Riotous Blasting — 200+; you may discard all Energy from this
//! Pokémon for 130 more damage.
//!
//! Twinleaf: PREVENT_DAMAGE(..., { sourceStage: BASIC }). Riotous Blasting
//! is a CONFIRMATION_PROMPT (WANT_TO_DISCARD_ENERGY) whose yes-callback adds
//! 130, then DISCARD_ALL_ENERGY_FROM_POKEMON(this): INVALID_TARGET unless
//! this card is in a Pokémon slot, CheckProvidedEnergyEffect(player) on the
//! Active, DiscardCardsEffect on this card's slot.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "MegaManectricEx", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let filter = PreventFilter { source_stage: Some(Stage::Basic as u8), ..Default::default() };
        prevent_damage_filtered(g, e, filter)?;
    }
    if was_attack_used(g, e, 1, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        g.retain_fx(e);
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        f.e[0] = e;
        confirmation_prompt(g, p, "WANT_TO_DISCARD_ENERGY", Cont::Card { card: me, frame: f });
    }
    Ok(())
}

/// `DISCARD_ALL_ENERGY_FROM_POKEMON(store, state, effect, card)`.
pub fn discard_all_energy_from_pokemon(g: &mut Game, e: EffId, card: CardId) -> R {
    let (p, opp, attack, source) = match *g.e(e) {
        Effect::Attack { p, opp, attack, source, .. } => (p, opp, attack, source),
        _ => return Ok(()),
    };
    let (mp, ms) = match g.st.find_pokemon_slot(card) {
        Some(x) => x,
        None => bail!("INVALID_TARGET"),
    };
    let pu = p as usize;
    let active = SlotRef::new(pu, g.st.players[pu].active);
    let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p, source: active, energy_map: SVec::new() })?;
    let mut cards: SVec<CardId, 16> = SVec::new();
    if let Effect::CheckProvidedEnergy { energy_map, .. } = pe {
        for em in energy_map.iter() {
            cards.push(em.card);
        }
    }
    let target = SlotRef::new(mp, ms);
    g.run_fx(Effect::DiscardCards { b: AtkBase { attack_effect: e, player: p, opponent: opp, attack, source, target }, cards })?;
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let atk = f.e[0];
    let yes = results.first().map(|r| r.as_bool()).unwrap_or(false);
    let r = (|| -> R {
        if !yes {
            return Ok(());
        }
        if let Effect::Attack { damage, .. } = g.e_mut(atk) {
            *damage += 130;
        }
        discard_all_energy_from_pokemon(g, atk, me)
    })();
    g.release_fx(atk);
    r
}
