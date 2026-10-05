//! Rotom (SSP): Crushing Pulse — your opponent reveals their hand; discard
//! all Item and Pokémon Tool cards you find there. Energy Short — 20× the
//! Energy attached to your opponent's Active Pokémon.
//!
//! Twinleaf: a ShowCardsPrompt whose callback moves every Item / Pokémon Tool
//! of `opponent.hand` into an escrow list one MOVE_CARDS at a time (fixed in
//! R1-11: it walked the hand itself with `forEach` while MOVE_CARDS spliced
//! it, so the card after each moved one was skipped; it now walks a filtered
//! copy), then MOVE_CARDS escrow → discard. Energy Short counts every
//! `provides` entry of CheckProvidedEnergyEffect(opponent) and sets
//! `damage = n * 20`.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Rotom@SSP", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let (p, o) = match *g.e(e) {
            Effect::Attack { p, opp, .. } => (p as usize, opp as usize),
            _ => return Ok(()),
        };
        let mut f = CardFrame::at(1);
        f.a[0] = o as i32;
        let id = g.player_id(p);
        g.prompt(id, "CARDS_SHOWED_BY_THE_OPPONENT", PromptKind::ShowCards, Cont::Card { card: me, frame: f });
    }
    if was_attack_used(g, e, 1, me) {
        let o = match *g.e(e) {
            Effect::Attack { opp, .. } => opp as usize,
            _ => return Ok(()),
        };
        let active = SlotRef::new(o, g.st.players[o].active);
        let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p: o as u8, source: active, energy_map: SVec::new() })?;
        let n = match pe {
            Effect::CheckProvidedEnergy { energy_map, .. } => energy_map.iter().map(|m| m.provides.len() as i32).sum::<i32>(),
            _ => 0,
        };
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage = n * 20;
        }
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, _results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let o = f.a[0] as usize;
    let hand = ListRef::Hand(o as u8);
    let escrow = g.alloc_temp(&[]);
    // Iterate over a copy of the matching cards (the hand is spliced as they move).
    let matching: Vec<CardId> = g
        .lst(hand)
        .iter()
        .copied()
        .filter(|c| {
            let d = g.st.cdef(*c);
            d.is_trainer() && (d.trainer_type == TrainerType::Item as u8 || d.trainer_type == TrainerType::Tool as u8)
        })
        .collect();
    for c in matching {
        move_cards(g, hand, escrow, &[c], me)?;
    }
    g.run_fx(Effect::MoveCards {
        source: escrow,
        destination: ListRef::Discard(o as u8),
        cards: None,
        count: None,
        to_top: false,
        to_bottom: false,
        skip_cleanup: false,
        source_card: me,
    })?;
    Ok(())
}
