//! Hilda (SV11W / WHT): search your deck for an Evolution Pokémon and an
//! Energy card, reveal them, put them into your hand, then shuffle.
//!
//! Twinleaf: every non-(Evolution Pokémon / Energy) deck card is blocked,
//! `max = min(evolutions,1) + min(energies,1)` with `maxPokemons` /
//! `maxEnergies`; ShowCards only when something was taken; the final
//! ShuffleDeckPrompt has no trailing wait.
//!
//! Fixed (phase 4b, R2): playable with an empty deck; it now throws
//! CANNOT_PLAY_THIS_CARD before any state change (rulings 779, 851).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Hilda", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    if g.st.players[p].supporter_turn > 0 {
        bail!("SUPPORTER_ALREADY_PLAYED");
    }
    // A search of an empty deck is not possible, so the card cannot be played.
    if g.st.players[p].deck.is_empty() {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    move_cards(g, ListRef::Hand(p as u8), ListRef::Supporter(p as u8), &[me], me)?;
    g.set_prevent(e, true);
    let (mut pokemon, mut energies) = (0u8, 0u8);
    let mut opts = ChooseCardsOpts::new(0, 0, false);
    for (i, c) in g.st.players[p].deck.iter().enumerate() {
        let d = g.st.cdef(c);
        if d.is_pokemon() && d.stage != Stage::Basic as u8 {
            pokemon += 1;
        } else if d.is_energy() {
            energies += 1;
        } else {
            opts.blocked.push(i as u8);
        }
    }
    let max_pokemons = pokemon.min(1);
    let max_energies = energies.min(1);
    opts.max = max_pokemons + max_energies;
    opts.max_pokemons = Some(max_pokemons);
    opts.max_energies = Some(max_energies);
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    choose_cards(g, p, "CHOOSE_CARD_TO_HAND", ListRef::Deck(p as u8), Filter::none(), opts, Cont::Card { card: me, frame: f });
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    reveal_then_shuffle_resume(g, me, f, results)
}

/// Stage 1: move to hand, ShowCards (if any) → stage 2: ShuffleDeck → stage 3: apply.
pub fn reveal_then_shuffle_resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    let first = results.first().copied().unwrap_or(Res::Null);
    match f.stage {
        1 => {
            let cards: Vec<CardId> = first.cards().to_vec();
            move_cards(g, ListRef::Deck(p as u8), ListRef::Hand(p as u8), &cards, me)?;
            if !cards.is_empty() {
                let mut nf = CardFrame::at(2);
                nf.a[0] = p as i32;
                let id = g.player_id(1 - p);
                g.prompt(id, "CARDS_SHOWED_BY_THE_OPPONENT", PromptKind::ShowCards, Cont::Card { card: me, frame: nf });
            } else {
                shuffle(g, me, p);
            }
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

fn shuffle(g: &mut Game, me: CardId, p: usize) {
    let mut f = CardFrame::at(3);
    f.a[0] = p as i32;
    let id = g.player_id(p);
    g.prompt(id, "", PromptKind::ShuffleDeck, Cont::Card { card: me, frame: f });
}
