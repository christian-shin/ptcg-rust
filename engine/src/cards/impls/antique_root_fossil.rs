//! Antique Root Fossil (SCR): play this card as a 60 HP Basic [C] Pokémon.
//! It can't be affected by Special Conditions and can't retreat. At any time
//! during your turn, you may discard it from play.
//!
//! Primal Root: as long as it is Active, attacks used by your opponent's
//! Basic Pokémon cost [C] more.
//!
//! Fixed (phase 4b, W4): Twinleaf did not implement Primal Root at all
//! (CheckAttackCostEffect of an attacker whose Active is Basic, with this card
//! the opponent's Active top card and the ability not blocked: +[C]). The
//! printed `powers` list still holds only the discard power.
//!
//! Twinleaf: on its own PlayItemEffect the card reduces a PlayPokemonEffect
//! into the first empty Bench slot (the item play then continues and finds
//! the card no longer in hand). In play, its Trainer Ability (a regular
//! UsePowerEffect path) MOVE_CARDS it to the discard; a RetreatEffect with it
//! Active throws; AddSpecialConditionsEffects on it are prevented.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "AntiqueRootFossil",
    // Played from the hand as a 60 HP Basic Pokémon.
    play: Some(PlaySpec { kind: PlayKind::Item, needs: &[], steps: &[Step::new(Op::PlayAsPokemon(PlayAsPokemonSpec {}))] }),
    // At any time during your turn, you may discard it from play.
    powers: &[PowerSpec {
        index: 0,
        once: Once::No,
        needs: &[],
        steps: &[Step::new(Op::RemoveFromPlay(RemoveFromPlaySpec { slot: SlotExpr::This, destination: ZoneRef(Who::Me, Zone::Discard), effect_of_attack: false }))],
    }],
    passives: &[
        // Primal Root: as long as it is Active, attacks used by your opponent's Basic Pokémon cost [C] more.
        Passive {
            origin: RuleSource::Ability,
            modifier: Modifier::AttackCost(AttackCostSpec {
                change: CostChange::Add(1),
                subject: SlotPred::Basic,
                side: Side::Opponent,
                guard: Cond::Slot(SlotExpr::Active(Who::Me), SlotPred::IsThisPokemon),
                ..AttackCostSpec::DEFAULT
            }),
        },
        // It can't be affected by Special Conditions and can't retreat.
        Passive {
            origin: RuleSource::CardRule,
            modifier: Modifier::ConditionImmunity(ConditionImmunitySpec {
                conds: &[],
                subject: SlotPred::All(&[SlotPred::Holder, SlotPred::IsThisPokemon]),
                prevent: true,
                sweep: false,
            }),
        },
        Passive { origin: RuleSource::CardRule, modifier: Modifier::BlockUse(BlockUseSpec { what: BlockWhat::RetreatThisActive }) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
