//! Koraidon ex (ASC, Tera): Orichalcum Fang - 50+, 120 more if any of your Pokémon were Knocked Out by attack damage during
//! your opponent's last turn. Impact Blow - 200; can't use Impact Blow during your next turn.
//!
//! Orichalcum Fang reads `Cond::KnockedOutLastTurn` by attack damage. Impact Blow arms a lasting `CANT_ATTACK` lock on this
//! Pokémon naming this attack (`NamedAttack::This`, until your next turn): its other attack stays usable. The Tera rule is
//! `TERA_RULE` (a `Prevent` over damage while on the Bench). This port is the ASC card (the TEF Koraidon ex is a different card).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Koraidonex@ASC",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::before_damage(more_damage_if(120, Cond::KnockedOutLastTurn { who: Who::Me, by_attack_damage: true, tag: None })),
        ] },
        AttackSpec { index: 1, steps: &[
            Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_this_pokemon(&CANT_ATTACK, LockUntil::YourNextTurn).naming(NamedAttack::This)) })),
        ] },
    ],
    passives: &[
        Passive { origin: RuleSource::CardRule, modifier: Modifier::Prevent(TERA_RULE) }
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
