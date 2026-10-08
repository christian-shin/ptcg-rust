//! Lillie's Clefairy ex (JTG): Fairy Zone - the Weakness of each of your
//! opponent's [N] Pokémon in play is now [P]. Full Moon Rondo - 20 + 20 for
//! each Benched Pokémon (both players).
//!
//! Twinleaf probes the ability lock for the *target's* owner (not the
//! Clefairy's owner), before checking that this Clefairy is in play.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "LilliesClefairyex",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::Mul(&Num::Add(&Num::BenchCount(Who::Me), &Num::BenchCount(Who::Opp)), &Num::Lit(20)), when: Cond::True })),
        ] },
    ],
    passives: &[
        Passive { origin: RuleSource::Ability, modifier: Modifier::WeaknessOverride(WeaknessOverrideSpec { subject: SlotPred::PrintedTypeIs(ct::DRAGON), weakness: ct::PSYCHIC }) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
