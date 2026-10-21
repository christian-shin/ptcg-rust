//! Deoxys (CRI 31): Genome Charge - search your deck for up to 2 Basic [P] Energy cards and attach them to this Pokémon.
//! Then, shuffle your deck. Psychic - 80+; this attack does 20 more damage for each Energy attached to your opponent's
//! Active Pokémon.
//!
//! Genome Charge is the search-and-attach of Okidogi ex SFA 36's Poisonous Musculature without the Poison; Psychic counts the Energy units the Defending
//! Pokémon provides before the damage (Rabsca TEF 24).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "DeoxysCRI31",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::If(IfSpec {
            cond: Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any),
            yes: &[
                Step::new(Op::Pick(PickSpec {
                    from: ZoneRef(Who::Me, Zone::Deck),
                    predicate: Pred::All(&[Pred::BasicEnergy, Pred::Name("Psychic Energy")]),
                    bounds: Bounds { min: Num::Lit(0), max: Num::Lit(2) },
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
                Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::Mul(&Num::EnergyOn(SlotSel::One(OPP_ACTIVE), EnergyUnit::ProvidedUnits), &Num::Lit(20)), when: Cond::True })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
