//! Mega Audino ex (MC): Kaleidowaltz — flip 3 coins; for each heads, search
//! your deck for up to 2 Basic Energy cards and attach them to your Pokémon
//! in any way you like, then shuffle. Ear Force — 20+, 80 more for each
//! Energy attached to your opponent's Active Pokémon.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MegaAudinoex",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[Step::after_damage(Op::Coin(CoinSpec {
                flips: Flips::Count(3),
                then: &[
                    Step::new(Op::If(IfSpec {
                        cond: Cond::Cmp(Num::Heads, CmpOp::Gt, Num::Lit(0)),
                        yes: &[Step::new(Op::Attach(AttachSpec {
                            from: ZoneRef(Who::Me, Zone::Deck),
                            predicate: Pred::BasicEnergy,
                            slots: AttachSlots::ActiveBench,
                            bounds: Bounds { min: Num::Lit(0), max: Num::Mul(&Num::Heads, &Num::Lit(2)) },
                            ..AttachSpec::DEFAULT
                        }))],
                        no: &[],
                    })),
                    Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
                ],
                ..CoinSpec::DEFAULT
            }))],
        },
        AttackSpec {
            index: 1,
            steps: &[Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::Mul(&Num::EnergyOn(SlotSel::One(OPP_ACTIVE), EnergyUnit::ProvidedUnits), &Num::Lit(80)), when: Cond::True }))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
