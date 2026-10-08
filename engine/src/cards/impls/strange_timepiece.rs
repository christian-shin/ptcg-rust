//! Strange Timepiece (MEG): devolve 1 of your evolved [P] Pokémon by putting
//! any number of Evolution cards on it into your hand.
//!
//! Twinleaf: the second prompt is a ChooseCardsPrompt over the whole slot
//! (Pokémon filter) with the Basic's index blocked (phase 4b fix: the Basic
//! used to be selectable and threw INVALID_PROMPT_RESULT; the throw stays as
//! a defensive check); choosing a card devolves `pokemons.length - index` times via
//! DEVOLVE_POKEMON (which sets `pokemonPlayedTurn` but not
//! `cannotEvolveNextTurn`).
use crate::cards::prelude::*;
use crate::spec::devolve_pokemon;

pub static IMPL: CardImpl = CardImpl { class: "StrangeTimepiece", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

/// `PokemonCardList.isEvolved()`.
fn is_evolved(g: &Game, p: usize, s: SlotId) -> bool {
    let pokemons = g.st.slot_pokemons(p, s);
    if pokemons.len() == 1 {
        return false;
    }
    if let Some(top) = g.st.slot_pokemon(p, s) {
        let st = g.st.cdef(top).stage;
        if st == Stage::Legend as u8 || st == Stage::Vunion as u8 {
            return false;
        }
        if st == Stage::LvX as u8 && pokemons.len() == 2 {
            return false;
        }
    }
    true
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    let mut can_devolve = false;
    let mut blocked: TargetList = SVec::new();
    for (s, _, t) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
        let target = SlotRef::new(p, s);
        let types = crate::engine::game_effect::pokemon_types(g, target);
        let (ce, _) = g.run_fx(Effect::CheckPokemonType { target, card_types: types })?;
        let psychic = matches!(ce, Effect::CheckPokemonType { card_types, .. } if card_types.contains(&ct::PSYCHIC));
        if is_evolved(g, p, s) && psychic {
            can_devolve = true;
        } else {
            blocked.push(t);
        }
    }
    if !can_devolve {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    let mut slots = SVec::new();
    slots.push(SlotType::Active as u8);
    slots.push(SlotType::Bench as u8);
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    let id = g.player_id(p);
    g.prompt(
        id,
        "CHOOSE_POKEMON",
        PromptKind::ChoosePokemon { player_type: PlayerType::BottomPlayer, slots, min: 1, max: 1, allow_cancel: false, blocked },
        Cont::Card { card: me, frame: f },
    );
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    match f.stage {
        1 => {
            let t = match results.first().and_then(|r| r.slots().first().copied()) {
                Some(t) => t,
                None => return Ok(()),
            };
            if g.st.slot_pokemons(t.p as usize, t.s).is_empty() {
                return Ok(());
            }
            let mut nf = CardFrame::at(2);
            nf.a[0] = p as i32;
            nf.a[1] = t.p as i32;
            nf.a[2] = t.s as i32;
            // The Basic can't be put into the hand (phase 4b fix): blocked by index.
            let mut opts = ChooseCardsOpts::new(1, 1, false);
            let basic = g.st.slot_pokemons(t.p as usize, t.s).get(0).copied();
            for (i, c) in g.st.slot(t.p as usize, t.s).cards.iter().enumerate() {
                if Some(c) == basic {
                    opts.blocked.push(i as u8);
                }
            }
            choose_cards(
                g,
                p,
                "CHOOSE_POKEMON_TO_PICK_UP",
                ListRef::Slot(t.p, t.s),
                Filter::super_type(SuperType::Pokemon),
                opts,
                Cont::Card { card: me, frame: nf },
            );
            Ok(())
        }
        2 => {
            let t = SlotRef::new(f.a[1] as usize, f.a[2] as SlotId);
            let sel = match results.first().and_then(|r| r.cards().first().copied()) {
                Some(c) => c,
                None => return Ok(()),
            };
            let pokemons = g.st.slot_pokemons(t.p as usize, t.s);
            let idx = pokemons.iter().position(|c| *c == sel);
            if idx == Some(0) {
                bail!("INVALID_PROMPT_RESULT");
            }
            if let Some(i) = idx {
                for _ in 0..(pokemons.len() - i) {
                    devolve_pokemon(g, t, ListRef::Hand(p as u8))?;
                }
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
