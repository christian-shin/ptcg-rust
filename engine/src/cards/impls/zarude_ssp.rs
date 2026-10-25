//! Zarude (SSP 11): Leaf Drain — 20; heal 20 damage from this Pokémon. Jungle Whip — 80+; you may put all Energy attached to this
//! Pokémon into your hand to have this attack do 80 more damage.
//!
//! Leaf Drain is a RemoveCounters (heal) event by the attack on Zarude. Jungle Whip: a yes / no choice; on yes the Energy goes into the
//! hand after the damage (a LeavePlay of each attached card, to the hand; id2385) and the damage is 80 more.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Zarude",
    attacks: &[
        AttackSpec { index: 0, steps: &[Step::after_damage(heal_active(20))] },
        AttackSpec {
            index: 1,
            // You may put all Energy attached to this Pokémon into your hand for 80 more damage.
            steps: &[Step::after_damage(Op::May(MaySpec {
                asker: Who::Me,
                when: Cond::True,
                yes: &[
                    Step::new(Op::DiscardEnergy(DiscardEnergySpec { selection: EnergySelection::All { provided: true, kind: EnergyKind::Any }, to: EnergyDest::Hand, ..DiscardEnergySpec::DEFAULT })),
                    Step::new(Op::ChoiceDamage(ChoiceDamageSpec { reg: None, by: ChoiceDamageBy::Each(DamageOp::Add, 80) })),
                ],
                no: &[],
            }))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
