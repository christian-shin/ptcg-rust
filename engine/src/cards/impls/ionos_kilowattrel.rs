//! Iono's Kilowattrel (JTG / ASC): Flashing Draw - you must discard a Basic
//! [L] Energy from this Pokémon to use this Ability; once during your turn,
//! draw cards until you have 6 cards in your hand. Mach Bolt - 70.
//!
//! Twinleaf quirks kept: the player marker RUMBLING_ENGINE_MARKER (source
//! this card) is cleared on this card's PlayPokemonEffect and on *every*
//! EndTurnEffect (whoever's turn it is, for that effect's player). The
//! Ability throws at 6+ cards in hand, when already used, or without a
//! "Lightning Energy" named basic Energy on the Pokémon. With exactly one
//! such Energy it is discarded without a prompt; otherwise a ChooseCards
//! prompt over the slot (min 0, max 1, cancellable; an empty answer does
//! nothing). Draw is a plain `deck.moveTo(hand, n)`.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "IonosKilowattrel",
    mask: mask(&[k::PLAY_POKEMON, k::END_TURN, k::POWER]),
    reduce,
    resume: Some(resume),
    coin: None,
    can_play: None,
};

fn rumbling() -> crate::markers::MarkerName {
    crate::marker!("RUMBLING_ENGINE_MARKER")
}

fn lightning_filter() -> Filter {
    Filter { super_type: Some(SuperType::Energy as u8), energy_type: Some(EnergyType::Basic as u8), name: Some("Lightning Energy"), ..Filter::none() }
}

fn finish(g: &mut Game, me: CardId, p: usize) {
    let n = 6usize.saturating_sub(g.st.players[p].hand.len());
    g.move_to(ListRef::Deck(p as u8), ListRef::Hand(p as u8), Some(n));
    g.st.players[p].marker.add(rumbling(), me, crate::markers::SourceType::None, crate::markers::TargetScope::None);
    ability_used(g, p, me);
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::PlayPokemon { p, card, .. } = *g.e(e) {
        if card == me {
            g.st.players[p as usize].marker.remove_from(rumbling(), me);
        }
    }
    if let Effect::EndTurn { p } = *g.e(e) {
        g.st.players[p as usize].marker.remove_from(rumbling(), me);
    }
    if was_power_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Power { p, .. } => p as usize,
            _ => return Ok(()),
        };
        if g.st.players[p].hand.len() >= 6 {
            bail!("CANNOT_USE_POWER");
        }
        if g.st.players[p].marker.has_from(rumbling(), me) {
            bail!("POWER_ALREADY_USED");
        }
        let (sp, ss) = match g.st.find_pokemon_slot(me) {
            Some(x) => x,
            None => bail!("INVALID_GAME_STATE"),
        };
        let filter = lightning_filter();
        let energies: Vec<CardId> = g.st.slot(sp, ss).cards.iter().filter(|c| filter.matches(g.st.cdef(*c))).collect();
        if energies.is_empty() {
            bail!("CANNOT_USE_POWER");
        }
        if energies.len() == 1 {
            move_cards(g, ListRef::Slot(sp as u8, ss), ListRef::Discard(p as u8), &[energies[0]], me)?;
            finish(g, me, p);
            return Ok(());
        }
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        f.a[1] = sp as i32;
        f.a[2] = ss as i32;
        choose_cards(g, p, "CHOOSE_CARD_TO_DISCARD", ListRef::Slot(sp as u8, ss), filter, ChooseCardsOpts::new(0, 1, true), Cont::Card { card: me, frame: f });
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let src = ListRef::Slot(f.a[1] as u8, f.a[2] as SlotId);
    let cards: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
    if cards.is_empty() {
        return Ok(());
    }
    for c in cards {
        move_cards(g, src, ListRef::Discard(p as u8), &[c], me)?;
    }
    finish(g, me, p);
    Ok(())
}
