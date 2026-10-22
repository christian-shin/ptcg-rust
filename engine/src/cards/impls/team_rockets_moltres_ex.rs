//! Team Rocket's Moltres ex (DRI 31): Flame Screen - 110; during your opponent's next turn, this Pokémon takes 50 less damage from
//! attacks (after applying Weakness and Resistance). Evil Incineration - discard a Team Rocket's Energy from this Pokémon. If you
//! do, discard your opponent's Active Pokémon and all attached cards.
//!
//! Flame Screen arms the lasting damage reduction on this Pokémon (Clamperl DRI 54). Evil Incineration: with a Team Rocket's Energy
//! attached to this Pokémon (the card's own condition for "if you do": this Pokémon's own Energy can't be kept from leaving by an
//! effect on the opponent's side, and an attack's removal of Energy waits for its damage, so no later step reads its outcome,
//! `spec::run::deferred_tests`), one chosen Team Rocket's Energy card of this Pokémon is discarded (a LeavePlay caused by the attack)
//! and the Defending Pokémon and every card attached to it go to its owner's discard pile (a LeavePlay by the attack; not a Knock
//! Out, no Prize card is taken; Gholdengo's removal of its own Pokémon).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TeamRocketsMoltresEx",
    attacks: &[
        AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::TakesLessDamage(50, DamageSource::Any) }))] },
        AttackSpec {
            index: 1,
            steps: &[Step::after_damage(Op::If(IfSpec {
                cond: Cond::Slot(MY_ACTIVE, SlotPred::HasEnergyNamed("Team Rocket's Energy")),
                yes: &[
                    Step::new(Op::DiscardEnergy(DiscardEnergySpec {
                        target: SlotTarget::Slot(MY_ACTIVE),
                        selection: EnergySelection::Chosen { pred: Pred::Name("Team Rocket's Energy") },
                        ..DiscardEnergySpec::DEFAULT
                    })),
                    Step::new(Op::RemoveFromPlay(RemoveFromPlaySpec { slot: OPP_ACTIVE, destination: ZoneRef(Who::Opp, Zone::Discard) })),
                ],
                no: &[],
            }))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
