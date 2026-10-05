//! Ethan's Magcargo (DRI 36 / ASC 24): Melt and Flow — if this Pokémon has no
//! Energy attached, it has no Retreat Cost. Lava Burst — 70x; discard up to 5
//! [R] Energy from this Pokémon, 70 damage for each card discarded.
//!
//! Twinleaf: Melt and Flow reacts to CheckRetreatCostEffect when this card is
//! in its player's Active (generic Ability probe, then a CheckProvidedEnergy on
//! the Active: no entries -> `cost = []`). Lava Burst runs a CheckProvidedEnergy
//! (unused), a non-cancellable DiscardEnergyPrompt (Active, basic Energy named
//! 'Fire Energy', min 0, max 5), then one DiscardCardsEffect (possibly with
//! no cards) aimed at `player.active` and `damage = 70 * cards`.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "EthansMagcargo", mask: mask(&[k::CHECK_RETREAT_COST, k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::CheckRetreatCost { p, .. } = *g.e(e) {
        let p = p as usize;
        let a = g.st.players[p].active;
        if g.st.slot(p, a).cards.contains(me) && !is_ability_blocked(g, p, me, None) {
            let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p: p as u8, source: SlotRef::new(p, a), energy_map: SVec::new() })?;
            if matches!(pe, Effect::CheckProvidedEnergy { energy_map, .. } if energy_map.is_empty()) {
                if let Effect::CheckRetreatCost { cost, no_cost, .. } = g.e_mut(e) {
                    *cost = SVec::new();
                    *no_cost = true;
                }
            }
        }
    }
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let a = g.st.players[p].active;
        g.run_fx(Effect::CheckProvidedEnergy { p: p as u8, source: SlotRef::new(p, a), energy_map: SVec::new() })?;
        let mut slots = SVec::new();
        slots.push(SlotType::Active as u8);
        let filter = Filter { super_type: Some(SuperType::Energy as u8), energy_type: Some(EnergyType::Basic as u8), name: Some("Fire Energy"), ..Filter::none() };
        let o = MoveOpts { allow_cancel: false, min: 0, max: Some(5), ..Default::default() };
        g.retain_fx(e);
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        f.e[0] = e;
        let id = g.player_id(p);
        g.prompt(
            id,
            "CHOOSE_ENERGIES_TO_DISCARD",
            PromptKind::DiscardEnergy { player_type: PlayerType::BottomPlayer, slots, filter, o },
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
        let mut cards: SVec<CardId, 16> = SVec::new();
        if let Some(Res::CardsFrom(t)) = results.first().copied() {
            for (_, c) in t.iter() {
                cards.push(*c);
            }
        }
        let n = cards.len() as i32;
        if let Effect::Attack { p, opp, attack, source, .. } = *g.e(atk) {
            let pp = p as usize;
            let target = SlotRef::new(pp, g.st.players[pp].active);
            g.run_fx(Effect::DiscardCards { b: AtkBase { attack_effect: atk, player: p, opponent: opp, attack, source, target }, cards })?;
        }
        if let Effect::Attack { damage, .. } = g.e_mut(atk) {
            *damage = 70 * n;
        }
        Ok(())
    })();
    g.release_fx(atk);
    r
}
