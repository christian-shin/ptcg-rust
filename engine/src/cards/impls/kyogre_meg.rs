//! Kyogre (MEG 34): Riptide — 20×; this attack does 20 damage for each Basic [W] Energy card in your discard pile. Then, shuffle
//! those cards into your deck. Swirling Waves — 130; discard 2 Energy from this Pokémon.
//!
//! Riptide counts the Basic [W] Energy cards of the discard pile for the damage (a Special Energy card isn't a Basic Energy card),
//! then those cards (all of them, chosen at step D) are put into the deck by one PutIntoDeck event and the deck is shuffled once
//! (APR E-19), only when there were any. Swirling Waves discards 2 Energy cards after the damage.
use crate::spec::prelude::*;

const DISCARD: ZoneRef = ZoneRef(Who::Me, Zone::Discard);
const BASIC_WATER: Pred = Pred::All(&[Pred::BasicEnergy, Pred::Provides(ct::WATER)]);

pub static SPEC: CardSpec = CardSpec {
    class: "KyogreMEG",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(damage_is(Num::Mul(&Num::CardCount(DISCARD, BASIC_WATER), &Num::Lit(20)))),
                Step::after_damage(Op::Search(SearchSpec {
                    pick: PickSpec { from: DISCARD, predicate: BASIC_WATER, bounds: Bounds { min: Num::CardCount(DISCARD, BASIC_WATER), max: Num::CardCount(DISCARD, BASIC_WATER) }, ..PickSpec::DEFAULT },
                    destination: SearchDestination::Deck { reveal: false },
                    cancel: false,
                })),
                Step::after_damage(Op::If(IfSpec { cond: Cond::Chosen(0), yes: &[Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true }))], no: &[] })),
            ],
        },
        AttackSpec {
            index: 1,
            steps: &[Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(MY_ACTIVE), selection: EnergySelection::Choose { count: 2, ty: ct::COLORLESS, up_to: false }, ..DiscardEnergySpec::DEFAULT }))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
