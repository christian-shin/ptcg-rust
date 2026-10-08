//! Mega Charizard Y ex (ASC / MC): Explosion Y — discard 3 Energy from this
//! Pokémon, and 280 damage to 1 of your opponent's Pokémon (the Bench takes no
//! Weakness/Resistance).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MegaCharizardYex",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::after_damage(Op::DamageSlot(DamageSlotSpec {
                target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::PokemonBenchFirst(Who::Opp), msg: "CHOOSE_POKEMON_TO_DAMAGE" }),
                hp: Num::Lit(280),
                target_damage_mul: 0,
                calc: DamageCalc::Auto,
                when: Cond::True,
            })),
            Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: MY_ACTIVE, selection: EnergySelection::Choose { count: 3, ty: crate::types::ct::COLORLESS } })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
