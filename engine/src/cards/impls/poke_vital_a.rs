//! Poké Vital A (SFA, ACE SPEC): heal 150 damage from 1 of your Pokémon.
//!
//! Twinleaf: undamaged Pokémon are blocked; no cancel; HealEffect 150. The "can't be put into your deck or hand" text is not implemented.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "PokeVitalA", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    let mut has = false;
    let mut blocked: TargetList = SVec::new();
    for (s, _, t) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
        if g.st.slot(p, s).damage == 0 {
            blocked.push(t);
        } else {
            has = true;
        }
    }
    if !has {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    g.set_prevent(e, true);
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
    let targets: Vec<SlotRef> = results.first().map(|r| r.slots().to_vec()).unwrap_or_default();
    for t in targets {
        g.run_fx(Effect::Heal { p: p as u8, target: t, damage: 150 })?;
    }
    Ok(())
}
