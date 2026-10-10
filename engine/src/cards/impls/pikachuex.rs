//! Pikachu ex (SSP 57 / ASC 57, Tera): Resolute Heart — if this Pokémon has full HP
//! and would be Knocked Out by damage from an attack, it isn't Knocked Out and its
//! remaining HP becomes 10 instead. Topaz Bolt — 300; discard 3 Energy from this
//! Pokémon. Tera: as long as this Pokémon is on your Bench, prevent all damage done
//! to it by attacks.
//!
//! Rule: `SurviveOnTen(IfFullHp)` is read by the Damage event's calculation
//! (`damage::survive_on_10`): with no damage counters on it (the HP at that moment,
//! Lively Stadium included, id2041) and damage reaching its HP, it takes HP - 10. It is
//! an Ability. The Tera rule is `TERA_RULE`, a card rule.
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
