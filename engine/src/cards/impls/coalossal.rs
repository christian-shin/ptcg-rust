//! Coalossal (ASC 120): Tar Cannon - this attack does 140 damage to 1 of your opponent's Pokémon. If you don't have 10 or more Basic
//! [F] Energy cards in your discard pile, this attack does nothing. (Don't apply Weakness and Resistance for Benched Pokémon.)
//! Bulky Bump - 220; discard 3 Energy from this Pokémon.
//!
//! Tar Cannon chooses one of the opponent's Pokémon and does 140 to it after the (zero) attack damage, with Weakness and
//! Resistance only for the Active Pokémon (APR B-08), only when the discard pile holds 10 Basic [F] Energy cards (the attack is still
//! used otherwise, APR D-01, id675). Bulky Bump discards 3 Energy of any type from this Pokémon after the damage.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Coalossal",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[Step::after_damage(Op::DamageSlot(DamageSlotSpec {
                target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Pokemon(Who::Opp) }),
                hp: Num::Lit(140),
                target_damage_mul: 0,
                calc: DamageCalc::Auto,
                when: Cond::Cmp(Num::CardCount(ZoneRef(Who::Me, Zone::Discard), Pred::All(&[Pred::BasicEnergy, Pred::Name("Fighting Energy")])), CmpOp::Ge, Num::Lit(10)),
            }))],
        },
        AttackSpec {
            index: 1,
            steps: &[Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(MY_ACTIVE), selection: EnergySelection::Choose { count: 3, ty: ct::COLORLESS, up_to: false }, ..DiscardEnergySpec::DEFAULT }))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
