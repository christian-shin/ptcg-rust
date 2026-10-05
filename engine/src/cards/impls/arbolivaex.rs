//! Arboliva ex (DRI): Oil Salvo — choose 1 of your opponent's Pokémon 6
//! times; 20 damage each time, not affected by Weakness or Resistance.
//! Aroma Shot — 160; this Pokémon recovers from all Special Conditions.
//!
//! Twinleaf: Oil Salvo is a non-cancellable PutDamagePrompt (120 damage in
//! multiples of 20, per-target cap = printed HP + 120), then
//! DAMAGE_OPPONENT_POKEMON per entry, so the Active's share goes through a
//! DealDamageEffect. Fixed (R1-1): the attack sets `ignoreWeakness` and
//! `ignoreResistance` (the damage isn't affected by Weakness or Resistance),
//! and Aroma Shot removes all five Special Conditions from the attacker's
//! Active (RemoveSpecialConditionsEffect(effect, undefined); it used to have
//! no handler).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Arbolivaex", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 1, me) {
        // Aroma Shot: this Pokémon recovers from all Special Conditions.
        if let Effect::Attack { p, opp, attack, source, .. } = *g.e(e) {
            let target = SlotRef::new(p as usize, g.st.players[p as usize].active);
            let b = AtkBase { attack_effect: e, player: p, opponent: opp, attack, source, target };
            let mut cs = SVec::new();
            for c in [SpecialCondition::Paralyzed, SpecialCondition::Confused, SpecialCondition::Asleep, SpecialCondition::Poisoned, SpecialCondition::Burned] {
                cs.push(c as u8);
            }
            g.run_fx(Effect::RemoveSpecialConditions { b, conditions: cs })?;
        }
        return Ok(());
    }
    if !was_attack_used(g, e, 0, me) {
        return Ok(());
    }
    // Oil Salvo: this damage isn't affected by Weakness or Resistance.
    if let Effect::Attack { ignore_weakness, ignore_resistance, .. } = g.e_mut(e) {
        *ignore_weakness = true;
        *ignore_resistance = true;
    }
    let (p, o) = match *g.e(e) {
        Effect::Attack { p, opp, .. } => (p as usize, opp as usize),
        _ => return Ok(()),
    };
    let mut max_allowed: SVec<(CardTarget, i32), 16> = SVec::new();
    for (_, c, t) in for_each_pokemon(g, o, PlayerType::TopPlayer).iter() {
        max_allowed.push((*t, g.st.cdef(*c).hp + 120));
    }
    let mut slots = SVec::new();
    slots.push(SlotType::Active as u8);
    slots.push(SlotType::Bench as u8);
    g.retain_fx(e);
    let mut f = CardFrame::at(1);
    f.e[0] = e;
    let id = g.player_id(p);
    g.prompt(
        id,
        "CHOOSE_POKEMON_TO_DAMAGE",
        PromptKind::PutDamage {
            player_type: PlayerType::TopPlayer,
            slots,
            damage: 120,
            max_allowed,
            allow_cancel: false,
            blocked: SVec::new(),
            allow_partial: false,
            damage_multiple: 20,
        },
        Cont::Card { card: me, frame: f },
    );
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let atk = f.e[0];
    let r = (|| -> R {
        let p = match *g.e(atk) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let map: SVec<(CardTarget, i32), 16> = match results.first().copied().unwrap_or(Res::Null) {
            Res::DamageMap(m) => m,
            _ => SVec::new(),
        };
        for (t, damage) in map.iter() {
            let target = get_target(&g.st, p, *t)?;
            damage_opponent_pokemon(g, atk, *damage, &[target])?;
        }
        Ok(())
    })();
    g.release_fx(atk);
    r
}
