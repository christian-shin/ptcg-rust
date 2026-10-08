//! Mega Hawlucha ex (M2a / ASC 116): Tenacious Body — if this Pokémon would
//! be Knocked Out by damage from an attack, flip a coin; heads, it survives
//! with 10 HP. Somersault Dive — 120+; 140 more if a Stadium is in play,
//! then discard that Stadium.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MegaHawluchaex",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::before_damage(more_damage_if(140, Cond::StadiumInPlay(Pred::Any))),
            // The Stadium is discarded after the damage.
            Step::after_damage(DISCARD_STADIUM),
        ],
    }],
    passives: &[Passive { origin: RuleSource::Ability, modifier: Modifier::SurviveOnTen(SurviveOnTenSpec { kind: SurviveKind::OnCoin }) }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
