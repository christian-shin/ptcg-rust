//! Pikachu ex (SSP, Tera): Tenacious Heart — if this Pokémon has full HP and
//! would be Knocked Out by an attack, it isn't Knocked Out and its remaining
//! HP becomes 10 instead. Topaz Bolt — 300; discard 3 Energy from this
//! Pokémon. Tera: no attack damage while on the Bench.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Pikachuex@SSP|ASC",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(MY_ACTIVE), selection: EnergySelection::Choose { count: 3, ty: crate::types::ct::COLORLESS }, ..DiscardEnergySpec::DEFAULT }))],
    }],
    passives: &[
        Passive { origin: RuleSource::Ability, modifier: Modifier::SurviveOnTen(SurviveOnTenSpec { kind: SurviveKind::IfFullHp }) },
        // Tera: no attack damage while Benched.
        Passive { origin: RuleSource::CardRule, modifier: Modifier::Prevent(TERA_RULE) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
