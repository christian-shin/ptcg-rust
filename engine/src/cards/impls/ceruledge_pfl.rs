//! Ceruledge (M2 / PFL): Purgatory Slash — 220; discard 4 Basic [R] Energy
//! cards from your hand or this attack does nothing.
//!
//! Twinleaf: with fewer than 4 "Fire Energy" basics in hand the damage is set
//! to 0 at once; otherwise a non-cancellable ChooseCardsPrompt (exactly 4)
//! whose callback discards them (or zeroes the damage if the filtered result
//! is not exactly 4).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Ceruledge@PFL", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn is_fire(g: &Game, c: CardId) -> bool {
    let d = g.st.cdef(c);
    d.is_energy() && d.energy_type == EnergyType::Basic as u8 && d.name == "Fire Energy"
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let n = g.st.players[p].hand.iter().filter(|c| is_fire(g, *c)).count();
        if n < 4 {
            if let Effect::Attack { damage, .. } = g.e_mut(e) {
                *damage = 0;
            }
            return Ok(());
        }
        g.retain_fx(e);
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        f.e[0] = e;
        let filter = Filter { super_type: Some(SuperType::Energy as u8), energy_type: Some(EnergyType::Basic as u8), name: Some("Fire Energy"), ..Filter::none() };
        choose_cards(g, p, "CHOOSE_CARD_TO_DISCARD", ListRef::Hand(p as u8), filter, ChooseCardsOpts::new(4, 4, false), Cont::Card { card: me, frame: f });
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let atk = f.e[0];
    let first = results.first().copied().unwrap_or(Res::Null);
    let r = (|| -> R {
        let cards: Vec<CardId> = first.cards().iter().copied().filter(|c| is_fire(g, *c)).collect();
        if cards.len() == 4 {
            move_cards(g, ListRef::Hand(p as u8), ListRef::Discard(p as u8), &cards, me)?;
        } else if let Effect::Attack { damage, .. } = g.e_mut(atk) {
            *damage = 0;
        }
        Ok(())
    })();
    g.release_fx(atk);
    r
}
