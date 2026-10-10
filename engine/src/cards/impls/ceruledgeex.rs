//! Ceruledge ex (SSP, Tera): Abyssal Flame — 30+; 20 more damage for each
//! Energy card in your discard pile. Amethyst Rage — 280; discard all Energy
//! from this Pokémon. Tera: no attack damage while on the Bench.
//!
//! Fixed (W1-B): Amethyst Rage used to push the Energy cards of the slot
//! straight onto the discard pile and rebuild `cards` without them, leaving
//! stale references in the slot's `energies`. It now uses the
//! DISCARD_ALL_ENERGY_FROM_POKEMON prefab (a DiscardCardsEffect).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Ceruledgeex",
    attacks: &[
        // Abyssal Flame: 20 more damage for each Energy card in your discard pile.
        AttackSpec {
            index: 0,
            steps: &[Step::before_damage(Op::Damage(DamageSpec {
                op: DamageOp::Add,
                hp: Num::Mul(&Num::CardCount(ZoneRef(Who::Me, Zone::Discard), Pred::Energy), &Num::Lit(20)),
                when: Cond::True,
            }))],
        },
        // Amethyst Rage: discard all Energy from this Pokémon.
        AttackSpec {
            index: 1,
            steps: &[Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(MY_ACTIVE), selection: EnergySelection::AllProvided, ..DiscardEnergySpec::DEFAULT }))],
        },
    ],
    // Tera: no attack damage while Benched.
    passives: &[Passive { origin: RuleSource::CardRule, modifier: Modifier::Prevent(TERA_RULE) }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
