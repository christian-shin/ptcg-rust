//! Miraidon ex (ASC 73): Tera (as long as this Pokémon is on your Bench, prevent all damage done to it by attacks). Slashing Claw - 40. Hadron Spark - 120+; if your opponent's Active Pokémon is a Pokémon ex, this attack does 120 more damage.
//!
//! The Tera rule is `TERA_RULE` (Koraidon ex ASC); Hadron Spark's 120 more is read before the damage (Bouffalant ex WHT 77).
//! This is the ASC card; the TEF Miraidon ex is another card.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MiraidonExASC",
    attacks: &[
        AttackSpec {
            index: 1,
            steps: &[
                Step::before_damage(more_damage_if(120, Cond::Slot(OPP_ACTIVE, SlotPred::Top(Pred::PokemonEx)))),
            ],
        },
    ],
    passives: &[Passive { origin: RuleSource::CardRule, modifier: Modifier::Prevent(TERA_RULE) }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
