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
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "TeamRocketsZapdos", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let (p, o) = match *g.e(e) {
            Effect::Attack { p, opp, .. } => (p as usize, opp as usize),
            _ => return Ok(()),
        };
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        f.a[1] = o as i32;
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
            let transfers: SVec<(CardTarget, CardId), 16> = match first {
                Res::Attach(t) => t,
                _ => SVec::new(),
            };
            let src = ListRef::Slot(o as u8, g.st.players[o].active);
            for (to, c) in transfers.iter().copied() {
                let target = get_target(&g.st, p, to)?;
                move_cards(g, src, target.list(), &[c], me)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
