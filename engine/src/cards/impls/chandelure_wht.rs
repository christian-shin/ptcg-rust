//! Chandelure (WHT 18): Incendiary Pillar — 50+; if you have 10 or more Basic [R] Energy cards in your discard pile, this attack
//! does 100 more damage. Burn It All Up — 180; discard all Energy from this Pokémon.
//!
//! Incendiary Pillar counts the Basic [R] Energy cards of your discard pile (a Special Energy card isn't a Basic Energy card,
//! JP FAQ Chandelure TWM 38). Burn It All Up discards every Energy card of this Pokémon after the damage.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "ChandelureWHT",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[Step::before_damage(Op::Damage(DamageSpec {
                op: DamageOp::Add,
                hp: Num::Lit(100),
                when: Cond::Cmp(Num::CardCount(ZoneRef(Who::Me, Zone::Discard), Pred::All(&[Pred::BasicEnergy, Pred::Provides(ct::FIRE)])), CmpOp::Ge, Num::Lit(10)),
            }))],
        },
        AttackSpec {
            index: 1,
            steps: &[Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(SlotExpr::This), selection: EnergySelection::AllProvided, ..DiscardEnergySpec::DEFAULT }))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
