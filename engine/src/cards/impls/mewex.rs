//! Mew ex (30C / M6a): Memory Helix — this Pokémon can use the attacks of
//! any of your Benched Pokémon (COPY_ATTACK_VIA_ABILITY, see
//! `copy_attack.rs`). Teleportation Burst — 30, you may switch this Pokémon
//! with 1 of your Benched Pokémon.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "Mewex",
    mask: mask(&[k::POWER, k::AFTER_ATTACK]),
    reduce,
    resume: Some(resume),
    coin: None,
    can_play: None,
};

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_power_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Power { p, .. } => p as usize,
            _ => return Ok(()),
        };
        if is_ability_blocked(g, p, me, None) {
            bail!("BLOCKED_BY_EFFECT");
        }
        return crate::copy_attack::copy_attack_via_ability(g, p, me);
    }

    if after_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::AfterAttack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let pl = &g.st.players[p];
        if pl.bench.iter().any(|b| !pl.slots[*b as usize].cards.is_empty()) {
            let mut f = CardFrame::at(1);
            f.a[0] = p as i32;
            confirmation_prompt(g, p, "WANT_TO_SWITCH_POKEMON", Cont::Card { card: me, frame: f });
        }
    }
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    if results.first().map(|r| r.as_bool()).unwrap_or(false) {
        switch_active_with_benched(g, f.a[0] as usize);
    }
    Ok(())
}
