//! Rotom (SSP 61): Crushing Pulse: your opponent reveals their hand. Discard all Item cards and Pokémon Tool cards you find
//! there. Energy Short: 20 damage for each Energy attached to your opponent's Active Pokémon.
//!
//! The opponent reveals their hand (a Reveal routine), then every Item and Pokémon Tool card in it is discarded (one Discard
//! event over their hand, this attack its cause).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Rotom",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::Reveal(RevealSpec { cards: RevealWhat::Zone(ZoneRef(Who::Opp, Zone::Hand)), by: Who::Opp, to: Who::Me, when_empty: true })),
                Step::after_damage(Op::Snapshot(SnapshotSpec { zone: ZoneRef(Who::Opp, Zone::Hand), predicate: Pred::OneOf(&[Pred::Item, Pred::Tool]), into: 0 })),
                Step::after_damage(Op::Discard(DiscardSpec { from: ZoneRef(Who::Opp, Zone::Hand), cards: CardSel::Chosen(0), ..DiscardSpec::DEFAULT })),
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
