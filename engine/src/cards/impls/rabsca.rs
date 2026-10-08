//! Rabsca (TEF): Spherical Shield — prevent all damage from and effects of
//! attacks done to your Benched Pokémon by your opponent's attacks.
//! Psychic — 10+; 30 more for each Energy attached to the opponent's Active.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Rabsca",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::Mul(&Num::EnergyOn(SlotSel::One(OPP_ACTIVE), EnergyUnit::ProvidedUnits), &Num::Lit(30)), when: Cond::True }))],
    }],
    passives: &[
        Passive {
            origin: RuleSource::Ability,
            modifier: Modifier::PreventDamage(PreventDamageSpec { subject: SlotPred::IsBench, side: Side::Owner, ..PreventDamageSpec::DEFAULT }),
        },
        Passive {
            origin: RuleSource::Ability,
            modifier: Modifier::PreventAttackEffects(PreventAttackEffectsSpec { subject: SlotPred::IsBench, side: Side::Owner, needs_source_pokemon: false, ..PreventAttackEffectsSpec::DEFAULT }),
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
