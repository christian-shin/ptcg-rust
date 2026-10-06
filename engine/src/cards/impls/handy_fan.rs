//! Handheld Fan (TWM, tool): whenever the Active Pokémon this card is
//! attached to takes damage from an opponent's attack, move an Energy from
//! the attacking Pokémon to 1 of your opponent's Benched Pokémon.
//!
//! Twinleaf: the lock check is a bare ToolEffect for the Fan's owner; the
//! prompt (answered by the Fan owner) moves an Energy from the attacking
//! Pokémon to the attacker's Bench ("your opponent's Benched Pokémon" from
//! the Fan owner's view).
//!
//! Fixed (phase 4b, W4): the "has Bench" check looked at the Fan owner's own
//! Bench (now the attacker's), the ToolEffect probe used the attacking player
//! (now the owner), and the move could be skipped (min 0): it now needs
//! min 1, and nothing happens when the attacker has no Energy.
//!
//! Fixed (phase 4b, R7F-10; rulings 1625, 1649, 1650, 1651): the prompt opened
//! inside the damage step and moved an Energy off the player's Active. The
//! damage now only arms a HANDY_FAN_MARKER on the attacking Pokémon's slot;
//! the effect resolves in AfterAttackEffect, after everything the attack did
//! (an Energy the attack discards or puts away is no longer there, Boomerang
//! Energy is attached again first, an attacker that left play has nothing to
//! move). The attacker is the Pokémon that used the attack even when it is
//! on the Bench (Alakazam ex's Dimensional Hand); it moves the Energy to one
//! of the attacking player's other Benched Pokémon. The marker is cleared at
//! the end of the turn when no AfterAttackEffect came.
//!
//! Fixed (phase 4b, F1, general step 7 mechanism; rulings 1625, 1649, 1650, 1651): the damage records the trigger
//! (`Game::attack_trigger`) and it resolves after the attack's own effects and the prompts they opened (the
//! AttackTrigger effect). The attacker is the Pokémon that used the attack wherever it is then (it can have been
//! switched to the Bench); it moves the Energy to one of the attacking player's other Benched Pokémon.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "HandyFan",
    mask: mask(&[k::AFTER_DAMAGE, k::ATTACK_TRIGGER]),
    reduce,
    resume: Some(resume),
    coin: None,
    can_play: None,
};

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    match *g.e(e) {
        Effect::AfterDamage { b, damage } => {
            let t = b.target;
            if !g.st.slot(t.p as usize, t.s).tools.contains(me) {
                return Ok(());
            }
            if damage <= 0 || b.player == t.p || g.st.players[t.p as usize].active != t.s {
                return Ok(());
            }
            g.attack_trigger(b, damage, me, None, true)?;
        }
        Effect::AttackTrigger { p, opp, card, target: t, source: attacker, source_in_play, retaliate: None, .. } if card == me => {
            if !g.st.slot(t.p as usize, t.s).tools.contains(me) {
                return Ok(());
            }
            if g.run_fx(Effect::Tool { p: opp, card: me }).is_err() {
                return Ok(());
            }
            if g.st.phase != GamePhase::Attack || !source_in_play {
                return Ok(());
            }
            let p = p as usize;
            let o = opp as usize;
            let s = attacker.s;
            let pl = &g.st.players[p];
            let bench_index = pl.bench.iter().position(|b| *b == s);
            let has_bench = pl.bench.iter().any(|b| *b != s && !pl.slots[*b as usize].cards.is_empty());
            let has_energy = g.st.slot(p, s).cards.iter().any(|c| g.st.cdef(c).is_energy());
            if !has_bench || !has_energy {
                return Ok(());
            }
            let mut slots = SVec::new();
            slots.push(SlotType::Bench as u8);
            let src = ListRef::Slot(p as u8, s);
            let n = g.lst(src).len().min(255) as u8;
            let mut o_opts = AttachOpts::new(n);
            o_opts.allow_cancel = false;
            o_opts.min = 1;
            o_opts.max = 1;
            if let Some(i) = bench_index {
                o_opts.blocked_to.push(CardTarget::new(PlayerType::TopPlayer, SlotType::Bench, i as u8));
            }
            let mut f = CardFrame::at(1);
            f.a[0] = p as i32;
            f.a[1] = s as i32;
            let id = g.player_id(o);
            g.prompt(
                id,
                "ATTACH_ENERGY_TO_BENCH",
                PromptKind::AttachEnergy { cards: src, player_type: PlayerType::TopPlayer, slots, filter: Filter::super_type(SuperType::Energy), o: o_opts },
                Cont::Card { card: me, frame: f },
            );
        }
        _ => {}
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let s = f.a[1] as SlotId;
    let o = 1 - p;
    if let Some(Res::Attach(ts)) = results.first() {
        for (to, c) in ts.iter() {
            let target = get_target(&g.st, o, *to)?;
            move_cards(g, ListRef::Slot(p as u8, s), target.list(), &[*c], me)?;
        }
    }
    Ok(())
}
