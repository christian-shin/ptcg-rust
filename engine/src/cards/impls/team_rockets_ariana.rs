//! Team Rocket's Ariana (DRI): draw until you have 5 cards in hand, or 8 if
//! all of your Pokémon in play are Team Rocket's Pokémon.
//!
//! Twinleaf: sets `rocketSupporter`; the draw is one MOVE_CARDS per card.
//! Phase 4b (rulings 851/959): unplayable when the deck is empty or the rest of the hand already
//! has the target number of cards.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "TeamRocketsAriana", mask: mask(&[k::TRAINER, k::END_TURN]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Some(p) = trainer_played(g, e, me) {
        if g.st.players[p].supporter_turn > 0 {
            bail!("SUPPORTER_ALREADY_PLAYED");
        }
        let mut all_rocket = true;
        let mut has_pokemon = false;
        let a = g.st.players[p].active;
        if !g.st.slot(p, a).cards.is_empty() {
            has_pokemon = true;
            match g.st.slot_pokemon(p, a) {
                Some(c) if g.st.cdef(c).has_tag(tag::TEAM_ROCKET) => {}
                _ => all_rocket = false,
            }
        }
        for (s, c, _) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
            if s != a {
                has_pokemon = true;
                if !g.st.cdef(c).has_tag(tag::TEAM_ROCKET) {
                    all_rocket = false;
                }
            }
        }
        let target = if has_pokemon && all_rocket { 8 } else { 5 };
        // Fixed (phase 4b, rulings 851/959): nothing to draw (empty deck, or the hand already has the target number of
        // cards) is obviously no effect. Computed before any state change.
        let others = g.st.players[p].hand.iter().filter(|c| *c != me).count();
        if g.st.players[p].deck.is_empty() || others >= target {
            bail!("CANNOT_PLAY_THIS_CARD");
        }
        g.st.players[p].rocket_supporter = true;
        move_cards(g, ListRef::Hand(p as u8), ListRef::Supporter(p as u8), &[me], me)?;
        g.set_prevent(e, true);
        while g.st.players[p].hand.len() < target {
            if g.st.players[p].deck.is_empty() {
                break;
            }
            move_count_from(g, ListRef::Deck(p as u8), ListRef::Hand(p as u8), 1, me)?;
        }
        return Ok(());
    }
    if let Effect::EndTurn { p } = *g.e(e) {
        if g.st.players[p as usize].rocket_supporter {
            g.st.players[p as usize].rocket_supporter = false;
        }
    }
    Ok(())
}
