//! Handheld Fan (TWM, tool): whenever the Active Pokémon this card is
//! attached to takes damage from an opponent's attack, move an Energy from
//! the attacking Pokémon to 1 of your opponent's Benched Pokémon.
//!
//! Twinleaf quirks kept: the lock check is a bare ToolEffect for the
//! attacking player; the "has Bench" check looks at the Fan owner's Bench
//! while the prompt (answered by the Fan owner) targets the attacker's Bench;
//! the Energy comes from the attacking player's current Active; the move may
//! be skipped (min 0).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "HandyFan", mask: mask(&[k::AFTER_DAMAGE]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let (b, damage) = match *g.e(e) {
        Effect::AfterDamage { b, damage } => (b, damage),
        _ => return Ok(()),
    };
    let t = b.target;
    if !g.st.slot(t.p as usize, t.s).tools.contains(me) {
        return Ok(());
    }
    if damage <= 0 || b.player == t.p || g.st.players[t.p as usize].active != t.s {
        return Ok(());
    }
    if g.run_fx(Effect::Tool { p: b.player, card: me }).is_err() {
        return Ok(());
    }
    if g.st.phase != GamePhase::Attack {
        return Ok(());
    }
    let p = b.player as usize;
    let o = 1 - p;
    let has_bench = g.st.players[o].bench.iter().any(|s| !g.st.players[o].slots[*s as usize].cards.is_empty());
    if !has_bench {
        return Ok(());
    }
    let mut slots = SVec::new();
    slots.push(SlotType::Bench as u8);
    let src = ListRef::Slot(p as u8, g.st.players[p].active);
    let n = g.lst(src).len().min(255) as u8;
    let mut o_opts = AttachOpts::new(n);
    o_opts.allow_cancel = false;
    o_opts.min = 0;
    o_opts.max = 1;
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    let id = g.player_id(o);
    g.prompt(
        id,
        "ATTACH_ENERGY_TO_BENCH",
        PromptKind::AttachEnergy { cards: src, player_type: PlayerType::TopPlayer, slots, filter: Filter::super_type(SuperType::Energy), o: o_opts },
        Cont::Card { card: me, frame: f },
    );
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let o = 1 - p;
    if let Some(Res::Attach(ts)) = results.first() {
        for (to, c) in ts.iter() {
            let target = get_target(&g.st, o, *to)?;
            let a = g.st.players[p].active;
            move_cards(g, ListRef::Slot(p as u8, a), target.list(), &[*c], me)?;
        }
    }
    Ok(())
}
