//! Grand Tree ("Great Tree SCR", ACE SPEC stadium): once during each
//! player's turn, that player may search their deck for a Stage 1 Pokémon
//! that evolves from 1 of their Basic Pokémon and put it onto that Pokémon to
//! evolve it; if they did, they may search for a Stage 2 that evolves from
//! it and evolve again. Then, that player shuffles their deck.
//!
//! Twinleaf quirks kept: the "can evolve" test uses every non-Basic card the
//! CardManager knows (`gen::evolutions::ALL_EVOLUTIONS`) whose evolvesFrom
//! names an in-play Basic not put into play this turn (CheckPokemonPlayedTurn);
//! there is no first-turn check. Only non-Basics and Basics played this turn
//! are blocked in the Pokémon prompt. The deck prompts (cancellable, deck
//! Pokémon with another evolvesFrom blocked) filter on Stage 1 / Stage 2 and
//! evolvesFrom. The second prompt appears whenever any known card evolves
//! from the chosen Stage 1. Evolving is MOVE_CARDS deck→slot + clearEffects
//! + pokemonPlayedTurn = turn (no EvolveEffect). The deck is shuffled at
//! every exit after the Pokémon prompt.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "GreatTree", mask: mask(&[k::USE_STADIUM]), reduce, resume: Some(resume), coin: None, can_play: None };

fn played_turn(g: &mut Game, p: usize, s: SlotId) -> R<i32> {
    let target = SlotRef::new(p, s);
    let played = g.st.slot(p, s).pokemon_played_turn;
    let (e, _) = g.run_fx(Effect::CheckPokemonPlayedTurn { p: p as u8, target, pokemon_played_turn: played })?;
    Ok(match e {
        Effect::CheckPokemonPlayedTurn { pokemon_played_turn, .. } => pokemon_played_turn,
        _ => played,
    })
}

fn evolves_from_any(name: &str) -> bool {
    crate::gen::evolutions::ALL_EVOLUTIONS.iter().any(|(_, from)| *from == name)
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match *g.e(e) {
        Effect::UseStadium { p, .. } if g.st.stadium_card() == Some(me) => p as usize,
        _ => return Ok(()),
    };
    if g.st.players[p].deck.is_empty() {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    let turn = g.st.turn as i32;
    let mut any = false;
    for (s, c, _) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
        let played = played_turn(g, p, s)?;
        let d = g.st.cdef(c);
        if d.stage != Stage::Basic as u8 || played == turn {
            continue;
        }
        if evolves_from_any(d.name) {
            any = true;
        }
    }
    if !any {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    let mut blocked = SVec::new();
    for (s, c, t) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
        if g.st.cdef(c).stage != Stage::Basic as u8 {
            blocked.push(t);
            continue;
        }
        if played_turn(g, p, s)? == turn {
            blocked.push(t);
        }
    }
    let mut slots = SVec::new();
    slots.push(SlotType::Bench as u8);
    slots.push(SlotType::Active as u8);
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    let id = g.player_id(p);
    g.prompt(
        id,
        "CHOOSE_POKEMON_TO_EVOLVE",
        PromptKind::ChoosePokemon { player_type: PlayerType::BottomPlayer, slots, min: 1, max: 1, allow_cancel: false, blocked },
        Cont::Card { card: me, frame: f },
    );
    Ok(())
}

/// Deck prompt for an evolution of `from` (`stage`), blocking deck Pokémon
/// that evolve from something else.
fn evolution_prompt(g: &mut Game, me: CardId, p: usize, from: &'static str, stage: Stage, frame: CardFrame) {
    let mut blocked = Blocked::default();
    for (i, c) in g.st.players[p].deck.iter().enumerate() {
        let d = g.st.cdef(c);
        if d.is_pokemon() && d.evolves_from != from {
            blocked.push(i as u8);
        }
    }
    let mut opts = ChooseCardsOpts::new(1, 1, true);
    opts.blocked = blocked;
    let filter = Filter { super_type: Some(SuperType::Pokemon as u8), stage: Some(stage as u8), evolves_from: Some(from), ..Filter::none() };
    choose_cards(g, p, "CHOOSE_CARD_TO_EVOLVE", ListRef::Deck(p as u8), filter, opts, Cont::Card { card: me, frame });
}

fn evolve(g: &mut Game, me: CardId, p: usize, t: SlotRef, card: CardId) -> R {
    move_cards(g, ListRef::Deck(p as u8), t.list(), &[card], me)?;
    let turn = g.st.turn;
    let slot = &mut g.st.players[t.p as usize].slots[t.s as usize];
    crate::engine::game_effect::clear_effects(slot);
    slot.pokemon_played_turn = turn;
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    let first = results.first().copied().unwrap_or(Res::Null);
    match f.stage {
        1 => {
            let t = match first.slots().first().copied() {
                Some(t) => t,
                None => {
                    shuffle_deck(g, p);
                    return Ok(());
                }
            };
            let c = match g.st.slot_pokemon(t.p as usize, t.s) {
                Some(c) => c,
                None => return Ok(()),
            };
            let played = played_turn(g, p, t.s)?;
            if g.st.cdef(c).stage != Stage::Basic as u8 || played == g.st.turn as i32 {
                return Ok(());
            }
            let mut nf = CardFrame::at(2);
            nf.a[0] = p as i32;
            nf.a[1] = t.p as i32;
            nf.a[2] = t.s as i32;
            evolution_prompt(g, me, p, g.st.cdef(c).name, Stage::Stage1, nf);
            Ok(())
        }
        2 => {
            let t = SlotRef { p: f.a[1] as u8, s: f.a[2] as SlotId };
            let evo = match first.cards().first().copied() {
                Some(c) => c,
                None => {
                    shuffle_deck(g, p);
                    return Ok(());
                }
            };
            evolve(g, me, p, t, evo)?;
            let name = g.st.cdef(evo).name;
            if evolves_from_any(name) {
                let mut nf = f;
                nf.stage = 3;
                evolution_prompt(g, me, p, name, Stage::Stage2, nf);
                return Ok(());
            }
            shuffle_deck(g, p);
            Ok(())
        }
        3 => {
            let t = SlotRef { p: f.a[1] as u8, s: f.a[2] as SlotId };
            if let Some(c) = first.cards().first().copied() {
                evolve(g, me, p, t, c)?;
            }
            shuffle_deck(g, p);
            Ok(())
        }
        _ => Ok(()),
    }
}
