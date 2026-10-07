//! Yanma (DRI): Whirlwind — switch out your opponent's Active Pokémon to the
//! Bench (your opponent chooses the new Active Pokémon). Razor Wing — 30.
//!
//! AFTER_ATTACK → SWITCH_OUT_OPPONENT_ACTIVE_POKEMON(player, { sourceEffect })
//! (same flow as Bayleef M1S): a preventable SwitchOutOpponentsActiveEffect
//! probe, the opponent's ChoosePokemonPrompt, then another effect carrying
//! the chosen bench target. Each effect is built on a fresh AttackEffect.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Yanma@DRI", mask: mask(&[k::AFTER_ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if after_attack_used(g, e, 0, me) {
        if let Effect::AfterAttack { p, opp, attack, .. } = *g.e(e) {
            let (p, o) = (p as usize, opp as usize);
            let pl = &g.st.players[o];
            if !pl.bench.iter().any(|b| !pl.slots[*b as usize].cards.is_empty()) {
                return Ok(());
            }
            if run_switch_out(g, p, o, attack, None)? {
                return Ok(());
            }
            let mut slots = SVec::new();
            slots.push(SlotType::Bench as u8);
            let mut f = CardFrame::at(1);
            f.a[0] = p as i32;
            f.a[1] = attack.index as i32;
            f.a[2] = attack.card as i32;
            let id = g.player_id(o);
            g.prompt(
                id,
                "CHOOSE_POKEMON_TO_SWITCH",
                PromptKind::ChoosePokemon { player_type: PlayerType::BottomPlayer, slots, min: 1, max: 1, allow_cancel: false, blocked: SVec::new() },
                Cont::Card { card: me, frame: f },
            );
        }
    }
    Ok(())
}

/// Reduce a SwitchOutOpponentsActiveEffect built on `new AttackEffect(player,
/// opponent, attack)`; returns `preventDefault`.
fn run_switch_out(g: &mut Game, p: usize, o: usize, attack: AttackRef, bench_target: Option<SlotRef>) -> R<bool> {
    let source = SlotRef::new(p, g.st.players[p].active);
    let atk = g.new_fx(Effect::Attack {
        p: p as u8,
        opp: o as u8,
        attack,
        damage: 0,
        ignore_weakness: false,
        ignore_resistance: false,
        ignore_defender_effects: false,
        source,
        barrage_used: false,
    });
    let target = SlotRef::new(o, g.st.players[o].active);
    let b = AtkBase { attack_effect: atk, player: p as u8, opponent: o as u8, attack, source, target };
    let r = g.run_fx(Effect::SwitchOutOpponentsActive { b, bench_target });
    g.release_fx(atk);
    Ok(r?.1)
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let attack = AttackRef { card: f.a[2] as CardId, index: f.a[1] as u8 };
    let first = results.first().copied().unwrap_or(Res::Null);
    let sel = first.slots();
    if sel.is_empty() {
        return Ok(());
    }
    run_switch_out(g, p, 1 - p, attack, Some(sel[0]))?;
    Ok(())
}
