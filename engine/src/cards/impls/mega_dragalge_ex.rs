//! Mega Dragalge ex (CRI 65): Corrosive Liquid - discard all Pokémon Tools and Special Energy from all of your opponent's
//! Pokémon. Pernicious Poison - your opponent's Active Pokémon is now Poisoned. During Pokémon Checkup, place 16 damage
//! counters on that Pokémon instead of 1.
//!
//! Corrosive Liquid is Megaton Blower's discard (each opponent's Pokémon's Special Energy and Tools leave play, LeavePlay
//! events caused by the attack; a protection is a Prevent read by each event) without the Stadium. Pernicious Poison makes
//! the Defending Pokémon Poisoned with 16 damage counters per Pokémon Checkup (`ConditionChange::Counters`; it keeps
//! placing 16 at every Checkup while it lasts, id2373; a new single-counter Poison replaces it, id1044).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MegaDragalgeEx",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[Step::after_damage(Op::ForEach(ForEachSpec {
                over: SlotSel::Pokemon(Who::Opp),
                body: &[
                    Step::new(Op::Snapshot(SnapshotSpec { zone: ZoneRef(Who::Me, Zone::Attached(SlotExpr::Picked)), predicate: Pred::All(&[Pred::Energy, Pred::Not(&Pred::BasicEnergy)]), into: 0 })),
                    Step::new(Op::Discard(DiscardSpec { from: ZoneRef(Who::Me, Zone::Attached(SlotExpr::Picked)), cards: CardSel::Chosen(0), ..DiscardSpec::DEFAULT })),
                    Step::new(Op::Discard(DiscardSpec { from: ZoneRef(Who::Opp, Zone::Tools(SlotExpr::Picked)), ..DiscardSpec::DEFAULT })),
                ],
            }))],
        },
        AttackSpec {
            index: 1,
            steps: &[Step::after_damage(Op::Conditions(ConditionsSpec { target: OPP_ACTIVE, change: ConditionChange::Counters(SpecialCondition::Poisoned, 16), when: Cond::True }))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
