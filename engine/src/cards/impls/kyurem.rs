//! Kyurem (SFA): Plasma Bane — if the opponent has a card with "Colress" in
//! its name in their discard pile, Trifrost costs only [C]. Trifrost —
//! discard all Energy from this Pokémon; 110 damage to 3 of the opponent's
//! Pokémon.
//!
//! Fixed (phase 4b, W4): the target prompt was min 1 max 3; it is now exactly
//! min(3, opponent's Pokémon in play).
//!
//! Fixed (phase 4b, R7F-11; ruling 1581): the cost [C] was an ordinary cost
//! that Pokémon League Headquarters, Rillaboom's Drum Beating, Antique Root
//! Fossil, Counter Gain, ... still changed; the Ability now sets it
//! (CheckAttackCostEffect.setCost), and a set cost is not increased or
//! decreased.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "Kyurem",
    mask: mask(&[k::CHECK_ATTACK_COST, k::ATTACK]),
    reduce,
    resume: Some(resume),
    coin: None,
    can_play: None,
};

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::CheckAttackCost { p, attack, .. } = *g.e(e) {
        if attack != my_attack(g, me, 0) {
            return Ok(());
        }
        let p = p as usize;
        let o = 1 - p;
        if is_ability_blocked(g, p, me, None) {
            return Ok(());
        }
        let colress = g.st.players[o].discard.iter().any(|c| {
            let d = g.st.cdef(c);
            d.is_trainer() && d.name.contains("Colress")
        });
        if colress {
            if let Effect::CheckAttackCost { cost, set_cost, .. } = g.e_mut(e) {
                cost.retain(|t| *t != ct::WATER && *t != ct::METAL);
                // "can use the Trifrost attack for [C]": a cost that is set is not
                // increased or decreased (R7F-11, ruling 1581).
                let mut c: crate::effects::Cost = SVec::new();
                c.push(ct::COLORLESS);
                *set_cost = Some(c);
            }
        }
        return Ok(());
    }

    if was_attack_used(g, e, 0, me) {
        let (p, opp, attack, source) = match *g.e(e) {
            Effect::Attack { p, opp, attack, source, .. } => (p, opp, attack, source),
            _ => return Ok(()),
        };
        let pu = p as usize;
        let active = SlotRef::new(pu, g.st.players[pu].active);
        let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p, source: active, energy_map: SVec::new() })?;
        let mut cards: SVec<CardId, 64> = SVec::new();
        if let Effect::CheckProvidedEnergy { energy_map, .. } = pe {
            for m in energy_map.iter() {
                cards.push(m.card);
            }
        }
        let b = AtkBase { attack_effect: e, player: p, opponent: opp, attack, source, target: active };
        g.run_fx(Effect::DiscardCards { b, cards })?;

        let mut slots = SVec::new();
        slots.push(SlotType::Active as u8);
        slots.push(SlotType::Bench as u8);
        let count = for_each_pokemon(g, opp as usize, PlayerType::BottomPlayer).len().min(3) as u8;
        g.retain_fx(e);
        let mut f = CardFrame::at(1);
        f.e[0] = e;
        let id = g.player_id(pu);
        g.prompt(
            id,
            "CHOOSE_POKEMON_TO_DAMAGE",
            PromptKind::ChoosePokemon { player_type: PlayerType::TopPlayer, slots, min: count, max: count, allow_cancel: false, blocked: SVec::new() },
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
    let targets: Vec<SlotRef> = results.first().map(|r| r.slots().to_vec()).unwrap_or_default();
    let r = damage_opponent_pokemon(g, atk, 110, &targets);
    g.release_fx(atk);
    r
}
