//! Cofagrigus (WHT 40): Extended Damagriiigus — move all damage counters from
//! 1 of your Benched Pokémon to 1 of your opponent's Pokémon. Perplex — 60;
//! your opponent's Active Pokémon is now Confused.
//!
//! Twinleaf: Extended Damagriiigus does nothing without a damaged Benched
//! Pokémon. Otherwise a non-cancellable ChoosePokemonPrompt over the Bench
//! (the Active and undamaged Pokémon blocked), then a second one over the
//! opponent's Active and Bench. The callback reads the source's damage after
//! the second prompt; with none left it stops, else a MoveDamageCountersEffect
//! (preventable) and a MoveCountersAttackEffect (source = the Benched
//! Pokémon, target = the chosen Pokémon) are reduced; the counters always
//! leave the source, and reach the target unless the effect was prevented.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "CofagrigusWHTPool", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let has_damaged_bench = {
            let pl = &g.st.players[p];
            pl.bench.iter().any(|b| !pl.slots[*b as usize].cards.is_empty() && pl.slots[*b as usize].damage > 0)
        };
        if !has_damaged_bench {
            return Ok(());
        }
        let mut blocked = TargetList::new();
        let active = g.st.players[p].active;
        for (s, _, t) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
            if s == active || g.st.slot(p, s).damage == 0 {
                blocked.push(t);
            }
        }
        let mut slots = SVec::new();
        slots.push(SlotType::Bench as u8);
        g.retain_fx(e);
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        f.e[0] = e;
        let id = g.player_id(p);
        g.prompt(
            id,
            "CHOOSE_POKEMON",
            PromptKind::ChoosePokemon { player_type: PlayerType::BottomPlayer, slots, min: 1, max: 1, allow_cancel: false, blocked },
            Cont::Card { card: me, frame: f },
        );
    }

    if was_attack_used(g, e, 1, me) {
        add_special_conditions_to_opponent_active(g, e, &[SpecialCondition::Confused])?;
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let atk = f.e[0];
    let p = f.a[0] as usize;
    let sel = results.first().map(|r| r.slots().to_vec()).unwrap_or_default();
    match f.stage {
        1 => {
            let src = match sel.first() {
                Some(s) => *s,
                None => {
                    g.release_fx(atk);
                    return Ok(());
                }
            };
            let mut slots = SVec::new();
            slots.push(SlotType::Active as u8);
            slots.push(SlotType::Bench as u8);
            let mut nf = CardFrame::at(2);
            nf.a[0] = p as i32;
            nf.a[1] = src.s as i32;
            nf.e[0] = atk;
            let id = g.player_id(p);
            g.prompt(
                id,
                "CHOOSE_POKEMON_TO_DAMAGE",
                PromptKind::ChoosePokemon { player_type: PlayerType::TopPlayer, slots, min: 1, max: 1, allow_cancel: false, blocked: SVec::new() },
                Cont::Card { card: me, frame: nf },
            );
            Ok(())
        }
        2 => {
            let r = (|| -> R {
                let tgt = match sel.first() {
                    Some(t) => *t,
                    None => return Ok(()),
                };
                let src = SlotRef::new(p, f.a[1] as SlotId);
                let move_damage = g.st.slot(p, src.s).damage;
                if move_damage <= 0 {
                    return Ok(());
                }
                let (_, prevented) = g.run_fx(Effect::MoveDamageCounters { p: p as u8 })?;
                if prevented {
                    return Ok(());
                }
                let (opp, attack) = match *g.e(atk) {
                    Effect::Attack { opp, attack, .. } => (opp, attack),
                    _ => return Ok(()),
                };
                let b = AtkBase { attack_effect: atk, player: p as u8, opponent: opp, attack, source: src, target: tgt };
                let (fin, prevented) = g.run_fx(Effect::MoveCounters { b, damage: move_damage })?;
                if let Effect::MoveCounters { b, damage } = fin {
                    let s = &mut g.st.players[b.source.p as usize].slots[b.source.s as usize];
                    s.damage -= damage;
                    if s.damage < 0 {
                        s.damage = 0;
                    }
                    if !prevented {
                        g.st.players[b.target.p as usize].slots[b.target.s as usize].damage += damage;
                    }
                }
                Ok(())
            })();
            g.release_fx(atk);
            r
        }
        _ => Ok(()),
    }
}
