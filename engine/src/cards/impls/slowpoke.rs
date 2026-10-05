//! Slowpoke (SCR): Dangle Tail — put a Pokémon from your discard pile into
//! your hand.
//!
//! Fixed (W1-E): the attack is unusable (CANNOT_USE_ATTACK) unless the discard
//! pile holds a Pokémon (Twinleaf checked for any card, so the min-1 prompt
//! could have no valid answer), and the chosen card is moved from the discard
//! pile itself (Twinleaf moved it from a fresh CardList, so it stayed in the
//! discard pile as well as going to the hand). Phase 4b (R4, Meta-Rulings): the
//! chosen cards are revealed to the opponent before they are moved (cards moving
//! from the discard pile to the hand are revealed).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "Slowpoke@Slowpoke SCR",
    mask: mask(&[k::ATTACK]),
    reduce,
    resume: Some(resume),
    coin: None,
    can_play: None,
};

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        if !g.st.players[p].discard.iter().any(|c| g.st.cdef(c).super_type == SuperType::Pokemon as u8) {
            bail!("CANNOT_USE_ATTACK");
        }
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        choose_cards(
            g,
            p,
            "CHOOSE_CARD_TO_HAND",
            ListRef::Discard(p as u8),
            Filter::super_type(SuperType::Pokemon),
            ChooseCardsOpts::new(1, 1, false),
            Cont::Card { card: me, frame: f },
        );
    }
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let selected: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
    show_cards_to_player(g, 1 - p, selected.len());
    g.run_fx(Effect::MoveCards {
        source: ListRef::Discard(p as u8),
        destination: ListRef::Hand(p as u8),
        cards: Some(List::from_slice(&selected)),
        count: None,
        to_top: false,
        to_bottom: false,
        skip_cleanup: false,
        source_card: NO_CARD,
    })?;
    Ok(())
}
