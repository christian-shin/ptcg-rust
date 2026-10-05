//! Rare Candy (SVI): choose 1 of your Basic Pokémon in play; if you have a
//! Stage 2 card in your hand that evolves from it, put it onto that Pokémon.
//!
//! Twinleaf: the Stage 1 link is looked up in the whole CardManager
//! (`gen::stage1::ALL_STAGE1`); no first-turn check other than the
//! `pokemonPlayedTurn < turn` test via CheckPokemonPlayedTurnEffect; the
//! evolution is a bare EvolveEffect (special conditions are kept). Phase 4b
//! fix: `canUseRareCandy` is false while the player can't evolve (Evolution
//! Jammer, Bronzong TEF), where the EvolveEffect used to throw
//! BLOCKED_BY_EFFECT after the prompts.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "RareCandy", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

/// `isMatchingStage2(stage1, basic, stage2)`.
fn is_matching_stage2(basic: CardId, stage2: CardId, g: &Game) -> bool {
    let b = g.st.cdef(basic).name;
    let s2 = g.st.cdef(stage2).evolves_from;
    crate::gen::stage1::ALL_STAGE1.iter().any(|(n, from)| *n == s2 && *from == b)
}

fn stage2_in_hand(g: &Game, p: usize) -> Vec<CardId> {
    g.st.players[p]
        .hand
        .iter()
        .filter(|c| {
            let d = g.st.cdef(*c);
            d.is_pokemon() && d.stage == Stage::Stage2 as u8
        })
        .collect()
}

fn played_turn(g: &mut Game, p: usize, s: SlotId) -> R<i32> {
    let target = SlotRef::new(p, s);
    let played = g.st.slot(p, s).pokemon_played_turn;
    let (e, _) = g.run_fx(Effect::CheckPokemonPlayedTurn { p: p as u8, target, pokemon_played_turn: played, can_evolve_on_first_turn: false })?;
    Ok(match e {
        Effect::CheckPokemonPlayedTurn { pokemon_played_turn, .. } => pokemon_played_turn,
        _ => played,
    })
}

/// `canUseRareCandy`.
fn can_use(g: &mut Game, p: usize) -> R<bool> {
    let stage2 = stage2_in_hand(g, p);
    // Evolution Jammer (Bronzong TEF): the player can't evolve (phase 4b fix).
    if stage2.is_empty() || g.st.players[p].cannot_evolve_pokemon_cards {
        return Ok(false);
    }
    let mut ok = false;
    for (s, c, _) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
        if g.st.cdef(c).stage != Stage::Basic as u8 {
            continue;
        }
        if !stage2.iter().any(|s2| is_matching_stage2(c, *s2, g)) {
            continue;
        }
        if played_turn(g, p, s)? < g.st.turn {
            ok = true;
        }
    }
    Ok(ok)
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    g.set_prevent(e, true);
    if !can_use(g, p)? {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    let stage2 = stage2_in_hand(g, p);
    let mut blocked: TargetList = SVec::new();
    for (s, c, t) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
        if g.st.cdef(c).stage == Stage::Basic as u8 && stage2.iter().any(|s2| is_matching_stage2(c, *s2, g)) {
            if played_turn(g, p, s)? < g.st.turn {
                continue;
            }
        }
        blocked.push(t);
    }
    move_cards(g, ListRef::Hand(p as u8), ListRef::Supporter(p as u8), &[me], me)?;
    let mut slots = SVec::new();
    slots.push(SlotType::Active as u8);
    slots.push(SlotType::Bench as u8);
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

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    match f.stage {
        1 => {
            let target = match results.first().and_then(|r| r.slots().first().copied()) {
                Some(t) => t,
                None => return Ok(()),
            };
            let base = match g.st.slot_pokemon(target.p as usize, target.s) {
                Some(c) => c,
                None => return Ok(()),
            };
            let mut opts = ChooseCardsOpts::new(1, 1, false);
            let hand: Vec<CardId> = g.st.players[p].hand.iter().collect();
            for (i, c) in hand.iter().enumerate() {
                let d = g.st.cdef(*c);
                if d.is_pokemon() && d.stage == Stage::Stage2 as u8 && !is_matching_stage2(base, *c, g) {
                    opts.blocked.push(i as u8);
                }
            }
            let filter = Filter { super_type: Some(SuperType::Pokemon as u8), stage: Some(Stage::Stage2 as u8), ..Filter::none() };
            let mut nf = CardFrame::at(2);
            nf.a[0] = p as i32;
            nf.a[1] = target.p as i32;
            nf.a[2] = target.s as i32;
            choose_cards(g, p, "CHOOSE_CARD_TO_EVOLVE", ListRef::Hand(p as u8), filter, opts, Cont::Card { card: me, frame: nf });
            Ok(())
        }
        2 => {
            let cards: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
            if let Some(c) = cards.first() {
                let target = SlotRef::new(f.a[1] as usize, f.a[2] as SlotId);
                g.run_fx(Effect::Evolve { p: p as u8, target, card: *c })?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
