//! Duskull (SFA): Come and Get You — put up to 3 Duskull from your discard
//! pile onto your Bench. Mumble — 30.
//!
//! Twinleaf quirks kept: the attack throws CANNOT_USE_POWER unless a Duskull
//! is in the *hand* (not the discard pile); the prompt is `min: 1` with
//! `max = min(empty bench slots, 3)` even when that is 0 or no Duskull is in
//! the discard pile.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Duskull@Duskull SFA|Duskull PRE", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !was_attack_used(g, e, 0, me) {
        return Ok(());
    }
    let (p, source) = match *g.e(e) {
        Effect::Attack { p, source, .. } => (p as usize, source),
        _ => return Ok(()),
    };
    let slots = empty_bench_slots(g, p);
    let max = slots.len().min(3) as u8;
    let has_duskull = g.st.players[p].hand.iter().any(|c| {
        let d = g.st.cdef(c);
        d.is_pokemon() && d.name == "Duskull"
    });
    if !has_duskull {
        bail!("CANNOT_USE_POWER");
    }
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    f.a[1] = g.st.slot_pokemon(source.p as usize, source.s).map(|c| c as i32).unwrap_or(-1);
    f.a[2] = slots.len() as i32;
    let mut packed = 0i32;
    for (i, s) in slots.iter().take(3).enumerate() {
        packed |= (*s as i32) << (8 * i);
    }
    f.a[3] = packed;
    let filter = Filter { super_type: Some(SuperType::Pokemon as u8), name: Some("Duskull"), ..Filter::none() };
    choose_cards(g, p, "CHOOSE_CARD_TO_PUT_ONTO_BENCH", ListRef::Discard(p as u8), filter, ChooseCardsOpts::new(1, max, false), Cont::Card { card: me, frame: f });
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let source_card = if f.a[1] < 0 { NO_CARD } else { f.a[1] as CardId };
    let n_slots = f.a[2] as usize;
    let mut cards: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
    cards.truncate(n_slots);
    for (i, c) in cards.iter().enumerate() {
        let s = ((f.a[3] >> (8 * i)) & 0xff) as SlotId;
        move_cards(g, ListRef::Discard(p as u8), ListRef::Slot(p as u8, s), &[*c], source_card)?;
        g.st.players[p].slots[s as usize].pokemon_played_turn = g.st.turn;
    }
    Ok(())
}
