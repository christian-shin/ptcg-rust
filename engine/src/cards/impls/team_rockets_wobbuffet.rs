//! Team Rocket's Wobbuffet (DRI): Rocket Mirror — move all damage counters
//! from 1 of your Benched Team Rocket's Pokémon to your opponent's Active
//! Pokémon. Jet Headbutt — 70.
//!
//! Twinleaf: every Bench position that is not a damaged Team Rocket's Pokémon
//! (empty positions included) is blocked; with no damaged one the attack does
//! nothing.
//!
//! Fixed (phase 4b, R7F-5; ruling 1665): the callback moved `damage` directly,
//! so Mist Energy, Repelling Veil, ... on the opponent's Active did not stop
//! it. Like Cofagrigus WHT it now checks MoveDamageCountersEffect and
//! reduces a MoveCountersAttackEffect (source = the Benched Pokémon, target =
//! the opponent's Active): the counters always leave the Benched Pokémon, and
//! are placed only when the move was not prevented.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "TeamRocketsWobbuffet", mask: mask(&[k::AFTER_ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if after_attack_used(g, e, 0, me) {
        let e = real_attack(g, e);
        let (p, opp) = match *g.e(e) {
            Effect::Attack { p, opp, .. } => (p as usize, opp),
            _ => return Ok(()),
        };
        let mut blocked: TargetList = SVec::new();
        let mut any = false;
        let bench: Vec<SlotId> = g.st.players[p].bench.iter().copied().collect();
        for (i, s) in bench.iter().enumerate() {
            let ok = !g.st.slot(p, *s).cards.is_empty()
                && g.st.slot_pokemon(p, *s).map_or(false, |c| g.st.cdef(c).has_tag(tag::TEAM_ROCKET))
                && g.st.slot(p, *s).damage > 0;
            if ok {
                any = true;
            } else {
                blocked.push(CardTarget::new(PlayerType::BottomPlayer, SlotType::Bench, i as u8));
            }
        }
        if !any {
            return Ok(());
        }
        let mut slots = SVec::new();
        slots.push(SlotType::Bench as u8);
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        f.a[1] = opp as i32;
        f.e[0] = e;
        g.retain_fx(e);
        let id = g.player_id(p);
        g.prompt(
            id,
            "CHOOSE_POKEMON_TO_DAMAGE",
            PromptKind::ChoosePokemon { player_type: PlayerType::BottomPlayer, slots, min: 1, max: 1, allow_cancel: false, blocked },
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
    let r = (|| -> R {
        let p = f.a[0] as usize;
        let opp = f.a[1] as usize;
        let src = match results.first().copied().unwrap_or(Res::Null).slots().first() {
            Some(t) => *t,
            None => return Ok(()),
        };
        let dmg = g.st.slot(src.p as usize, src.s).damage;
        if dmg <= 0 {
            return Ok(());
        }
        // "Damage counters can't be moved" cancels the whole move.
        let (_, prevented) = g.run_fx(Effect::MoveDamageCounters { p: p as u8 })?;
        if prevented {
            return Ok(());
        }
        let attack = match *g.e(atk) {
            Effect::Attack { attack, .. } => attack,
            _ => return Ok(()),
        };
        let a = g.st.players[opp].active;
        let b = AtkBase { attack_effect: atk, player: p as u8, opponent: opp as u8, attack, source: SlotRef::new(src.p as usize, src.s), target: SlotRef::new(opp, a) };
        let (fin, prevented) = g.run_fx(Effect::MoveCounters { b, damage: dmg })?;
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
