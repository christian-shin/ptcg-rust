//! Okidogi ex (SFA): Poisonous Musculature — search your deck for up to 2
//! Basic [D] Energy and attach them to this Pokémon, shuffle; if you attached
//! any, this Pokémon is now Poisoned. Chain-Crazed — 130+; 130 more if this
//! Pokémon is Poisoned.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Okidogiex",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[Step::after_damage(Op::If(IfSpec {
                cond: Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any),
                yes: &[
                    Step::new(Op::PickSlot(PickSlotSpec { chooser: Who::Me, among: SlotSel::One(SlotExpr::This), msg: "" })),
                    Step::new(Op::Pick(PickSpec {
                        from: ZoneRef(Who::Me, Zone::Deck),
                        predicate: Pred::All(&[Pred::BasicEnergy, Pred::Name("Darkness Energy")]),
                        bounds: Bounds { min: Num::Lit(0), max: Num::Lit(2) },
                        into: 0,
                        msg: "CHOOSE_CARD_TO_ATTACH",
                        ..PickSpec::DEFAULT
                    })),
                    Step::new(Op::If(IfSpec {
                        cond: Cond::Chosen(0),
                        yes: &[
                            Step::new(Op::Attach(AttachSpec { from: ZoneRef(Who::Me, Zone::Deck), cards: CardSel::Chosen(0), onto: Some(SlotExpr::Picked), ..AttachSpec::DEFAULT })),
                            Step::new(Op::Conditions(ConditionsSpec { target: MY_ACTIVE, change: ConditionChange::Add(&[SpecialCondition::Poisoned]), gate: Gate::None, when: Cond::True })),
                        ],
                        no: &[],
                    })),
                    Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
                ],
                no: &[],
            }))],
        },
        AttackSpec { index: 1, steps: &[Step::before_damage(more_damage_if(130, Cond::Slot(MY_ACTIVE, SlotPred::Condition(SpecialCondition::Poisoned))))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
