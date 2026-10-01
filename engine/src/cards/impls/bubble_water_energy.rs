//! Bubbly Water Energy ("Bubble Water Energy M4", CRI): provides [W]. The
//! [W] Pokémon this card is attached to recovers from all Special Conditions
//! and can't be affected by any Special Conditions.
//!
//! Twinleaf quirks kept: no [W] type check anywhere (the CheckPokemonType
//! on attach is computed and ignored). On its AttachEnergyEffect (before the
//! card is attached) the target loses all five conditions unless the special
//! energy is blocked. PREVENT_AND_CLEAR_SPECIAL_CONDITIONS: attack and
//! power AddSpecialConditions effects on a slot holding this card are
//! prevented, and every CheckTableState clears the conditions of each slot
//! holding it (both unless blocked for the slot's owner).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "BubbleWaterEnergy",
    mask: mask(&[k::ATTACH_ENERGY, k::ADD_SPECIAL_CONDITIONS, k::ADD_SPECIAL_CONDITIONS_POWER, k::CHECK_TABLE_STATE]),
    reduce,
    resume: None,
    coin: None,
    can_play: None,
};

fn clear_all(g: &mut Game, t: SlotRef) {
    let sc = &mut g.st.players[t.p as usize].slots[t.s as usize].special_conditions;
    for c in [SpecialCondition::Poisoned, SpecialCondition::Asleep, SpecialCondition::Burned, SpecialCondition::Confused, SpecialCondition::Paralyzed] {
        sc.retain(|x| *x != c as u8);
    }
}

fn should_apply(g: &mut Game, me: CardId, t: SlotRef) -> bool {
    g.st.slot(t.p as usize, t.s).cards.contains(me) && !is_special_energy_blocked(g, t.p as usize, me, t, false)
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    match *g.e(e) {
        Effect::AttachEnergy { p, card, target } if card == me => {
            if is_special_energy_blocked(g, p as usize, me, target, false) {
                return Ok(());
            }
            let types = crate::engine::game_effect::pokemon_types(g, target);
            g.run_fx(Effect::CheckPokemonType { target, card_types: types })?;
            clear_all(g, target);
        }
        Effect::AddSpecialConditions { b, .. } => {
            if should_apply(g, me, b.target) {
                g.set_prevent(e, true);
            }
        }
        Effect::AddSpecialConditionsPower { target, .. } => {
            if should_apply(g, me, target) {
                g.set_prevent(e, true);
            }
        }
        Effect::CheckTableState { .. } => {
            for p in 0..2usize {
                for (s, _, _) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
                    let t = SlotRef::new(p, s);
                    if !g.st.slot(p, s).special_conditions.is_empty() && should_apply(g, me, t) {
                        clear_all(g, t);
                    }
                }
            }
        }
        _ => {}
    }
    Ok(())
}
