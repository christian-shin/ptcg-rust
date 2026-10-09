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
        // Spherical Shield: prevent all damage from and effects of the opponent's attacks done to your Benched Pokémon (every
        // event they cause, the switches included: APR C-04 / C-05, id2025, id2155).
        Passive {
            origin: RuleSource::Ability,
            modifier: Modifier::Prevent(PreventSpec::on(
                SlotPred::All(&[SlotPred::IsBench, SlotPred::OnMySide]),
                EventPred::All(&[DAMAGE_OR_EFFECTS, EventPred::Cause(CausePred::All(&[CausePred::By(Who::Opp), CausePred::Kind(crate::cause::CauseKind::Attack)]))]),
            )),
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
