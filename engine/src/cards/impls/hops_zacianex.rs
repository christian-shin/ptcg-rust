//! Hop's Zacian ex (JTG 111): Insta-Strike — 30, and 30 damage to 1 of your
//! opponent's Benched Pokémon. Brave Slash — 240; during your next turn this
//! Pokémon can't use Brave Slash.
//!
//! Twinleaf: no prompt without a Benched Pokémon; ChoosePokemonPrompt (bench,
//! no cancel) then a PutDamageEffect of 30. Brave Slash pushes its name onto
//! the Active's `cannotUseAttacksNextTurnPending` if missing.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "HopsZacianex", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let o = 1 - p;
        let pl = &g.st.players[o];
        if !pl.bench.iter().any(|b| !pl.slots[*b as usize].cards.is_empty()) {
            return Ok(());
        }
        let mut slots = SVec::new();
        slots.push(SlotType::Bench as u8);
        g.retain_fx(e);
        let mut f = CardFrame::at(1);
        f.e[0] = e;
        let id = g.player_id(p);
        g.prompt(
            id,
            "CHOOSE_POKEMON_TO_DAMAGE",
            PromptKind::ChoosePokemon { player_type: PlayerType::TopPlayer, slots, min: 1, max: 1, allow_cancel: false, blocked: SVec::new() },
            Cont::Card { card: me, frame: f },
        );
        return Ok(());
    }
    if was_attack_used(g, e, 1, me) {
        if let Effect::Attack { p, .. } = *g.e(e) {
            let p = p as usize;
            let a = g.st.players[p].active;
            let pending = &mut g.st.players[p].slots[a as usize].cannot_use_attacks_next_turn_pending;
            if !pending.iter().any(|n| *n == "Brave Slash") {
                pending.push("Brave Slash");
            }
        }
    }
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let atk = f.e[0];
    let sel = results.first().copied().unwrap_or(Res::Null);
    let sel = sel.slots();
    let r = if sel.is_empty() { Ok(()) } else { put_damage(g, atk, 30, sel[0]) };
    g.release_fx(atk);
    r
}
