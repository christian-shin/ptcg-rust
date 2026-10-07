//! Pokémon Center Lady (FLF): heal 60 damage and remove all Special
//! Conditions from 1 of your Pokémon.
//!
//! Twinleaf: no supporter-turn check in the card; the conditions are wiped
//! directly (`specialConditions = []`) after the HealEffect. Fixed in phase 4b
//! (R4, Rulings Compendium 851): throws CANNOT_PLAY_THIS_CARD when no Pokémon
//! has damage or a Special Condition (a card can't be played for no effect).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "PokemonCenterLady", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    let can_heal = for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().any(|(s, _, _)| {
        let sl = g.st.slot(p, *s);
        sl.damage > 0 || !sl.special_conditions.is_empty()
    });
    if !can_heal {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    // A Pokémon with no damage counters and no Special Condition has nothing to heal and can't be chosen
    // (Advanced Rulebook C-06; ruling 43).
    let mut blocked = TargetList::new();
    for (s, _, target) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter() {
        let sl = g.st.slot(p, *s);
        if sl.damage == 0 && sl.special_conditions.is_empty() {
            blocked.push(*target);
        }
    }
    let mut slots = SVec::new();
    slots.push(SlotType::Active as u8);
    slots.push(SlotType::Bench as u8);
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    let id = g.player_id(p);
    g.prompt(
        id,
        "CHOOSE_POKEMON_TO_HEAL",
        PromptKind::ChoosePokemon { player_type: PlayerType::BottomPlayer, slots, min: 1, max: 1, allow_cancel: false, blocked },
        Cont::Card { card: me, frame: f },
    );
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let t = match results.first().and_then(|r| r.slots().first().copied()) {
        Some(t) => t,
        None => return Ok(()),
    };
    g.run_fx(Effect::Heal { p: p as u8, target: t, damage: 60 })?;
    g.st.players[t.p as usize].slots[t.s as usize].special_conditions.clear();
    Ok(())
}
