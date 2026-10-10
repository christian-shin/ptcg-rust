//! Unown (30C): Mysterious Signal — 40; if your opponent's Pokémon is Knocked Out by damage from this attack, take 1 more
//! Prize card.
//!
//! `Modifier::PrizeAdjust` over the KnockOut event: `ko_by` = AttackDamage and the attack that did the damage is
//! Mysterious Signal of this Unown's player (`by_own_attack`). Any of the opponent's Pokémon counts, Active or Benched.
//! Damage prevented to 0, or counters instead of damage, don't count as "Knocked Out by damage".
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Unown@30C",
    passives: &[Passive {
        origin: RuleSource::CardRule,
        // Mysterious Signal: if the opponent's Pokémon is Knocked Out by damage from this attack, take 1 more Prize card.
        modifier: Modifier::PrizeAdjust(PrizeAdjustSpec { delta: 1, subject: SlotPred::Any, by_attack_damage: true, by_own_attack: Some("Mysterious Signal"), guard: Cond::True, ..PrizeAdjustSpec::DEFAULT }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
