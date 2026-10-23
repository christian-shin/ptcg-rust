//! Revavroom ex (SFA 15): Tera - as long as this Pokémon is on your Bench, prevent all damage done to this Pokémon by
//! attacks (both yours and your opponent's). Accelerator Flash - 20+; if this Pokémon moved from your Bench to the Active
//! Spot this turn, this attack does 120 more damage. Shattering Speed - 250; discard this Pokémon and all attached cards.
//!
//! The Tera rule is `TERA_RULE`. Accelerator Flash adds 120 when this Pokémon moved from the Bench to the Active Spot this
//! turn (`Cond::ThisMovedToActive`, Mega Lopunny ex PFL 84). Shattering Speed removes this Pokémon and everything attached
//! to it from play to the discard pile after the damage (`Op::RemoveFromPlay`, a LeavePlay event; Emolga CRI 30's Sky
//! Return to the hand).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "RevavroomEx",
    attacks: &[
        AttackSpec { index: 0, steps: &[Step::before_damage(more_damage_if(120, Cond::ThisMovedToActive))] },
        AttackSpec { index: 1, steps: &[Step::after_damage(Op::RemoveFromPlay(RemoveFromPlaySpec { slot: MY_ACTIVE, destination: ZoneRef(Who::Me, Zone::Discard), top: false }))] },
    ],
    passives: &[Passive { origin: RuleSource::CardRule, modifier: Modifier::Prevent(TERA_RULE) }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
