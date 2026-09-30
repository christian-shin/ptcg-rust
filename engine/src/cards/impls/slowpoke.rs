//! Slowpoke (SCR): Dangle Tail — put a Pokémon from your discard pile into
//! your hand.
//!
//! Twinleaf quirk kept: the chosen card is copied into a fresh CardList and
//! MOVE_CARDS moves that list to the hand, so the card also stays in the
//! discard pile (duplicated).
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
        if g.st.players[p].discard.is_empty() {
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
    let temp = g.alloc_temp(&selected);
    g.run_fx(Effect::MoveCards {
        source: temp,
        destination: ListRef::Hand(p as u8),
        cards: None,
        count: None,
        to_top: false,
        to_bottom: false,
        skip_cleanup: false,
        source_card: NO_CARD,
    })?;
    Ok(())
}
