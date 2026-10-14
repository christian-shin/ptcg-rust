//! Dialga (MEG 95): Beam - 30. Chrono Burst - 80+; you may shuffle all Energy attached to this Pokémon into your deck
//! and have this attack do 80 more damage.
//!
//! The choice is always asked: the 80 more damage doesn't depend on having Energy, and with none attached nothing moves
//! and the deck isn't shuffled (id2352: do as much as you can). Yes adds 80 before the damage and flags the attack (`AttackFlagKind::OptionChosen`); the Energy goes back into
//! the deck after the damage (id2031, id2385: the Energy is still attached when the damage is done): a LeavePlay per
//! card (D1), then the deck is shuffled.
use crate::spec::prelude::*;

const ATTACHED: ZoneRef = ZoneRef(Who::Me, Zone::Attached(MY_ACTIVE));

pub static SPEC: CardSpec = CardSpec {
    class: "Dialga",
    // Chrono Burst: you may shuffle all Energy attached to this Pokémon into your deck and have
    // this attack do 80 more damage (the Energy goes back into the deck after the damage).
    attacks: &[AttackSpec {
        index: 1,
        steps: &[
            Step::before_damage(Op::May(MaySpec {
                asker: Who::Me,
                when: Cond::True,
                yes: &[
                    Step::new(Op::AttackFlag(AttackFlagSpec { flag: AttackFlagKind::OptionChosen, value: true })),
                    Step::new(more_damage_if(80, Cond::True)),
                ],
                no: &[],
            })),
            Step::after_damage(Op::If(IfSpec {
                cond: Cond::AttackFlag(AttackFlagKind::OptionChosen),
                yes: &[
                    Step::new(Op::Snapshot(SnapshotSpec { zone: ATTACHED, predicate: Pred::Energy, into: 0 })),
                    Step::new(Op::PutIntoDeck(PutIntoDeckSpec { from: ATTACHED, cards: CardSel::Chosen(0), position: DeckPosition::Bottom, ..PutIntoDeckSpec::DEFAULT })),
                    Step::new(Op::If(IfSpec {
                        cond: Cond::Chosen(0),
                        yes: &[Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true }))],
                        no: &[],
                    })),
                ],
                no: &[],
            })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
