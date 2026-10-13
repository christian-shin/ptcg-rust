//! Dark Bell (M5 / PBL 75): both Active non-[D] Pokémon are now Confused.
//!
//! One Special Condition event per non-[D] Active Pokémon (type as the game checks it), yours first, caused by this Item. It
//! can be played when at least one of them would change (`WouldChangeConditions`, id290, id925).
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
                when: MY_NON_DARK,
            })),
            Step::new(Op::Conditions(ConditionsSpec {
                target: OPP_ACTIVE,
                change: ConditionChange::Add(&[SpecialCondition::Confused]),
                when: OPP_NON_DARK,
            })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
