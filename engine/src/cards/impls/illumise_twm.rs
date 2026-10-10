//! Illumise (TWM): Slowing Perfume — only if you go second, during your
//! first turn: shuffle 1 of your opponent's Benched Pokémon and all attached
//! cards into their deck. Glide — 30.
//!
//! Legal only on turn 2 (`Op::Fail`, CANNOT_USE_ATTACK otherwise). The attacker chooses 1 Benched Pokémon at step D and
//! nothing happens with an empty opposing Bench. The shuffle is `Op::RemoveFromPlay` of the chosen Pokémon (the whole
//! stack): one LeavePlay event with the attack as its cause, which "prevent all effects of attacks" (Mist Energy, Hide
//! 'n' Sneak) refuses, the Pokémon staying; then the opponent's deck is shuffled.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Illumise@TWM",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::before_damage(Op::Fail(FailSpec { unless: Cond::Cmp(Num::Turn, CmpOp::Eq, Num::Lit(2)), error: "CANNOT_USE_ATTACK" })),
            // The attacker picks at step D; nothing happens when the opponent has no Benched Pokémon.
            Step::after_damage(Op::PickSlot(PickSlotSpec { chooser: Who::Me, among: SlotSel::Bench(Who::Opp), msg: "CHOOSE_POKEMON_TO_SHUFFLE" })),
            Step::after_damage(Op::If(IfSpec {
                cond: Cond::Slot(SlotExpr::Picked, SlotPred::Any),
                yes: &[
                    Step::new(Op::RemoveFromPlay(RemoveFromPlaySpec { slot: SlotExpr::Picked, destination: ZoneRef(Who::Opp, Zone::Deck) })),
                    Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Opp, Zone::Deck), wait: true })),
                ],
                no: &[],
            })),
        ] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
