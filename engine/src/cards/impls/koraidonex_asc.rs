//! Koraidon ex (ASC, Tera): Orichalcum Fang - 50+, 120 more if any of your
//! Pokémon were Knocked Out by attack damage during your opponent's last
//! turn. Impact Blow - 200; can't use Impact Blow during your next turn.
//!
//! Twinleaf has two unrelated `Koraidonex` classes (ASC and TEF); this port
//! is registered for the ASC one only.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Koraidonex@ASC",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::before_damage(more_damage_if(120, Cond::KnockedOutLastTurn { who: Who::Me, by_attack: true, tag: None })),
        ] },
        AttackSpec { index: 1, steps: &[
            Step::after_damage(Op::Arm(ArmSpec { what: Lasting::CannotUseThisAttackNextTurn })),
        ] },
    ],
    passives: &[
        Passive { origin: RuleSource::CardRule, modifier: Modifier::PreventDamage(PreventDamageSpec { how: PreventHow::Tera, ..PreventDamageSpec::DEFAULT }) }
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
