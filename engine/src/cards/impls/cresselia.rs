//! Cresselia (SFA): Healing Pirouette — heal 20 damage from each of your
//! Pokémon. Crescent Purge — 80+; you may turn 1 of your face-down Prize
//! cards face up for 80 more damage.
//!
//! Fixed (phase 4b): the face-down test counts only non-empty Prize lists that
//! are still face down, and the ChoosePrizePrompt (count 1) is `faceDownOnly`
//! and no longer cancellable (declining is the ConfirmPrompt's job), so a
//! face-up Prize can't be picked (it used to throw CANNOT_USE_POWER) and
//! cancelling can't crash (`chosenPrize[0]` on null). The chosen list gets
//! isSecret = false, faceUpPrize = true.
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
                msg: "WANT_TO_USE_ABILITY",
                yes: &[
                    Step::new(Op::PickPrize(PickPrizeSpec { chooser: Who::Me, face_down_only: true })),
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
