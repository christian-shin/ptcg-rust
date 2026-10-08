//! Zarude (SSP): Leaf Drain — 20, heal 20 damage from this Pokémon.
//! Jungle Whip — 80+; you may put all Energy attached to this Pokémon into
//! your hand for 80 more damage.
//!
//! Twinleaf: HealTargetEffect(20) on the Active; Jungle Whip is a
//! ConfirmPrompt (WANT_TO_USE_ABILITY) whose yes-callback reads
//! CheckProvidedEnergyEffect on the Active, MOVE_CARDS those cards to the
//! hand, then adds 80 to the attack's damage.
//! R7A (ruling 1846): the Energy goes into the hand after the damage (`move_cards_after_damage`).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Zarude@SSP",
    attacks: &[
        AttackSpec { index: 0, steps: &[Step::after_damage(heal_active(20, HealVia::Attack))] },
        AttackSpec {
            index: 1,
            // You may put all Energy attached to this Pokémon into your hand for 80 more damage.
            steps: &[Step::after_damage(Op::May(MaySpec {
                asker: Who::Me,
                when: Cond::True,
                msg: "WANT_TO_USE_ABILITY",
                yes: &[
                    Step::new(Op::EnergyChoice(EnergyChoiceSpec { how: EnergyHow::All { provided: true }, to: EnergyDest::Hand, ..EnergyChoiceSpec::DEFAULT })),
                    Step::new(Op::ChoiceDamage(ChoiceDamageSpec { reg: None, op: DamageOp::Add, per: 80 })),
                ],
                no: &[],
            }))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
