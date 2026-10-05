//! Larry's Skill (PRE 115): discard your hand and search your deck for a
//! Pokémon, a Supporter card, and a Basic Energy card, reveal them, and put
//! them into your hand. Then, shuffle your deck.
//!
//! Twinleaf order kept: the card moves itself to the supporter pile and
//! cancels the default discard; the whole rest of the hand is discarded
//! before the search. `blocked` holds the deck positions (unsorted deck
//! order) of cards that are neither Pokémon, Supporters nor Basic Energy;
//! the prompt caps each kind at 1. The final ShuffleDeckPrompt has no
//! trailing WaitPrompt.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "LarrysSkill", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    if g.st.players[p].supporter_turn > 0 {
        bail!("SUPPORTER_ALREADY_PLAYED");
    }
    // Fixed (phase 4b, rulings 779/851; 1037: the hand discard is a cost): a search of an empty deck is not possible, so the card can't be played.
    if g.st.players[p].deck.is_empty() {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    move_cards(g, ListRef::Hand(p as u8), ListRef::Supporter(p as u8), &[me], me)?;
    g.set_prevent(e, true);
    g.run_fx(Effect::MoveCards {
        source: ListRef::Hand(p as u8),
        destination: ListRef::Discard(p as u8),
        cards: None,
        count: None,
        to_top: false,
        to_bottom: false,
        skip_cleanup: false,
        source_card: me,
    })?;
    let (mut pokemons, mut supporters, mut energies) = (0u8, 0u8, 0u8);
    let mut blocked = Blocked::default();
    for (i, c) in g.st.players[p].deck.iter().enumerate() {
        let d = g.st.cdef(c);
        if d.is_pokemon() {
            pokemons += 1;
        } else if d.is_trainer() && d.trainer_type == TrainerType::Supporter as u8 {
            supporters += 1;
        } else if d.is_energy() && d.energy_type == EnergyType::Basic as u8 {
            energies += 1;
        } else {
            blocked.push(i as u8);
        }
    }
    let max_pokemons = pokemons.min(1);
    let max_supporters = supporters.min(1);
    let max_energies = energies.min(1);
    let mut opts = ChooseCardsOpts::new(0, max_pokemons + max_supporters + max_energies, false);
    opts.blocked = blocked;
    opts.max_pokemons = Some(max_pokemons);
    opts.max_supporters = Some(max_supporters);
    opts.max_energies = Some(max_energies);
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    choose_cards(g, p, "CHOOSE_CARDS", ListRef::Deck(p as u8), Filter::none(), opts, Cont::Card { card: me, frame: f });
    Ok(())
}

fn shuffle(g: &mut Game, me: CardId, p: usize) {
    let mut f = CardFrame::at(3);
    f.a[0] = p as i32;
    let id = g.player_id(p);
    g.prompt(id, "", PromptKind::ShuffleDeck, Cont::Card { card: me, frame: f });
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    let first = results.first().copied().unwrap_or(Res::Null);
    match f.stage {
        1 => {
            let cards: Vec<CardId> = first.cards().to_vec();
            move_cards(g, ListRef::Deck(p as u8), ListRef::Hand(p as u8), &cards, me)?;
            if !cards.is_empty() {
                let oid = g.player_id(1 - p);
                g.prompt(oid, "CARDS_SHOWED_BY_THE_OPPONENT", PromptKind::ShowCards, Cont::Card { card: me, frame: CardFrame { stage: 2, ..f } });
                return Ok(());
            }
            shuffle(g, me, p);
            Ok(())
        }
        2 => {
            shuffle(g, me, p);
            Ok(())
        }
        3 => {
            if let Res::Order(o) = first {
                crate::game::apply_order(&mut g.st.players[p].deck, o.as_slice());
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
