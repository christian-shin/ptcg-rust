//! Zarude (SSP 11): Leaf Drain — 20; heal 20 damage from this Pokémon. Jungle Whip — 80+; you may put all Energy
//! attached to this Pokémon into your hand to have this attack do 80 more damage.
//!
//! Leaf Drain is a RemoveCounters (heal) event by the attack on Zarude. Jungle Whip: a yes / no choice; on yes the
//! Energy goes into the hand after the damage (id2385) and the damage is 80 more. The move to the hand is a raw move
//! until the PutIntoHand event (B7).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Zarude@SSP",
    attacks: &[
        AttackSpec { index: 0, steps: &[Step::after_damage(heal_active(20))] },
        AttackSpec {
            index: 1,
            // You may put all Energy attached to this Pokémon into your hand for 80 more damage.
            steps: &[Step::after_damage(Op::May(MaySpec {
                asker: Who::Me,
                when: Cond::True,
                msg: "WANT_TO_USE_ABILITY",
                yes: &[
                    Step::new(Op::DiscardEnergy(DiscardEnergySpec { selection: EnergySelection::All { provided: true }, to: EnergyDest::Hand, ..DiscardEnergySpec::DEFAULT })),
                    Step::new(Op::ChoiceDamage(ChoiceDamageSpec { reg: None, op: DamageOp::Add, per: 80 })),
                ],
                no: &[],
            }))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
