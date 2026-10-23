//! Pyroar (MEG 24): Intimidating Fang - as long as this Pokémon is in the Active Spot, attacks used by your opponent's
//! Active Pokémon do 30 less damage (before applying Weakness and Resistance). Searing Flame - 70; your opponent's Active
//! Pokémon is now Burned.
//!
//! Intimidating Fang is a `DamageDealt` declaration at the Deal stage (step 2: effects on the Pokémon doing the damage,
//! before Weakness and Resistance) of -30 on every attack of the opponent's Active Pokémon, while this Pokémon is in the
//! Active Spot, to any Pokémon the attack damages: the Defending Pokémon, a Benched Pokémon, the attacker itself (id158,
//! id2213: it reduces the damage to each Pokémon). It is an effect on the attacking Pokémon, so Shred doesn't ignore it
//! (id2214), two copies add up (id2344), and Hide 'n' Sneak on the attacker keeps it off (JP FAQ Banette). Searing Flame
//! burns the Defending Pokémon after the damage.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "PyroarMEG",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(inflict(&[SpecialCondition::Burned]))] }],
    passives: &[Passive {
        origin: RuleSource::Ability,
        modifier: Modifier::DamageDealt(DamageDealtSpec {
            amount: -30,
            attacker: SlotPred::IsActive,
            side: Side::Opponent,
            opp_active_only: false,
            needs_damage: true,
            guard: Cond::IsActive(SlotExpr::This),
            ..DamageDealtSpec::DEFAULT
        }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
