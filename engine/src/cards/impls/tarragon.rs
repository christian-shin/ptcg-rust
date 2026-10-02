//! Tarragon (POR, supporter): put up to 4 in any combination of [F] Pokémon
//! and Basic [F] Energy cards from your discard pile into your hand.
//!
//! Twinleaf: DiscardToHandEffect is probed first (a prevented effect just
//! leaves the card to be discarded); the supporter check follows, then the
//! card moves to the supporter pile. Energy counts only when it is a Basic
//! card named 'Fighting Energy'; Pokémon by `pokemonHasCardType`. The
//! prompt allows 0 to 4 (maxPokemons/maxEnergies = min(count, 4)); the cards
//! are moved to the hand first and shown to the opponent afterwards.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Tarragon", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    let (_, prevented) = g.run_fx(Effect::DiscardToHand { p: p as u8, card: me })?;
    if prevented {
        return Ok(());
    }
    if g.st.players[p].supporter_turn > 0 {
        bail!("SUPPORTER_ALREADY_PLAYED");
    }
    move_cards(g, ListRef::Hand(p as u8), ListRef::Supporter(p as u8), &[me], me)?;
    g.set_prevent(e, true);
    let mut pokemons = 0u8;
    let mut energies = 0u8;
    let mut opts = ChooseCardsOpts::new(0, 4, false);
    let discard: Vec<CardId> = g.st.players[p].discard.iter().collect();
    for (i, c) in discard.iter().enumerate() {
        let d = g.st.cdef(*c);
        if d.is_energy() && d.energy_type == EnergyType::Basic as u8 && d.name == "Fighting Energy" {
            energies += 1;
        } else if d.is_pokemon() && d.card_type.contains(&ct::FIGHTING) {
            pokemons += 1;
        } else {
            opts.blocked.push(i as u8);
        }
    }
    opts.max_pokemons = Some(pokemons.min(4));
    opts.max_energies = Some(energies.min(4));
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    choose_cards(g, p, "CHOOSE_CARD_TO_HAND", ListRef::Discard(p as u8), Filter::none(), opts, Cont::Card { card: me, frame: f });
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let cards: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
    move_cards(g, ListRef::Discard(p as u8), ListRef::Hand(p as u8), &cards, me)?;
    show_cards_to_player(g, 1 - p, cards.len());
    Ok(())
}
