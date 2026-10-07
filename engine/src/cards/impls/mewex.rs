//! Mew ex (30C / M6a): Memory Helix — a passive Ability: this Pokémon can use
//! the attacks of any of your Benched Pokémon. They are added to its attack
//! options (CheckPokemonAttacksEffect.copiedAttacks) while it is Active and
//! the Ability isn't blocked; the AttackAction then runs the chosen one with
//! `delegateFrom` (see `copy_attack.rs`). Teleportation Burst — 30, you may
//! switch this Pokémon with 1 of your Benched Pokémon.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "Mewex",
    mask: mask(&[k::CHECK_POKEMON_ATTACKS, k::AFTER_ATTACK]),
    reduce,
    resume: Some(resume),
    coin: None,
    can_play: None,
};

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::CheckPokemonAttacks { p, .. } = *g.e(e) {
        let p = p as usize;
        if g.st.active_pokemon(p) != Some(me) || is_ability_blocked(g, p, me, None) {
            return Ok(());
        }
        let mut add: SVec<AttackRef, 32> = SVec::new();
        let bench: Vec<SlotId> = g.st.players[p].bench.iter().copied().collect();
        for b in bench {
            if let Some(c) = g.st.slot_pokemon(p, b) {
                for i in 0..g.st.cdef(c).attacks.len() {
                    add.push(AttackRef { card: c, index: i as u8 });
                }
            }
        }
        if let Effect::CheckPokemonAttacks { attacks, copied, .. } = g.e_mut(e) {
            for a in add.iter() {
                attacks.push(*a);
                copied.push(*a);
            }
        }
        return Ok(());
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
