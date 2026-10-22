//! Sylveon (SFA 22): Mystical Return — flip a coin. If heads, choose 1 of your opponent's Benched Pokémon. Shuffle that Pokémon
//! and all attached cards into their deck. Disarming Voice — 90; your opponent's Active Pokémon is now Confused.
//!
//! Mystical Return: on heads the attacker chooses a Benched Pokémon and the whole stack leaves play to the opponent's deck (a
//! LeavePlay caused by the attack, which "prevent all effects of attacks" refuses), then the deck is shuffled (as Illumise
//! TWM 10). Disarming Voice is a Special Condition event on the Defending Pokémon.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "SylveonSFA",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[Step::after_damage(Op::Coin(CoinSpec {
                heads: &[
                    Step::new(Op::PickSlot(PickSlotOp { pick: PickSlotSpec { chooser: Who::Me, among: SlotSel::Bench(Who::Opp) }, purpose: PromptPurpose::PutIntoDeck })),
                    Step::new(Op::If(IfSpec {
                        cond: Cond::Slot(SlotExpr::Picked, SlotPred::Any),
                        yes: &[
                            Step::new(Op::RemoveFromPlay(RemoveFromPlaySpec { slot: SlotExpr::Picked, destination: ZoneRef(Who::Opp, Zone::Deck) })),
                            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Opp, Zone::Deck), wait: true })),
                        ],
                        no: &[],
                    })),
                ],
                ..CoinSpec::DEFAULT
            }))],
        },
        AttackSpec { index: 1, steps: &[Step::after_damage(inflict(&[SpecialCondition::Confused]))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
