//! Mega Gardevoir ex (MEG 60): Overflowing Wishes: for each of your Benched Pokémon, search your deck for a Basic [P]
//! Energy card and attach it to that Pokémon. Then, shuffle your deck. Mega Symphonia: 50 damage for each [P] Energy
//! attached to all of your Pokémon.
//!
//! One Basic [P] Energy per Benched Pokémon (one Attach event each, this attack its cause), then the deck is shuffled
//! (APR E-19; an empty deck can't be searched, id840).
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
