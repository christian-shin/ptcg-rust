//! Antique Root Fossil (SCR): play this card as a 60 HP Basic [C] Pokémon.
//! It can't be affected by Special Conditions and can't retreat. At any time
//! during your turn, you may discard it from play.
//!
//! Primal Root: as long as it is Active, attacks used by your opponent's
//! Basic Pokémon cost [C] more.
//!
//! Primal Root is the card's printed Ability (printed powers: the fossil rule
//! and discard action, then Primal Root), so every Ability reader sees it:
//! Froslass's Freezing Shroud, `HasAbility`, the locks. The discard action
//! (power 0) stays always available and is not an Ability.
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
                // Conditions added directly (not through an effect) are cleared at the next check.
                sweep: true,
            }),
        },
        Passive { origin: RuleSource::CardRule, modifier: Modifier::BlockUse(BlockUseSpec::RETREAT_THIS_ACTIVE) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
