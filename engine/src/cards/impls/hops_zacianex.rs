//! Hop's Zacian ex (JTG 111): Insta-Strike — 30, and 30 damage to 1 of your
//! opponent's Benched Pokémon. Brave Slash — 240; during your next turn this
//! Pokémon can't use Brave Slash.
//!
//! Twinleaf: no prompt without a Benched Pokémon; ChoosePokemonPrompt (bench,
//! no cancel) then a PutDamageEffect of 30. Brave Slash pushes its name onto
//! the Active's `cannotUseAttacksNextTurnPending` if missing.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "HopsZacianex",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::after_damage(Op::DamageSlot(DamageSlotSpec { target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Bench(Who::Opp), msg: "CHOOSE_POKEMON_TO_DAMAGE" }), hp: Num::Lit(30), target_damage_mul: 0, calc: DamageCalc::Put, when: Cond::True })),
        ] },
        AttackSpec { index: 1, steps: &[
            Step::after_damage(Op::Arm(ArmSpec { what: Lasting::CannotUseThisAttackNextTurn })),
        ] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
