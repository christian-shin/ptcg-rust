//! Team Rocket's Porygon-Z (DRI): Reconstitute — discard 2 cards from your
//! hand; once during your turn, draw a card. R Command — 20 damage for each
//! Team Rocket Supporter in your discard pile.
//!
//! Twinleaf: POWER_ALREADY_USED with the marker, CANNOT_USE_POWER with
//! fewer than 2 cards in hand or an empty deck, then a non-cancellable
//! ChooseCardsPrompt (exactly 2) on the hand; the callback marks, adds the
//! ability-used board effect, discards the pair and draws 1.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "TeamRocketsPorygonZ",
    mask: mask(&[k::END_TURN, k::POWER, k::ATTACK]),
    reduce,
    resume: Some(resume),
    coin: None,
    can_play: None,
};

fn reconstitute() -> crate::markers::MarkerName {
    crate::marker!("RECONSTITUTE_MARKER")
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::EndTurn { p } = *g.e(e) {
        let m = &mut g.st.players[p as usize].marker;
        if m.has_from(reconstitute(), me) {
            m.remove_from(reconstitute(), me);
        }
    }
    if was_power_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Power { p, .. } => p as usize,
            _ => return Ok(()),
        };
        if g.st.players[p].marker.has_from(reconstitute(), me) {
            bail!("POWER_ALREADY_USED");
        }
        if g.st.players[p].hand.len() < 2 {
            bail!("CANNOT_USE_POWER");
        }
        if g.st.players[p].deck.is_empty() {
            bail!("CANNOT_USE_POWER");
        }
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        choose_cards(g, p, "CHOOSE_CARD_TO_DISCARD", ListRef::Hand(p as u8), Filter::none(), ChooseCardsOpts::new(2, 2, false), Cont::Card { card: me, frame: f });
        return Ok(());
    }
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let n = g.st.players[p]
            .discard
            .iter()
            .filter(|c| {
                let d = g.st.cdef(*c);
                d.is_trainer() && d.name.contains("Team Rocket") && d.trainer_type == TrainerType::Supporter as u8
            })
            .count() as i32;
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage = 20 * n;
        }
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let cards = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
    if cards.is_empty() {
        return Ok(());
    }
    g.st.players[p].marker.add(reconstitute(), me, crate::markers::SourceType::None, crate::markers::TargetScope::None);
    ability_used(g, p, me);
    move_cards(g, ListRef::Hand(p as u8), ListRef::Discard(p as u8), &cards, me)?;
    draw_cards(g, p, 1)
}
