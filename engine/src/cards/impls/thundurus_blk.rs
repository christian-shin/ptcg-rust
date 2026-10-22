//! Thundurus (BLK 33): Charge - search your deck for a Basic [L] Energy card and attach it to this Pokémon. Then, shuffle
//! your deck. Disaster Volt - 110; discard an Energy from this Pokémon.
//!
//! Charge: with a card in the deck, one search (it may find none) and one Attach onto this Pokémon, then the shuffle (APR
//! E-19; Cresselia PFL 39). Disaster Volt discards one Energy of your choice from this Pokémon after the damage (Charcadet
//! SSP 33).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Thundurus",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::If(IfSpec {
                    cond: Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any),
                    yes: &[
                        Step::new(Op::Pick(PickSpec {
                            from: ZoneRef(Who::Me, Zone::Deck),
                            predicate: Pred::All(&[Pred::BasicEnergy, Pred::Name("Lightning Energy")]),
                            bounds: Bounds { min: Num::Lit(0), max: Num::Lit(1) },
                            into: 0,
                            ..PickSpec::DEFAULT
                        })),
                        Step::new(Op::If(IfSpec {
                            cond: Cond::Chosen(0),
                            yes: &[Step::new(Op::Attach(AttachSpec { from: ZoneRef(Who::Me, Zone::Deck), cards: CardSel::Chosen(0), onto: Some(SlotExpr::This), ..AttachSpec::DEFAULT }))],
                            no: &[],
                        })),
                        Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
                    ],
                    no: &[],
                })),
            ],
        },
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(MY_ACTIVE), selection: EnergySelection::Choose { count: 1, ty: ct::COLORLESS, up_to: false }, ..DiscardEnergySpec::DEFAULT })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
