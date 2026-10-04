//! Salvatore (TEF): search your deck for a Pokémon, except any Pokémon with
//! an Ability, that evolves from 1 of your Pokémon in play and put it on
//! that Pokémon to evolve it (also on the turn it was played), then shuffle.
//!
//! Twinleaf: evolution names come from every non-Basic card in the
//! CardManager whose `evolvesFrom` names one of your Pokémon in play (only
//! used for the "nothing can evolve" throw). Fixed (phase 4b): a deck Pokémon
//! is blocked when its `evolvesFrom` names no Pokémon in play, or when it has
//! an Ability (before, any Ability-less Pokémon was selectable and the second
//! prompt then had no valid target; Ability evolutions were selectable too).
//! The CheckPokemonPowers probe runs for every deck Pokémon. The card moves to the
//! supporter pile and the TrainerEffect is prevented before the deck check.
//! The evolution is a plain MOVE_CARDS deck→slot + clearEffects +
//! pokemonPlayedTurn = turn (no EvolveEffect).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Salvatore", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    if g.st.players[p].supporter_turn > 0 {
        bail!("SUPPORTER_ALREADY_PLAYED");
    }
    move_cards(g, ListRef::Hand(p as u8), ListRef::Supporter(p as u8), &[me], me)?;
    g.set_prevent(e, true);
    if g.st.players[p].deck.is_empty() {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    let mut names: Vec<&'static str> = Vec::new();
    for (_, c, _) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
        let base = g.st.cdef(c).name;
        for (n, from) in crate::gen::evolutions::ALL_EVOLUTIONS.iter() {
            if *from == base && !names.contains(n) {
                names.push(n);
            }
        }
    }
    if names.is_empty() {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    let in_play: Vec<&'static str> = for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().map(|(_, c, _)| g.st.cdef(*c).name).collect();
    let mut blocked = Blocked::default();
    let deck: Vec<CardId> = g.st.players[p].deck.iter().collect();
    for (i, c) in deck.into_iter().enumerate() {
        let d = g.st.cdef(c);
        if !d.is_pokemon() {
            continue;
        }
        let not_evolution = !in_play.contains(&d.evolves_from);
        let mut powers = SVec::new();
        for k in 0..d.powers.len() {
            powers.push(PowerRef { card: c, index: k as u8 });
        }
        let (pe, _) = g.run_fx(Effect::CheckPokemonPowers { p: p as u8, target: c, powers })?;
        let has_ability = match pe {
            Effect::CheckPokemonPowers { powers, .. } => powers.iter().any(|r| power_type_of(g, *r) == PowerType::Ability as u8),
            _ => false,
        };
        if not_evolution || has_ability {
            blocked.push(i as u8);
        }
    }
    let mut opts = ChooseCardsOpts::new(1, 1, true);
    opts.blocked = blocked;
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    choose_cards(g, p, "CHOOSE_CARD_TO_EVOLVE", ListRef::Deck(p as u8), Filter::super_type(SuperType::Pokemon), opts, Cont::Card { card: me, frame: f });
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    let first = results.first().copied().unwrap_or(Res::Null);
    match f.stage {
        1 => {
            let evo = match first.cards().first().copied() {
                Some(c) => c,
                None => {
                    shuffle_deck(g, p);
                    return Ok(());
                }
            };
            let from = g.st.cdef(evo).evolves_from;
            let mut blocked = SVec::new();
            for (_, c, t) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
                if g.st.cdef(c).name != from {
                    blocked.push(t);
                }
            }
            let mut slots = SVec::new();
            slots.push(SlotType::Active as u8);
            slots.push(SlotType::Bench as u8);
            let mut nf = CardFrame::at(2);
            nf.a[0] = p as i32;
            nf.a[1] = evo as i32;
            let id = g.player_id(p);
            g.prompt(
                id,
                "CHOOSE_POKEMON_TO_EVOLVE",
                PromptKind::ChoosePokemon { player_type: PlayerType::BottomPlayer, slots, min: 1, max: 1, allow_cancel: false, blocked },
                Cont::Card { card: me, frame: nf },
            );
            Ok(())
        }
        2 => {
            let evo = f.a[1] as CardId;
            let t = match first.slots().first().copied() {
                Some(t) => t,
                None => {
                    shuffle_deck(g, p);
                    return Ok(());
                }
            };
            if g.st.slot_pokemon(t.p as usize, t.s).is_none() {
                shuffle_deck(g, p);
                return Ok(());
            }
            move_cards(g, ListRef::Deck(p as u8), t.list(), &[evo], me)?;
            let turn = g.st.turn;
            let slot = &mut g.st.players[t.p as usize].slots[t.s as usize];
            crate::engine::game_effect::clear_effects(slot);
            slot.pokemon_played_turn = turn;
            shuffle_deck(g, p);
            Ok(())
        }
        _ => Ok(()),
    }
}
