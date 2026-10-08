//! Dolliv (DRI): Nutrients — heal 40 damage from 1 of your Pokémon.
//! Tackle — 40.
//!
//! Twinleaf: a non-cancellable ChoosePokemonPrompt over your Active and
//! Bench (undamaged Pokémon selectable, message CHOOSE_POKEMON_TO_DAMAGE),
//! then a HealTargetEffect(effect, 40) on the choice.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Dolliv@DRI",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::Heal(HealSpec { target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Pokemon(Who::Me), msg: "CHOOSE_POKEMON_TO_DAMAGE" }), hp: Num::Lit(40), via: HealVia::Attack, clear_conditions: false })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
