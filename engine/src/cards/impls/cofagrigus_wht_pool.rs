//! Cofagrigus (WHT 40): Extended Damagriiigus — move all damage counters from
//! 1 of your Benched Pokémon to 1 of your opponent's Pokémon. Perplex — 60;
//! your opponent's Active Pokémon is now Confused.
//!
//! Extended Damagriiigus does nothing without a damaged Benched Pokémon; otherwise it chooses one (non-cancellable),
//! then one of the opponent's Pokémon. The move is one MoveCounters event with the attack as its cause (APR C-08): as many
//! counters as the source has (id63) leave it and reach the destination unless it is protected (a Mist Energy
//! destination or Battle Cage on a Benched one makes them vanish, id2257, JP FAQ Battle Cage), and Patrat's Watchful Eye
//! stops the move with the counters staying (id2350).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "CofagrigusWHTPool",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::MoveCounters(MoveCountersSpec { kind: MoveCountersKind::AllFromOne { from: PickSlotSpec { chooser: Who::Me, among: SlotSel::Filtered(&SlotSel::Bench(Who::Me), SlotPred::Damaged), msg: "CHOOSE_POKEMON" }, to: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Pokemon(Who::Opp), msg: "CHOOSE_POKEMON_TO_DAMAGE" }) } })),
            ],
        },
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(inflict(&[SpecialCondition::Confused])),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
