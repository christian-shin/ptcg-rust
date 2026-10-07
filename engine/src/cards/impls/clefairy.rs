//! Clefairy (M3 / POR 30): Follow Me — switch in 1 of your opponent's
//! Benched Pokémon. Flop — 30.
//!
//! Twinleaf: AFTER_ATTACK; with no Benched opponent Pokémon the attack is still
//! usable and does nothing (phase 4b R7E, ruling 1790: it used to throw
//! CANNOT_USE_ATTACK); GUST_OPPONENT_BENCHED_POKEMON(player, { sourceEffect }) prompts the
//! attacker (no cancel) and then reduces a GustOpponentBenchEffect built on a
//! fresh AttackEffect (preventable, e.g. Mist Energy), whose reducer does the
//! switch.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Clefairy", mask: mask(&[k::AFTER_ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if after_attack_used(g, e, 0, me) {
        if let Effect::AfterAttack { p, opp, attack, .. } = *g.e(e) {
            let (p, o) = (p as usize, opp as usize);
            let pl = &g.st.players[o];
            if !pl.bench.iter().any(|b| !pl.slots[*b as usize].cards.is_empty()) {
                return Ok(());
            }
            let mut slots = SVec::new();
            slots.push(SlotType::Bench as u8);
            let mut f = CardFrame::at(1);
            f.a[0] = p as i32;
            f.a[1] = attack.index as i32;
            f.a[2] = attack.card as i32;
            let id = g.player_id(p);
            g.prompt(
                id,
                "CHOOSE_POKEMON_TO_SWITCH",
                PromptKind::ChoosePokemon { player_type: PlayerType::TopPlayer, slots, min: 1, max: 1, allow_cancel: false, blocked: SVec::new() },
                Cont::Card { card: me, frame: f },
            );
        }
    }
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let o = 1 - p;
    let attack = AttackRef { card: f.a[2] as CardId, index: f.a[1] as u8 };
    let first = results.first().copied().unwrap_or(Res::Null);
    let sel = first.slots();
    if sel.is_empty() {
        return Ok(());
    }
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
    let target = sel[0];
    let b = AtkBase { attack_effect: atk, player: p as u8, opponent: o as u8, attack, source, target };
    let r = g.run_fx(Effect::GustOpponentBench { b });
    g.release_fx(atk);
    r?;
    Ok(())
}
