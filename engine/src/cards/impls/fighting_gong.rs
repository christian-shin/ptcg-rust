//! Fighting Gong (M1L, item): search your deck for a Basic [F] Pokémon or a
//! Basic [F] Energy card, reveal it, and put it into your hand; then shuffle.
//!
//! Twinleaf quirks kept: the item is moved to the supporter pile (and never
//! discarded by the card); the energy count is passed as `maxTrainers`, so
//! `max = min(pokemon,1) || min(energy,1)`; ShowCards only when something
//! was taken.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "FightingGong", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    // Fixed (phase 4b, R3): unplayable with an empty deck (nothing to search).
    if g.st.players[p].deck.is_empty() {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    let (mut pokemons, mut trainers) = (0u8, 0u8);
    let mut opts = ChooseCardsOpts::new(0, 0, false);
    for (i, c) in g.st.players[p].deck.iter().enumerate() {
        let d = g.st.cdef(c);
        if d.is_energy() && d.energy_type == EnergyType::Basic as u8 && d.name == "Fighting Energy" {
            trainers += 1;
        } else if d.is_pokemon() && d.card_type.contains(&ct::FIGHTING) && d.stage == Stage::Basic as u8 {
            pokemons += 1;
        } else {
            opts.blocked.push(i as u8);
        }
    }
    g.set_prevent(e, true);
    move_cards(g, ListRef::Hand(p as u8), ListRef::Supporter(p as u8), &[me], me)?;
    let max_pokemons = pokemons.min(1);
    let max_trainers = trainers.min(1);
    opts.max = if max_pokemons != 0 { max_pokemons } else { max_trainers };
    opts.max_pokemons = Some(max_pokemons);
    opts.max_trainers = Some(max_trainers);
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    choose_cards(g, p, "CHOOSE_CARD_TO_HAND", ListRef::Deck(p as u8), Filter::none(), opts, Cont::Card { card: me, frame: f });
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    super::hilda::reveal_then_shuffle_resume(g, me, f, results)
}
