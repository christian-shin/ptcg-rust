//! N's Darmanitan (JTG): Back Draft — 30 damage for each Basic Energy card in
//! your opponent's discard pile. Flamebody Cannon — 90; discard all Energy
//! from this Pokémon; also 90 damage to 1 of your opponent's Benched Pokémon.
//!
//! Back Draft sets the main damage. Flamebody Cannon discards every Energy of the Active Pokémon, then (only when the
//! opponent has a Benched Pokémon) a mandatory pick and one Damage event on it (no Weakness or Resistance), with the
//! attack as its cause.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "NsDarmanitan",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Set, hp: Num::Mul(&Num::CardCount(ZoneRef(Who::Opp, Zone::Discard), Pred::BasicEnergy), &Num::Lit(30)), when: Cond::True })),
            ],
        },
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(MY_ACTIVE), selection: EnergySelection::AllProvided, ..DiscardEnergySpec::DEFAULT })),
                Step::after_damage(Op::DamageSlot(DamageSlotSpec { target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Bench(Who::Opp), msg: "CHOOSE_POKEMON_TO_DAMAGE" }), hp: Num::Lit(90), target_damage_mul: 0, calc: DamageCalc::Put, when: Cond::True })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
