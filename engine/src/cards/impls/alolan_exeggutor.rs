//! Alolan Exeggutor (30C 2): Scale Up — if this Pokémon has 6 or more [G]
//! Energy attached, it gets +250 HP. Mega Drain — 150; heal 50 damage from
//! this Pokémon.
//!
//! Twinleaf: on CheckHpEffect for the slot whose top Pokémon is this card,
//! unless the Ability is blocked, counts GRASS / ANY `provides` entries of a
//! CheckProvidedEnergyEffect on that slot.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "AlolanExeggutor",
    // Scale Up: with 6 or more [G] Energy attached, +250 HP.
    passives: &[Passive {
        origin: RuleSource::Ability,
        modifier: Modifier::HpMod(HpModSpec {
            amount: 250,
            subject: SlotPred::Holder,
            guard: Cond::Cmp(Num::EnergyOn(SlotSel::One(SlotExpr::This), EnergyUnit::Provided(ct::GRASS)), CmpOp::Ge, Num::Lit(6)),
        }),
    }],
    // Mega Drain: heal 50 damage from this Pokémon.
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(heal_active(50))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
