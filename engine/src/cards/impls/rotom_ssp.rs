//! Rotom (SSP): Crushing Pulse — your opponent reveals their hand; discard
//! all Item and Pokémon Tool cards you find there. Energy Short — 20× the
//! Energy attached to your opponent's Active Pokémon.
//!
//! Twinleaf: a ShowCardsPrompt whose callback walks `opponent.hand.cards`
//! with `forEach` while MOVE_CARDS splices matches out into an escrow list —
//! the element after each moved card is skipped (quirk kept) — then
//! MOVE_CARDS escrow → discard. Energy Short counts every `provides` entry
//! of CheckProvidedEnergyEffect(opponent) and sets `damage = n * 20`.
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
    // Array.prototype.forEach over a list spliced during iteration.
    let n0 = g.lst(hand).len();
    let mut i = 0;
    while i < n0 && i < g.lst(hand).len() {
        let c = g.lst(hand)[i];
        let d = g.st.cdef(c);
        if d.is_trainer() && (d.trainer_type == TrainerType::Item as u8 || d.trainer_type == TrainerType::Tool as u8) {
            move_cards(g, hand, escrow, &[c], me)?;
        }
        i += 1;
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
