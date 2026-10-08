//! Dialga (MEG 95): Beam — 30. Chrono Burst — 80+; you may shuffle all
//! Energy attached to this Pokémon into your deck and have this attack do 80
//! more damage.
//!
//! Fixed (ruling 1822): Twinleaf asked nothing when the Active had no Energy, so
//! the 80 more damage was unreachable; the ConfirmPrompt is now always asked and
//! yes adds 80 (with no Energy nothing moves and the deck is not shuffled).
//! Twinleaf: a ConfirmPrompt, then one MOVE_CARDS per mapped card (captured
//! before the prompt; Active to deck), SHUFFLE_DECK and `effect.damage += 80`
//! (after the shuffle prompt is opened, before it resolves).
//! R7A (rulings 1580, 1846): the Energy goes back into the deck, and the deck is shuffled, after the damage (`move_cards_after_damage`, `shuffle_deck_after_damage`).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Dialga", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 1, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let a = g.st.players[p].active;
        let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p: p as u8, source: SlotRef::new(p, a), energy_map: SVec::new() })?;
        let mut cards: SVec<CardId, 64> = SVec::new();
        if let Effect::CheckProvidedEnergy { energy_map, .. } = pe {
            for m in energy_map.iter() {
                cards.push(m.card);
            }
        }
        let temp = g.alloc_temp(cards.as_slice());
        g.retain_fx(e);
        let mut f = CardFrame::at(1);
        f.e[0] = e;
        f.a[0] = p as i32;
        f.l[0] = a;
        f.l[1] = match temp {
            ListRef::Temp(i) => i,
            _ => 0,
        };
        confirmation_prompt(g, p, "WANT_TO_USE_ABILITY", Cont::Card { card: me, frame: f });
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let atk = f.e[0];
    let p = f.a[0] as usize;
    let r = (|| -> R {
        if !results.first().copied().unwrap_or(Res::Null).as_bool() {
            return Ok(());
        }
        let slot = SlotRef::new(p, f.l[0]);
        let cards: Vec<CardId> = g.lst(ListRef::Temp(f.l[1])).to_vec();
        for c in cards.iter().copied() {
            move_cards_after_damage(g, atk, slot.list(), ListRef::Deck(p as u8), &[c], me)?;
        }
        if !cards.is_empty() {
            shuffle_deck_after_damage(g, atk, p);
        }
        if let Effect::Attack { damage, .. } = g.e_mut(atk) {
            *damage += 80;
        }
        Ok(())
    })();
    g.release_fx(atk);
    r
}
