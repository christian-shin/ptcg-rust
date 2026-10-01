//! Dragonair (M2a / ASC 151): Evolutionary Guidance — once during your turn,
//! if this Pokémon has any Energy attached, search your deck for an Evolution
//! Pokémon (non-Basic Pokémon; every other card is blocked by deck index),
//! reveal it, put it into your hand and shuffle. Tail Snap — 60.
//!
//! Twinleaf order: ability-blocked probe, in-play lookup, energy check,
//! marker check; then SEARCH_YOUR_DECK_FOR_POKEMON_AND_PUT_INTO_HAND (throws
//! on an empty deck before the marker is set), marker, board effect.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "Dragonair",
    mask: mask(&[k::PLAY_POKEMON, k::END_TURN, k::POWER]),
    reduce,
    resume: None,
    coin: None,
    can_play: None,
};

fn marker() -> crate::markers::MarkerName {
    crate::marker!("EVOLUTION_GUIDANCE_MARKER")
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::PlayPokemon { p, card, .. } = *g.e(e) {
        if card == me {
            g.st.players[p as usize].marker.remove_from(marker(), me);
        }
    }

    if let Effect::EndTurn { p } = *g.e(e) {
        let m = &mut g.st.players[p as usize].marker;
        if m.has_from(marker(), me) {
            m.remove_from(marker(), me);
        }
    }

    if was_power_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Power { p, .. } => p as usize,
            _ => return Ok(()),
        };
        if is_ability_blocked(g, p, me, None) {
            return Ok(());
        }
        let mut found = None;
        for (s, c, _) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
            if c == me {
                found = Some(s);
            }
        }
        let slot = match found {
            Some(s) => s,
            None => bail!("CANNOT_USE_POWER"),
        };
        if !g.st.slot(p, slot).cards.iter().any(|c| g.st.cdef(c).is_energy()) {
            bail!("CANNOT_USE_POWER");
        }
        if g.st.players[p].marker.has_from(marker(), me) {
            bail!("POWER_ALREADY_USED");
        }
        let mut opts = ChooseCardsOpts::new(0, 1, true);
        for (i, c) in g.st.players[p].deck.iter().enumerate() {
            let d = g.st.cdef(c);
            let block = if d.is_pokemon() { d.stage == Stage::Basic as u8 } else { true };
            if block {
                opts.blocked.push(i as u8);
            }
        }
        search_deck_for_pokemon_to_hand(g, p, Filter::none(), opts)?;
        g.st.players[p].marker.add(marker(), me, crate::markers::SourceType::None, crate::markers::TargetScope::None);
        ability_used(g, p, me);
    }
    Ok(())
}
