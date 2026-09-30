//! Night Stretcher (SFA): put a Pokémon or a Basic Energy card from your
//! discard pile into your hand.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "NightlyStretcher", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    let (_, prevented) = g.run_fx(Effect::DiscardToHand { p: p as u8, card: me })?;
    if prevented {
        return Ok(());
    }
    let (mut pokemons, mut energies) = (0u8, 0u8);
    let mut opts = ChooseCardsOpts::new(0, 1, false);
    let discard: Vec<CardId> = g.st.players[p].discard.iter().collect();
    for (i, c) in discard.iter().enumerate() {
        let d = g.st.cdef(*c);
        if d.is_energy() && d.energy_type == EnergyType::Basic as u8 {
            energies += 1;
        } else if d.is_pokemon() {
            pokemons += 1;
        } else {
            opts.blocked.push(i as u8);
        }
    }
    if pokemons == 0 && energies == 0 {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    opts.max_pokemons = Some(pokemons.min(1));
    opts.max_energies = Some(energies.min(1));
    g.set_prevent(e, true);
    move_cards(g, ListRef::Hand(p as u8), ListRef::Supporter(p as u8), &[me], me)?;
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
    if !cards.is_empty() {
        let id = g.player_id(1 - p);
        g.prompt(id, "CARDS_SHOWED_BY_THE_OPPONENT", PromptKind::ShowCards, Cont::Noop);
    }
    Ok(())
}
