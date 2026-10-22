//! Mesprit (SSP 79): Full Heart - attach up to 2 Basic [P] Energy cards from your hand to your Pokémon in any way you like.
//! Guardian Burst - 160; if you don't have Uxie and Azelf on your Bench, this attack does nothing.
//!
//! Full Heart: an Attach event per card from the hand (0 to 2, the Active Pokémon or the Bench; Chansey's Lucky Attachment form).
//! Guardian Burst sets the damage to 0 before the damage unless a Pokémon named Uxie and one named Azelf are both on the Bench
//! (an attack that does nothing is still used: APR D-01, id675).
use crate::spec::prelude::*;

const BASIC_P: Pred = Pred::All(&[Pred::BasicEnergy, Pred::Name("Psychic Energy")]);

pub static SPEC: CardSpec = CardSpec {
    class: "Mesprit",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[Step::after_damage(Op::If(IfSpec {
                cond: Cond::Nonempty(ZoneRef(Who::Me, Zone::Hand), BASIC_P),
                yes: &[Step::new(Op::Attach(AttachSpec {
                    from: ZoneRef(Who::Me, Zone::Hand),
                    predicate: BASIC_P,
                    slots: AttachSlots::ActiveBench,
                    bounds: Bounds { min: Num::Lit(0), max: Num::Lit(2) },
                    ..AttachSpec::DEFAULT
                }))],
                no: &[],
            }))],
        },
        AttackSpec {
            index: 1,
            steps: &[Step::before_damage(Op::Damage(DamageSpec {
                op: DamageOp::Set,
                hp: Num::Lit(0),
                when: Cond::Not(&Cond::All(&[Cond::InPlay(Who::Me, PlayScope::Bench, Pred::Name("Uxie")), Cond::InPlay(Who::Me, PlayScope::Bench, Pred::Name("Azelf"))])),
            }))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
