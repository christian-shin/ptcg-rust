//! Metagross (TEF): Meteor Mash — 60; during your next turn, this Pokémon's
//! Meteor Mash attack does 60 more damage. Luster Blast — 200; discard 2
//! Energy from this Pokémon.
//!
//! Twinleaf (temporal-forces file): NEXT_TURN_ATTACK_BONUS runs on every
//! AttackEffect whose attacker is this card: an armed
//! `nextTurnAttackDamageBonus` from "Metagross TEF" whose attack name matches
//! adds its bonus, and using Meteor Mash arms
//! `nextTurnAttackDamageBonusPending` (rolled over at the end of the turn).
//! Luster Blast prices the discard as [C][C] on the Active's
//! CheckProvidedEnergyEffect (non-cancellable ChooseEnergyPrompt) and reduces
//! a DiscardCardsEffect on `player.active`.
use crate::cards::prelude::*;
use crate::state::NextTurnAttackDamageBonus;

pub static IMPL: CardImpl = CardImpl { class: "Metagross@TEF", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

const METEOR_MASH: &str = "Meteor Mash";

fn next_turn_attack_bonus(g: &mut Game, me: CardId, e: EffId) {
    let (attack, source) = match *g.e(e) {
        Effect::Attack { attack, source, .. } => (attack, source),
        _ => return,
    };
    if g.st.slot_pokemon(source.p as usize, source.s) != Some(me) {
        return;
    }
    let full_name = g.st.cdef(me).full_name;
    let attack_name = g.st.cdef(attack.card).attacks[attack.idx()].name;
    let slot = &g.st.players[source.p as usize].slots[source.s as usize];
    let bonus = match slot.next_turn_attack_damage_bonus {
        Some(b) if b.source_card_name == full_name && (b.attack_name == "*" || b.attack_name == attack_name) => b.bonus_damage,
        _ => 0,
    };
    if bonus != 0 {
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage += bonus;
        }
    }
    if attack_name != METEOR_MASH {
        return;
    }
    g.st.players[source.p as usize].slots[source.s as usize].next_turn_attack_damage_bonus_pending =
        Some(NextTurnAttackDamageBonus { attack_name: METEOR_MASH, bonus_damage: 60, source_card_name: full_name });
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    next_turn_attack_bonus(g, me, e);

    if was_attack_used(g, e, 1, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let active = SlotRef::new(p, g.st.players[p].active);
        let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p: p as u8, source: active, energy_map: SVec::new() })?;
        let energy = match pe {
            Effect::CheckProvidedEnergy { energy_map, .. } => energy_map,
            _ => SVec::new(),
        };
        let mut cost = SVec::new();
        cost.push(ct::COLORLESS);
        cost.push(ct::COLORLESS);
        g.retain_fx(e);
        let mut f = CardFrame::at(1);
        f.e[0] = e;
        let id = g.player_id(p);
        g.prompt(id, "CHOOSE_ENERGIES_TO_DISCARD", PromptKind::ChooseEnergy { energy, cost, allow_cancel: false }, Cont::Card { card: me, frame: f });
    }
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let atk = f.e[0];
    let mut cards: SVec<CardId, 64> = SVec::new();
    if let Some(Res::Energy(c)) = results.first() {
        for x in c.as_slice() {
            cards.push(*x);
        }
    }
    let r = (|| -> R {
        let (p, opp, attack, source) = match *g.e(atk) {
            Effect::Attack { p, opp, attack, source, .. } => (p, opp, attack, source),
            _ => return Ok(()),
        };
        let a = g.st.players[p as usize].active;
        let b = AtkBase { attack_effect: atk, player: p, opponent: opp, attack, source, target: SlotRef::new(p as usize, a) };
        g.run_fx(Effect::DiscardCards { b, cards })?;
        Ok(())
    })();
    g.release_fx(atk);
    r
}
