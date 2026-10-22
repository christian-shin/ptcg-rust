//! Registeel ex (ASC 145): Regi Charge - attach up to 2 Basic [M] Energy cards from your discard pile to this Pokémon. Protecting
//! Steel - 140; during your opponent's next turn, this Pokémon takes 50 less damage from attacks (after applying Weakness and
//! Resistance).
//!
//! Regi Charge: the cards are chosen in the discard pile (a public zone, 0 to 2: an attack's "up to", id2301) and each is an Attach event
//! onto this Pokémon (Team Rocket's Spidops). Protecting Steel is the lasting "takes N less damage" (Seedot TEF).
use crate::spec::prelude::*;

const DISCARD: ZoneRef = ZoneRef(Who::Me, Zone::Discard);

pub static SPEC: CardSpec = CardSpec {
    class: "RegisteelEx",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[Step::after_damage(Op::If(IfSpec {
                cond: Cond::Nonempty(DISCARD, Pred::All(&[Pred::BasicEnergy, Pred::Name("Metal Energy")])),
                yes: &[
                    Step::new(Op::Pick(PickSpec {
                        from: DISCARD,
                        predicate: Pred::All(&[Pred::BasicEnergy, Pred::Name("Metal Energy")]),
                        bounds: Bounds { min: Num::Lit(0), max: Num::Lit(2) },
                        into: 0,
                        ..PickSpec::DEFAULT
                    })),
                    Step::new(Op::If(IfSpec {
                        cond: Cond::Chosen(0),
                        yes: &[Step::new(Op::Attach(AttachSpec { from: DISCARD, cards: CardSel::Chosen(0), onto: Some(SlotExpr::This), ..AttachSpec::DEFAULT }))],
                        no: &[],
                    })),
                ],
                no: &[],
            }))],
        },
        AttackSpec { index: 1, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::TakesLessDamage(50, DamageSource::Any) }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
