//! Paldean Tauros (SSP 39): Upthrusting Horns — 30; you may put 2 Energy
//! attached to your opponent's Active Stage 2 Pokémon into their hand.
//! Jet Headbutt — 100.
//!
//! Twinleaf: nothing unless the Defending Pokémon is Stage 2 and provides
//! Energy (CheckProvidedEnergyEffect on the opponent's Active); then a
//! ConfirmPrompt (WANT_TO_USE_ABILITY) and a non-cancellable ChooseEnergyPrompt
//! over that map for min(2, entries) [C]; MOVE_CARDS Active -> hand.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "PaldeanTaurosSSP39Pool", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !was_attack_used(g, e, 0, me) {
        return Ok(());
    }
    let (p, o) = match *g.e(e) {
        Effect::Attack { p, opp, .. } => (p as usize, opp as usize),
        _ => return Ok(()),
    };
    let target = match g.st.active_pokemon(o) {
        Some(c) => c,
        None => return Ok(()),
    };
    if g.st.cdef(target).stage != Stage::Stage2 as u8 {
        return Ok(());
    }
    let a = g.st.players[o].active;
    let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p: o as u8, source: SlotRef::new(o, a), energy_map: SVec::new() })?;
    let energy = match pe {
        Effect::CheckProvidedEnergy { energy_map, .. } => energy_map,
        _ => SVec::new(),
    };
    if energy.is_empty() {
        return Ok(());
    }
    let count = energy.len().min(2);
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    f.a[1] = o as i32;
    f.a[2] = count as i32;
    // The energy map is recomputed unchanged: nothing can alter the Defending
    // Pokémon between the Confirm prompt and its answer.
    confirmation_prompt(g, p, "WANT_TO_USE_ABILITY", Cont::Card { card: me, frame: f });
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    let o = f.a[1] as usize;
    let first = results.first().copied().unwrap_or(Res::Null);
    match f.stage {
        1 => {
            if !first.as_bool() {
                return Ok(());
            }
            let a = g.st.players[o].active;
            let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p: o as u8, source: SlotRef::new(o, a), energy_map: SVec::new() })?;
            let energy = match pe {
                Effect::CheckProvidedEnergy { energy_map, .. } => energy_map,
                _ => SVec::new(),
            };
            let mut cost = SVec::new();
            for _ in 0..f.a[2] {
                cost.push(ct::COLORLESS);
            }
            let mut nf = CardFrame::at(2);
            nf.a[0] = p as i32;
            nf.a[1] = o as i32;
            let id = g.player_id(p);
            g.prompt(id, "CHOOSE_ENERGIES_TO_HAND", PromptKind::ChooseEnergy { energy, cost, allow_cancel: false }, Cont::Card { card: me, frame: nf });
            Ok(())
        }
        2 => {
            let cards: Vec<CardId> = match first {
                Res::Energy(c) => c.as_slice().to_vec(),
                _ => Vec::new(),
            };
            if !cards.is_empty() {
                let a = g.st.players[o].active;
                move_cards(g, ListRef::Slot(o as u8, a), ListRef::Hand(o as u8), &cards, me)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
