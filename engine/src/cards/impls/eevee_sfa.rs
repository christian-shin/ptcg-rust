//! Eevee (SFA): Colorful Catch — search your deck for up to 3 Basic Energy
//! cards of different types, reveal them, and put them into your hand, then
//! shuffle. Headbutt — 20.
//!
//! Twinleaf has several `Eevee` classes; this port is bound to SFA. The
//! prompt's max is the number of distinct `provides[0]` among the deck's
//! Basic Energy (capped at 3). Fixed (R1-5): an empty deck no longer makes
//! the attack fail (nothing happens), the cards are revealed (ShowCards for
//! the opponent) and the deck is shuffled (the generator never resumed after
//! the prompt callback, so neither ever happened); the unreachable
//! CAN_ONLY_SELECT_TWO_DIFFERENT_ENERGY_TYPES throw is gone.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Eevee@SFA", mask: mask(&[k::AFTER_ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !after_attack_used(g, e, 0, me) {
        return Ok(());
    }
    let e = real_attack(g, e);
    let (p, source) = match *g.e(e) {
        Effect::Attack { p, source, .. } => (p as usize, source),
        _ => return Ok(()),
    };
    if g.st.players[p].deck.is_empty() {
        return Ok(());
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

fn shuffle(g: &mut Game, p: usize) {
    let id = g.player_id(p);
    g.prompt(id, "", PromptKind::ShuffleDeck, Cont::ShuffleApplyNoWait { p: p as u8 });
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    match f.stage {
        1 => {
            let cards: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
            let sc = g.st.slot_pokemon(f.l[0] as usize, f.l[1]).unwrap_or(NO_CARD);
            move_cards(g, ListRef::Deck(p as u8), ListRef::Hand(p as u8), &cards, sc)?;
            if !cards.is_empty() {
                let oid = g.player_id(1 - p);
                g.prompt(oid, "CARDS_SHOWED_BY_THE_OPPONENT", PromptKind::ShowCards, Cont::Card { card: me, frame: CardFrame { stage: 2, ..f } });
                return Ok(());
            }
            shuffle(g, p);
            Ok(())
        }
        2 => {
            shuffle(g, p);
            Ok(())
        }
        _ => Ok(()),
    }
}
