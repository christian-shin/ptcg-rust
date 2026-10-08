//! Hearthflame Mask Ogerpon ex (TWM / PRE, Tera): Wrathful Hearth - 20 damage
//! for each damage counter on this Pokémon. Dynamic Blaze - 140+; if the
//! opposing Active isn't a Basic, 140 more and discard all Energy from this
//! Pokémon. Tera: no attack damage while on the Bench.
//!
//! Twinleaf: Wrathful Hearth is `damage = player.active.damage * 2`; Dynamic
//! Blaze adds 140 when `!opponent.active.isStage(BASIC)` (true as well for
//! an empty Active slot), then discards every card of the Active's
//! CheckProvidedEnergyEffect in one DiscardCardsEffect. The Tera rule is
//! TERA_RULE spelled out (see `tera_rule`).
//!
//! Fixed (phase 4b, R2): the discard of all Energy ran even when the
//! opponent's Active was a Basic Pokémon; "If your opponent's Active Pokémon
//! is an Evolution Pokémon, this attack does 140 more damage, and discard all
//! Energy from this Pokémon" makes both parts depend on the condition.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "HearthflameMaskOgerponex",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::before_damage(damage_is(Num::Mul(&Num::DamageOn(SlotExpr::This), &Num::Lit(2)))),
        ] },
        AttackSpec { index: 1, steps: &[
            Step::before_damage(more_damage_if(140, Cond::Not(&Cond::Slot(SlotExpr::Active(Who::Opp), SlotPred::Basic)))),
            Step::after_damage(Op::If(IfSpec { cond: Cond::Not(&Cond::Slot(SlotExpr::Active(Who::Opp), SlotPred::Basic)), yes: &[Step::new(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(SlotExpr::Active(Who::Me)), selection: EnergySelection::AllProvided, ..DiscardEnergySpec::DEFAULT }))], no: &[] })),
        ] },
    ],
    passives: &[
        Passive { origin: RuleSource::CardRule, modifier: Modifier::PreventDamage(PreventDamageSpec { how: PreventHow::Tera, ..PreventDamageSpec::DEFAULT }) }
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
