//! Lillie's Clefairy ex (JTG 56 / ASC 76): Fairy Zone - the Weakness of each of your opponent's [N] Pokémon in play is now [P].
//! Full Moon Rondo - 20 + 20 for each Benched Pokémon (both players).
//!
//! Fairy Zone is a `Modifier::WeaknessOverride` read by the derived Weakness of each opposing Pokémon while the Ability
//! works (an Ability lock on this Pokémon's owner turns it off); the subject is the Pokémon's type as the game checks it
//! (`SlotPred::TypeIs`: a Pokémon's type can change, APR D-07; no pool card changes a type to or from [N]). The override is
//! only the Weakness of the Active Pokémon in the damage calculation, never of a Benched one (APR B-08; JP FAQ Lillie's
//! Clefairy ex).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "LilliesClefairyex",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::Mul(&Num::Add(&Num::BenchCount(Who::Me), &Num::BenchCount(Who::Opp)), &Num::Lit(20)), when: Cond::True })),
        ] },
    ],
    passives: &[
        Passive { origin: RuleSource::Ability, modifier: Modifier::WeaknessOverride(WeaknessOverrideSpec { subject: SlotPred::TypeIs(ct::DRAGON), weakness: ct::PSYCHIC }) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
