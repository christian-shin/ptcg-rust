//! Luxray ex (TWM): Piercing Gaze — 120; look at your opponent's hand and
//! discard 1 card you find there. Volt Strike — 250; discard all Energy from
//! this Pokémon.
//!
//! Twinleaf: nothing on an empty opponent's hand, else a ChooseCardsPrompt
//! (CHOOSE_CARD_TO_DECK, min 0 max 1, no cancel) on it; a pick is
//! MOVE_CARDS'd to their discard, followed by a no-op
//! MOVE_CARDS(player.supporter → player.discard, [this]) (quirk kept).
//! Volt Strike: CheckProvidedEnergyEffect on the Active → DiscardCardsEffect.
use super::galvantulaex::discard_all_active_energy;
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Luxrayex", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let (p, o) = match *g.e(e) {
            Effect::Attack { p, opp, .. } => (p as usize, opp as usize),
            _ => return Ok(()),
        };
        if g.st.players[o].hand.is_empty() {
            return Ok(());
        }
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        choose_cards(g, p, "CHOOSE_CARD_TO_DECK", ListRef::Hand(o as u8), Filter::none(), ChooseCardsOpts::new(0, 1, false), Cont::Card { card: me, frame: f });
        return Ok(());
    }
    if was_attack_used(g, e, 1, me) {
        discard_all_active_energy(g, e)?;
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let o = 1 - p;
    let picked: Option<CardId> = match results.first() {
        Some(Res::Cards(c)) => c.iter().next(),
        _ => None,
    };
    let c = match picked {
        Some(c) => c,
        None => return Ok(()),
    };
    move_cards(g, ListRef::Hand(o as u8), ListRef::Discard(o as u8), &[c], me)?;
    move_cards(g, ListRef::Supporter(p as u8), ListRef::Discard(p as u8), &[me], me)?;
    Ok(())
}
