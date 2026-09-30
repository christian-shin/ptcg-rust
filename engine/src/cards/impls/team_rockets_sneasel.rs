//! Team Rocket's Sneasel (DRI): Scratch — 20. Backstab — 20 damage to 1 of
//! your opponent's Benched Pokémon for each damage counter on it.
//!
//! Twinleaf: a PutDamageEffect of `target.damage * 2` (read when the prompt
//! resolves) on the chosen Benched Pokémon.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "TeamRocketsSneasel", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 1, me) {
        let (p, opp) = match *g.e(e) {
            Effect::Attack { p, opp, .. } => (p as usize, opp as usize),
            _ => return Ok(()),
        };
        let pl = &g.st.players[opp];
        if !pl.bench.iter().any(|b| !pl.slots[*b as usize].cards.is_empty()) {
            return Ok(());
        }
        g.retain_fx(e);
        let mut f = CardFrame::at(1);
        f.e[0] = e;
        let mut slots = SVec::new();
        slots.push(SlotType::Bench as u8);
        let id = g.player_id(p);
        g.prompt(
            id,
            "CHOOSE_POKEMON_TO_DAMAGE",
            PromptKind::ChoosePokemon { player_type: PlayerType::TopPlayer, slots, min: 1, max: 1, allow_cancel: false, blocked: SVec::new() },
            Cont::Card { card: me, frame: f },
        );
    }
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let atk = f.e[0];
    let t = match results.first().copied().unwrap_or(Res::Null).slots().first() {
        Some(t) => *t,
        None => {
            g.release_fx(atk);
            bail!("TypeError: Cannot read properties of undefined");
        }
    };
    let dmg = g.st.slot(t.p as usize, t.s).damage * 2;
    let r = put_damage(g, atk, dmg, t);
    g.release_fx(atk);
    r
}
