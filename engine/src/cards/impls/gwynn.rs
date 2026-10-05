//! Gwynn (M5 / PBL): discard up to 2 Pokémon that don't have a Rule Box from
//! your hand, and draw 3 cards for each card discarded.
//!
//! Twinleaf: needs 1 selectable card (else CANNOT_PLAY_THIS_CARD, after the
//! card moved to the supporter zone; fixed in phase 4b, R4: it needed 2 though
//! the text says up to 2); the prompt is `min: 1, max: 2`.
//!
//! R7C: as the effect of an attack (Mr. Mime's Look-Alike Show) the prompt is min 0
//! (rulings 1844, 1853); played from the hand it is min 1.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Gwynn", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    if g.st.players[p].supporter_turn > 0 {
        bail!("SUPPORTER_ALREADY_PLAYED");
    }
    move_cards(g, ListRef::Hand(p as u8), ListRef::Supporter(p as u8), &[me], me)?;
    g.set_prevent(e, true);
    let mut opts = ChooseCardsOpts::new(if trainer_via_attack(g, e) { 0 } else { 1 }, 2, false);
    let hand: Vec<CardId> = g.st.players[p].hand.iter().collect();
    let mut blocked = 0usize;
    for (i, c) in hand.iter().enumerate() {
        let d = g.st.cdef(*c);
        if !d.is_pokemon() || d.has_rule_box() {
            opts.blocked.push(i as u8);
            blocked += 1;
        }
    }
    if hand.len() - blocked < 1 {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    choose_cards(g, p, "CHOOSE_CARD_TO_DISCARD", ListRef::Hand(p as u8), Filter::none(), opts, Cont::Card { card: me, frame: f });
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let cards: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
    move_cards(g, ListRef::Hand(p as u8), ListRef::Discard(p as u8), &cards, me)?;
    draw_cards(g, p, cards.len() * 3)
}
