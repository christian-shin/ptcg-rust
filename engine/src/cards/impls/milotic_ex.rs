//! Milotic ex (SSP): Sparkling Scales — prevent all damage and effects done
//! to this Pokémon by your opponent's Tera Pokémon's attacks. Hypno Splash —
//! 160; your opponent's Active Pokémon is now Asleep.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Miloticex",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(inflict(&[SpecialCondition::Asleep]))] }],
    passives: &[
        // An opponent's attack switching this Pokémon, by the opponent's Tera Pokémon's attacks in or out (ChangeActive: APR C-04 / C-05, id2025, id2155).
        Passive { origin: RuleSource::Ability, modifier: Modifier::Prevent(PreventSpec::on(SlotPred::All(&[SlotPred::Holder, SlotPred::IsThisPokemon]), EventPred::All(&[EventPred::Kind(EventKind::ChangeActive), EventPred::Cause(CausePred::All(&[CausePred::By(Who::Opp), CausePred::Kind(crate::cause::CauseKind::Attack), CausePred::Card(Pred::Tag(crate::types::tag::POKEMON_TERA))]))]))) },
        Passive {
            origin: RuleSource::Ability,
            modifier: Modifier::PreventAttackEffects(PreventAttackEffectsSpec { subject: SlotPred::All(&[SlotPred::Holder, SlotPred::IsThisPokemon]), attacker: SlotPred::Tag(crate::types::tag::POKEMON_TERA), ..PreventAttackEffectsSpec::DEFAULT }),
        },
        Passive {
            origin: RuleSource::Ability,
            modifier: Modifier::PreventDamage(PreventDamageSpec { how: PreventHow::Zero, subject: SlotPred::All(&[SlotPred::Holder, SlotPred::IsThisPokemon]), source: SlotPred::Tag(crate::types::tag::POKEMON_TERA), ..PreventDamageSpec::DEFAULT }),
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
