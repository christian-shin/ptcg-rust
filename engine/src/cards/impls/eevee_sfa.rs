//! Eevee (SFA): Colorful Catch — search your deck for up to 3 Basic Energy
//! cards of different types, reveal them, and put them into your hand, then
//! shuffle. Headbutt — 20.
//!
//! Twinleaf has several `Eevee` classes; this port is bound to SFA.
//! Twinleaf quirks kept: an empty deck makes the attack fail
//! (CANNOT_PLAY_THIS_CARD); the prompt's max is the number of distinct
//! `provides[0]` among the deck's Basic Energy (capped at 3); the cards are
//! not revealed, and the generator never resumes after the prompt callback
//! (it doesn't call `next()`), so the deck is never shuffled.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Eevee@SFA", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !was_attack_used(g, e, 0, me) {
        return Ok(());
    }
    let (p, source) = match *g.e(e) {
        Effect::Attack { p, source, .. } => (p as usize, source),
        _ => return Ok(()),
    };
    if g.st.players[p].deck.is_empty() {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    let mut types: Vec<CardType> = Vec::new();
    for c in g.st.players[p].deck.iter() {
        let d = g.st.cdef(c);
        if d.is_energy() && d.energy_type == EnergyType::Basic as u8 {
            if let Some(t) = d.provides.first() {
                if !types.contains(t) {
                    types.push(*t);
                }
            }
        }
    }
    let max = types.len().min(3) as u8;
    let mut opts = ChooseCardsOpts::new(0, max, false);
    opts.different_types = true;
    let mut filter = Filter::super_type(SuperType::Energy);
    filter.energy_type = Some(EnergyType::Basic as u8);
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    f.l[0] = source.p;
    f.l[1] = source.s;
    choose_cards(g, p, "CHOOSE_CARD_TO_HAND", ListRef::Deck(p as u8), filter, opts, Cont::Card { card: me, frame: f });
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let cards: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
    if cards.len() > 1 && g.st.cdef(cards[0]).name == g.st.cdef(cards[1]).name {
        bail!("CAN_ONLY_SELECT_TWO_DIFFERENT_ENERGY_TYPES");
    }
    let sc = g.st.slot_pokemon(f.l[0] as usize, f.l[1]).unwrap_or(NO_CARD);
    move_cards(g, ListRef::Deck(p as u8), ListRef::Hand(p as u8), &cards, sc)?;
    Ok(())
}
