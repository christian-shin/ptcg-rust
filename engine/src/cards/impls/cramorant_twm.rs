//! Cramorant (TWM 58): Water Gun - 20. Spit Shot - discard all Energy from this Pokémon. This attack does 120 damage to 1 of
//! your opponent's Pokémon. (Don't apply Weakness and Resistance for Benched Pokémon.)
//!
//! The Energy goes first (a LeavePlay per card, this attack the cause), then one Damage event on the Pokémon the player
//! chooses among all of the opponent's: the full calculation with Weakness and Resistance on the Active Pokémon, none on a
//! Benched one (`DamageCalc::Auto`; Zeraora DRI 78's Thunder Raid).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Cramorant",
    attacks: &[
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(MY_ACTIVE), selection: EnergySelection::AllProvided, ..DiscardEnergySpec::DEFAULT })),
                Step::after_damage(Op::DamageSlot(DamageSlotSpec {
                    target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Pokemon(Who::Opp) }),
                    hp: Num::Lit(120),
                    target_damage_mul: 0,
                    calc: DamageCalc::Auto,
                    when: Cond::True,
                })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
