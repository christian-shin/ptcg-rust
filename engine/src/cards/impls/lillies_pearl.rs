//! Lillie's Pearl (JTG, tool): if the Lillie's Pokémon this card is attached
//! to is Knocked Out by damage from an attack from your opponent's Pokémon,
//! that player takes 1 fewer Prize card.
//!
//! Rule: a `PrizeAdjust` over the KnockOut view: it applies to a Knock Out whose
//! `ko_by` is AttackDamage (the damage of an opponent's attack; Poison, an effect
//! that Knocks Out, never), on a Pokémon with a Lillie's card in its stack.
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "LilliesPearl",
    passives: &[Passive {
        origin: RuleSource::Tool,
        // The Lillie's Pokémon Knocked Out by damage from an attack: the opponent takes 1 fewer Prize card.
        modifier: Modifier::PrizeAdjust(PrizeAdjustSpec {
            delta: -1,
            subject: SlotPred::All(&[SlotPred::Holder, SlotPred::AnyCardTag(tag::LILLIES)]),
            by_attack_damage: true,
            by_own_attack: None,
            guard: Cond::True,
            ..PrizeAdjustSpec::DEFAULT
        }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
