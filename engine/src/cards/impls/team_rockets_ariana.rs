//! Team Rocket's Ariana (DRI): draw until you have 5 cards in hand, or 8 if
//! all of your Pokémon in play are Team Rocket's Pokémon.
//!
//! Twinleaf: sets `rocketSupporter` (not when used as the effect of an attack);
//! the draw is one MOVE_CARDS per card. Fixed (phase 4b, R7C): a card that would
//! draw nothing can't be played (rulings 851, 959).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "TeamRocketsAriana", mask: mask(&[k::TRAINER, k::END_TURN]), reduce, resume: None, coin: None, can_play: None };

/// Hand size to draw up to: 8 when all of your Pokémon in play are Team Rocket's Pokémon, else 5.
fn target_hand_size(g: &Game, p: usize) -> usize {
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
        // Twinleaf checks `card instanceof PokemonCard`: a Fossil in play (a Trainer card) is skipped.
        if s != a && g.st.cdef(c).is_pokemon() {
            has_pokemon = true;
            if !g.st.cdef(c).has_tag(tag::TEAM_ROCKET) {
                all_rocket = false;
            }
        }
    }
    if has_pokemon && all_rocket {
        8
    } else {
        5
    }
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Some(p) = trainer_played(g, e, me) {
        if g.st.players[p].supporter_turn > 0 {
            bail!("SUPPORTER_ALREADY_PLAYED");
        }
        // A card that would draw nothing (empty deck, or already N cards in hand without this
        // one) can't be played (rulings 851, 959).
        if g.st.players[p].deck.is_empty() || g.st.players[p].hand.iter().filter(|c| *c != me).count() >= target_hand_size(g, p) {
            bail!("CANNOT_PLAY_THIS_CARD");
        }
        // Using the effect of a Supporter as the effect of an attack is not playing it from the hand.
        if !trainer_via_attack(g, e) {
            g.st.players[p].rocket_supporter = true;
        }
        move_cards(g, ListRef::Hand(p as u8), ListRef::Supporter(p as u8), &[me], me)?;
        g.set_prevent(e, true);
        let target = target_hand_size(g, p);
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
