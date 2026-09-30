//! Dolliv (DRI): Nutrients — heal 40 damage from 1 of your Pokémon.
//! Tackle — 40.
//!
//! Twinleaf: a non-cancellable ChoosePokemonPrompt over your Active and
//! Bench (undamaged Pokémon selectable, message CHOOSE_POKEMON_TO_DAMAGE),
//! then a HealTargetEffect(effect, 40) on the choice.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Dolliv@DRI", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !was_attack_used(g, e, 0, me) {
        return Ok(());
    }
    let p = match *g.e(e) {
        Effect::Attack { p, .. } => p as usize,
        _ => return Ok(()),
    };
    let mut slots = SVec::new();
    slots.push(SlotType::Active as u8);
    slots.push(SlotType::Bench as u8);
    g.retain_fx(e);
    let mut f = CardFrame::at(1);
    f.e[0] = e;
    let id = g.player_id(p);
    g.prompt(
        id,
        "CHOOSE_POKEMON_TO_DAMAGE",
        PromptKind::ChoosePokemon { player_type: PlayerType::BottomPlayer, slots, min: 1, max: 1, allow_cancel: false, blocked: SVec::new() },
        Cont::Card { card: me, frame: f },
    );
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let atk = f.e[0];
    let first = results.first().copied().unwrap_or(Res::Null);
    let sel = first.slots();
    let r = (|| -> R {
        let target = match sel.first() {
            Some(t) => *t,
            None => return Ok(()),
        };
        let (p, opp, attack, source) = match *g.e(atk) {
            Effect::Attack { p, opp, attack, source, .. } => (p, opp, attack, source),
            _ => return Ok(()),
        };
        g.run_fx(Effect::HealTarget { b: AtkBase { attack_effect: atk, player: p, opponent: opp, attack, source, target }, damage: 40 })?;
        Ok(())
    })();
    g.release_fx(atk);
    r
}
