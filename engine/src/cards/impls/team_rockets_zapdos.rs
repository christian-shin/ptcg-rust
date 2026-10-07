//! Team Rocket's Zapdos (DRI): Jamming Wave — 30; you may move an Energy
//! from your opponent's Active Pokémon to 1 of their Benched Pokémon.
//! Bad Thunder — 60+, 60 more if this Pokémon has Team Rocket's Energy.
//!
//! Twinleaf: CONFIRMATION_PROMPT; on yes, nothing without an opponent's
//! Benched Pokémon or an Energy card in their Active, else a non-cancellable
//! AttachEnergyPrompt (opponent's Active → TOP_PLAYER Bench, min 1 max 1).
//! Fixed in phase 4b: the callback resolved `getTarget(state, opponent, to)`,
//! which reads TOP_PLAYER as the attacker's Bench (the Energy landed there,
//! even in an empty slot); it now uses the attacker as the perspective, so the
//! Energy goes to the opponent's Benched Pokémon. Bad Thunder checks the
//! Energy name "Team Rocket's Energy" (it compared "Team Rocket Energy", so
//! the +60 never applied).
//!
//! Fixed (phase 4b, R7F-7; ruling 1843): the move was a plain MOVE_CARDS, so
//! Mist Energy (or any effect that prevents the effects of attacks) on the
//! Defending Pokémon did not stop it; it is now a MoveOpponentEnergyEffect
//! (target = the Defending Pokémon), like Elgyem's Slight Shift.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "TeamRocketsZapdos", mask: mask(&[k::ATTACK, k::AFTER_ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if after_attack_used(g, e, 0, me) {
        let e = real_attack(g, e);
        let (p, o) = match *g.e(e) {
            Effect::Attack { p, opp, .. } => (p as usize, opp as usize),
            _ => return Ok(()),
        };
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        f.a[1] = o as i32;
        f.e[0] = e;
        g.retain_fx(e);
        confirmation_prompt(g, p, "WANT_TO_USE_ABILITY", Cont::Card { card: me, frame: f });
    }
    if was_attack_used(g, e, 1, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let a = g.st.players[p].active;
        let has = g.st.slot(p, a).cards.iter().any(|c| {
            let d = g.st.cdef(c);
            d.is_energy() && d.name == "Team Rocket's Energy"
        });
        if has {
            if let Effect::Attack { damage, .. } = g.e_mut(e) {
                *damage += 60;
            }
        }
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let atk = f.e[0];
    // Stage 1 hands the retained AttackEffect to the Energy prompt of stage 2.
    let opened = f.stage == 1 && {
        let o = f.a[1] as usize;
        let pl = &g.st.players[o];
        results.first().copied().unwrap_or(Res::Null).as_bool()
            && pl.bench.iter().any(|b| !pl.slots[*b as usize].cards.is_empty())
            && pl.slots[pl.active as usize].cards.iter().any(|c| g.st.cdef(c).is_energy())
    };
    let r = resume_inner(g, me, f, results);
    if !opened || r.is_err() {
        g.release_fx(atk);
    }
    r
}

fn resume_inner(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    let o = f.a[1] as usize;
    let first = results.first().copied().unwrap_or(Res::Null);
    match f.stage {
        1 => {
            if !first.as_bool() {
                return Ok(());
            }
            let pl = &g.st.players[o];
            if !pl.bench.iter().any(|b| !pl.slots[*b as usize].cards.is_empty()) {
                return Ok(());
            }
            let a = pl.active;
            if !pl.slots[a as usize].cards.iter().any(|c| g.st.cdef(c).is_energy()) {
                return Ok(());
            }
            let n = g.st.slot(o, a).cards.len() as u8;
            let mut opts = AttachOpts::new(n);
            opts.allow_cancel = false;
            opts.min = 1;
            opts.max = 1;
            let mut slots = SVec::new();
            slots.push(SlotType::Bench as u8);
            let mut nf = CardFrame::at(2);
            nf.a[0] = p as i32;
            nf.a[1] = o as i32;
            nf.e[0] = f.e[0];
            let id = g.player_id(p);
            g.prompt(
                id,
                "ATTACH_ENERGY_TO_BENCH",
                PromptKind::AttachEnergy {
                    cards: ListRef::Slot(o as u8, a),
                    player_type: PlayerType::TopPlayer,
                    slots,
                    filter: Filter::super_type(SuperType::Energy),
                    o: opts,
                },
                Cont::Card { card: me, frame: nf },
            );
            Ok(())
        }
        2 => {
            let transfers: SVec<(CardTarget, CardId), 64> = match first {
                Res::Attach(t) => t,
                _ => SVec::new(),
            };
            let (opp, attack, source) = match *g.e(f.e[0]) {
                Effect::Attack { opp, attack, source, .. } => (opp, attack, source),
                _ => return Ok(()),
            };
            let active = SlotRef::new(o, g.st.players[o].active);
            for (to, c) in transfers.iter().copied() {
                let target = get_target(&g.st, p, to)?;
                let b = AtkBase { attack_effect: f.e[0], player: p as u8, opponent: opp, attack, source, target: active };
                g.run_fx(Effect::MoveOpponentEnergy { b, card: c, destination: target })?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
