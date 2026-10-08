//! Rotom (SSP): Crushing Pulse — your opponent reveals their hand; discard
//! all Item and Pokémon Tool cards you find there. Energy Short — 20× the
//! Energy attached to your opponent's Active Pokémon.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Rotom@SSP",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::Reveal(RevealSpec { cards: RevealWhat::Zone(ZoneRef(Who::Opp, Zone::Hand)), to: Who::Me, when_empty: true })),
                Step::after_damage(Op::Snapshot(SnapshotSpec { zone: ZoneRef(Who::Opp, Zone::Hand), predicate: Pred::OneOf(&[Pred::Item, Pred::Tool]), into: 0 })),
                Step::after_damage(Op::Move(MoveSpec { from: ZoneRef(Who::Opp, Zone::Hand), to: ZoneRef(Who::Opp, Zone::Discard), cards: CardSel::Chosen(0), ..MoveSpec::DEFAULT })),
            ],
        },
        AttackSpec {
            index: 1,
            steps: &[Step::before_damage(damage_is(Num::Mul(&Num::EnergyOn(SlotSel::One(OPP_ACTIVE), EnergyUnit::ProvidedUnits), &Num::Lit(20))))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
