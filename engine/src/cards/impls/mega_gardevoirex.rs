//! Mega Gardevoir ex (M1S / ASC): Overflowing Wishes - for each of your
//! Benched Pokémon, search your deck for a Basic [P] Energy and attach it to
//! that Pokémon, then shuffle. Mega Symphonia - 50 damage for each [P] Energy
//! attached to all of your Pokémon.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MegaGardevoirex",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[Step::after_damage(Op::If(IfSpec {
                cond: Cond::All(&[Cond::AnySlot(SlotSel::Bench(Who::Me), SlotPred::Any), Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any)]),
                yes: &[
                    Step::new(Op::Attach(AttachSpec {
                        from: ZoneRef(Who::Me, Zone::Deck),
                        predicate: Pred::All(&[Pred::BasicEnergy, Pred::Name("Psychic Energy")]),
                        slots: AttachSlots::Bench,
                        bounds: Bounds { min: Num::Lit(0), max: Num::BenchCount(Who::Me) },
                        different_targets: true,
                        ..AttachSpec::DEFAULT
                    })),
                    Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
                ],
                no: &[],
            }))],
        },
        AttackSpec {
            index: 1,
            steps: &[Step::before_damage(damage_is(Num::Mul(&Num::EnergyOn(SlotSel::Pokemon(Who::Me), EnergyUnit::Provided(crate::types::ct::PSYCHIC)), &Num::Lit(50))))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
