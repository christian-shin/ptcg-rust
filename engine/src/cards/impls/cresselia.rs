//! Cresselia (SFA): Healing Pirouette — heal 20 damage from each of your Pokémon. Crescent Purge — 80+; you may turn 1 of your
//! face-down Prize cards face up for 80 more damage.
//!
//! Crescent Purge asks (a "you may") only while a Prize card is face down; the chosen face-down Prize is turned face up for
//! good (JP FAQ Cresselia SFA 21) and the bonus is added before the damage.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Cresselia",
    attacks: &[
        // Healing Pirouette: heal 20 damage from each of your Pokémon.
        AttackSpec {
            index: 0,
            steps: &[Step::after_damage(Op::ForEach(ForEachSpec {
                over: SlotSel::Pokemon(Who::Me),
                body: &[Step::new(Op::Heal(HealSpec { target: SlotTarget::Slot(SlotExpr::Picked), hp: Num::Lit(20), clear_conditions: false }))],
            }))],
        },
        // Crescent Purge: you may turn 1 of your face-down Prize cards face up for 80 more damage.
        AttackSpec {
            index: 1,
            steps: &[Step::before_damage(Op::May(MaySpec {
                asker: Who::Me,
                when: Cond::FaceDownPrize(Who::Me),
                yes: &[
                    Step::new(Op::PickPrize(PickPrizeSpec { chooser: Who::Me, whose: Who::Me, face_down_only: true, purpose: PromptPurpose::TurnPrizeFaceUp })),
                    Step::new(Op::PrizeVisibility(PrizeVisibilitySpec { action: PrizeAction::FaceUp(Who::Me) })),
                    Step::new(more_damage_if(80, Cond::True)),
                ],
                no: &[],
            }))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
