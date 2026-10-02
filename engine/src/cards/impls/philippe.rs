//! Philippe (CRI 79 / M4): attach up to 2 Basic [M] Energy cards from your
//! discard pile to 1 of your [M] Pokémon.
//!
//! Twinleaf: no supporter bookkeeping of its own (the core handles the
//! TrainerEffect default). Both probes use CheckPokemonTypeEffect; blocked
//! Pokémon are the non-[M] ones (both Active and Bench, Bench before the
//! prompt). The discard prompt uses the filter `superType: ENERGY` and
//! blocks, by (unsorted) discard position, everything but Basic [M] Energy,
//! with min 0 and max min(2, count); each chosen card is its own MOVE_CARDS.
use crate::cards::prelude::*;
use crate::engine::game_effect::pokemon_types;

pub static IMPL: CardImpl = CardImpl { class: "Philippe", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn is_metal(g: &mut Game, target: SlotRef) -> R<bool> {
    let types = pokemon_types(g, target);
    let (fx, _) = g.run_fx(Effect::CheckPokemonType { target, card_types: types })?;
    Ok(matches!(fx, Effect::CheckPokemonType { card_types, .. } if card_types.contains(&ct::METAL)))
}

fn basic_metal(g: &Game, c: CardId) -> bool {
    let d = g.st.cdef(c);
    d.is_energy() && d.energy_type == EnergyType::Basic as u8 && d.provides.contains(&ct::METAL)
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    let count = g.st.players[p].discard.iter().filter(|c| basic_metal(g, *c)).count();
    if count == 0 {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    let mut blocked = TargetList::new();
    let mut metal = 0;
    for (s, _, t) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
        if !is_metal(g, SlotRef::new(p, s))? {
            blocked.push(t);
        } else {
            metal += 1;
        }
    }
    if metal == 0 {
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
        "CHOOSE_POKEMON_TO_ATTACH_CARDS",
        PromptKind::ChoosePokemon { player_type: PlayerType::BottomPlayer, slots, min: 1, max: 1, allow_cancel: false, blocked },
        Cont::Card { card: me, frame: f },
    );
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    match f.stage {
        1 => {
            let target = match results.first() {
                Some(r) if !r.slots().is_empty() => r.slots()[0],
                _ => return Ok(()),
            };
            if !is_metal(g, target)? {
                return Ok(());
            }
            let count = g.st.players[p].discard.iter().filter(|c| basic_metal(g, *c)).count();
            let mut blocked = Blocked::default();
            for (i, c) in g.st.players[p].discard.iter().enumerate() {
                if !basic_metal(g, c) {
                    blocked.push(i as u8);
                }
            }
            let mut opts = ChooseCardsOpts::new(0, 2.min(count) as u8, false);
            opts.blocked = blocked;
            let mut nf = CardFrame::at(2);
            nf.a[0] = p as i32;
            nf.a[1] = target.s as i32;
            // The prompt's target slot belongs to the player (BOTTOM_PLAYER).
            choose_cards(g, p, "CHOOSE_CARD_TO_ATTACH", ListRef::Discard(p as u8), Filter::super_type(SuperType::Energy), opts, Cont::Card { card: me, frame: nf });
            Ok(())
        }
        2 => {
            let cards: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
            let s = f.a[1] as SlotId;
            for c in cards {
                move_cards(g, ListRef::Discard(p as u8), ListRef::Slot(p as u8, s), &[c], me)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
