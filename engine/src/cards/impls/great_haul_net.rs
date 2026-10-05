//! Great Haul Net / Big Catch Net (CRI): choose 1 or both — shuffle up to 3
//! [W] Pokémon and/or up to 3 Basic [W] Energy cards from your discard pile
//! into your deck.
//!
//! Twinleaf: one ChooseCardsPrompt (min 1 since phase 4b, rulings 1778/1853: public zone; max 6, maxPokemons 3,
//! maxBasicEnergies 3) over the discard with the other cards blocked; then
//! the card moves from wherever it is to the discard before a wait-less
//! shuffle.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "GreatHaulNet", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    let mut blocked = Blocked::default();
    let mut n = 0;
    for (i, c) in g.st.players[p].discard.iter().enumerate() {
        let d = g.st.cdef(c);
        let water_mon = d.is_pokemon() && d.card_type.contains(&ct::WATER);
        let water_energy = d.is_energy() && d.energy_type == EnergyType::Basic as u8 && d.provides.contains(&ct::WATER);
        if !water_mon && !water_energy {
            blocked.push(i as u8);
            n += 1;
        }
    }
    if g.st.players[p].discard.len() - n == 0 {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    g.set_prevent(e, true);
    let mut opts = ChooseCardsOpts::new(1, 6, false);
    opts.blocked = blocked;
    opts.max_pokemons = Some(3);
    opts.max_basic_energies = Some(3);
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    choose_cards(g, p, "CHOOSE_CARD_TO_DECK", ListRef::Discard(p as u8), Filter::none(), opts, Cont::Card { card: me, frame: f });
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let cards: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
    if !cards.is_empty() {
        move_cards(g, ListRef::Discard(p as u8), ListRef::Deck(p as u8), &cards, me)?;
    }
    if let Some(l) = g.st.locate(me) {
        move_cards(g, l, ListRef::Discard(p as u8), &[me], me)?;
    }
    let id = g.player_id(p);
    g.prompt(id, "", PromptKind::ShuffleDeck, Cont::ShuffleApplyNoWait { p: p as u8 });
    Ok(())
}
