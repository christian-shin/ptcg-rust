//! Scream Tail (TEF 77): Supportive Singing — heal 100 damage from 1 of your
//! Benched Ancient Pokémon. Hyper Voice — 40.
//!
//! Twinleaf: nothing without a Benched Ancient Pokémon; else a mandatory
//! ChoosePokemonPrompt on the Bench with every non-Ancient (or empty) Bench
//! index blocked, and a HealEffect per chosen target.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "ScreamTailTEFPool", mask: mask(&[k::AFTER_ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !after_attack_used(g, e, 0, me) {
        return Ok(());
    }
    let e = real_attack(g, e);
    let p = match *g.e(e) {
        Effect::Attack { p, .. } => p as usize,
        _ => return Ok(()),
    };
    let mut blocked: TargetList = SVec::new();
    let mut has_ancient = false;
    let bench: Vec<SlotId> = g.st.players[p].bench.iter().copied().collect();
    for (i, b) in bench.iter().enumerate() {
        match g.st.slot_pokemon(p, *b) {
            Some(c) if g.st.cdef(c).has_tag(tag::ANCIENT) => has_ancient = true,
            _ => blocked.push(CardTarget::new(PlayerType::BottomPlayer, SlotType::Bench, i as u8)),
        }
    }
    if !has_ancient {
        return Ok(());
    }
    let mut slots = SVec::new();
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
    let targets: Vec<SlotRef> = results.first().map(|r| r.slots().to_vec()).unwrap_or_default();
    for t in targets {
        g.run_fx(Effect::Heal { p: p as u8, target: t, damage: 100 })?;
    }
    Ok(())
}
