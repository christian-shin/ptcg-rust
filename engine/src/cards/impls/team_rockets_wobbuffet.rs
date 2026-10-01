//! Team Rocket's Wobbuffet (DRI): Rocket Mirror — move all damage counters
//! from 1 of your Benched Team Rocket's Pokémon to your opponent's Active
//! Pokémon. Jet Headbutt — 70.
//!
//! Twinleaf: every Bench position that is not a damaged Team Rocket's Pokémon
//! (empty positions included) is blocked; with no damaged one the attack does
//! nothing. The callback moves `damage` directly (no effects).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "TeamRocketsWobbuffet", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
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
    let opp = f.a[1] as usize;
    let t = match results.first().copied().unwrap_or(Res::Null).slots().first() {
        Some(t) => *t,
        None => return Ok(()),
    };
    let dmg = g.st.slot(t.p as usize, t.s).damage;
    g.st.players[t.p as usize].slots[t.s as usize].damage = 0;
    let a = g.st.players[opp].active;
    g.st.players[opp].slots[a as usize].damage += dmg;
    Ok(())
}
