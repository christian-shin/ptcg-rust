//! Lunatone (M1L): Lunar Cycle — once during your turn, if you have Solrock
//! in play, you may discard a Basic [F] Energy card from your hand in order
//! to use this Ability. Draw 3 cards (1 Lunar Cycle per turn). Power Gem — 50.
//!
//! Twinleaf keeps the flag on the player (`usedLunarCycle`, set only after a
//! discard); every copy resets it at the end of its owner's turn. Solrock may
//! be Active or Benched (matched by name). Cancelling the discard prompt
//! uses nothing.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Lunatone", mask: mask(&[k::POWER, k::END_TURN]), reduce, resume: Some(resume), coin: None, can_play: None };

fn fighting_filter() -> Filter {
    Filter {
        super_type: Some(SuperType::Energy as u8),
        energy_type: Some(EnergyType::Basic as u8),
        name: Some("Fighting Energy"),
        ..Filter::none()
    }
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_power_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Power { p, .. } => p as usize,
            _ => return Ok(()),
        };
        if g.st.players[p].used_lunar_cycle {
            bail!("CANNOT_USE_POWER");
        }
        if !for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().any(|(_, c, _)| g.st.cdef(*c).name == "Solrock") {
            bail!("CANNOT_USE_POWER");
        }
        if g.st.players[p].deck.is_empty() {
            bail!("CANNOT_USE_POWER");
        }
        let filter = fighting_filter();
        if !g.st.players[p].hand.iter().any(|c| filter.matches(g.st.cdef(c))) {
            bail!("CANNOT_USE_POWER");
        }
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        choose_cards(g, p, "CHOOSE_CARD_TO_DISCARD", ListRef::Hand(p as u8), filter, ChooseCardsOpts::new(1, 1, true), Cont::Card { card: me, frame: f });
    }
    if let Effect::EndTurn { p } = *g.e(e) {
        if g.st.owner(me) == p as usize {
            g.st.players[p as usize].used_lunar_cycle = false;
        }
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let cards: Vec<CardId> = results.first().copied().unwrap_or(Res::Null).cards().to_vec();
    if cards.is_empty() {
        return Ok(());
    }
    move_cards(g, ListRef::Hand(p as u8), ListRef::Discard(p as u8), &cards, me)?;
    draw_cards(g, p, 3)?;
    g.st.players[p].used_lunar_cycle = true;
    ability_used(g, p, me);
    Ok(())
}
