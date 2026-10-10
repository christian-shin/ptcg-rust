//! Ceruledge ex (PRE 147 / SSP 36, Tera): Abyssal Flames — 30+; 20 more
//! damage for each Energy card in your discard pile. Raging Amethyst — 280;
//! discard all Energy from this Pokémon. Tera: as long as this Pokémon is on your
//! Bench, prevent all damage done to it by attacks.
//!
//! Rule: Raging Amethyst's discard is the attack's own effect after the damage
//! (Discard events of the Energy provided by this Pokémon, cause: this attack).
//! The Tera rule is `TERA_RULE`, a card rule (printed above the Ability line, so
//! Ability locks don't touch it): a `Prevent` over `Kind(Damage)` on this
//! Pokémon while Benched (step 6, APR C-16).
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
