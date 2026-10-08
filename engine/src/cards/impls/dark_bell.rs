//! Dark Bell (M5 / PBL 75): both Active non-[D] Pokémon are now Confused.
//!
//! Twinleaf: the player's own Active first, then the opponent's Active unless
//! a TrainerTargetEffect on it is blocked; each uses a CheckPokemonTypeEffect
//! and ADD_CONFUSION_TO_PLAYER_ACTIVE (AddSpecialConditionsPowerEffect, which
//! also resets poison/burn/sleep values to the defaults). `canPlay` is UI only.
//! Phase 4b (R6): throws CANNOT_PLAY_THIS_CARD when neither Active Pokémon is a
//! non-[D] Pokémon (it used to be playable with no effect).
use crate::spec::prelude::*;

const MY_NON_DARK: Cond = Cond::Not(&Cond::Slot(MY_ACTIVE, SlotPred::TypeIs(ct::DARK)));
const OPP_NON_DARK: Cond = Cond::Not(&Cond::Slot(OPP_ACTIVE, SlotPred::TypeIs(ct::DARK)));

pub static SPEC: CardSpec = CardSpec {
    class: "DarkBell",
    // Both Active non-[D] Pokémon are now Confused (it can't be played when neither would change).
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[Cond::Any(&[
            Cond::All(&[MY_NON_DARK, Cond::WouldChangeConditions(MY_ACTIVE, &[SpecialCondition::Confused])]),
            Cond::All(&[OPP_NON_DARK, Cond::WouldChangeConditions(OPP_ACTIVE, &[SpecialCondition::Confused])]),
        ])],
        steps: &[
            Step::new(Op::Conditions(ConditionsSpec {
                target: MY_ACTIVE,
                change: ConditionChange::Add(&[SpecialCondition::Confused]),
                cause: Cause::Ability,
                gate: Gate::None,
                when: MY_NON_DARK,
            })),
            Step::new(Op::Conditions(ConditionsSpec {
                target: OPP_ACTIVE,
                change: ConditionChange::Add(&[SpecialCondition::Confused]),
                cause: Cause::Ability,
                gate: Gate::TrainerTarget,
                when: OPP_NON_DARK,
            })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
