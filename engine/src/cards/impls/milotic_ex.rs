//! Milotic ex (SSP): Sparkling Scales — prevent all damage and effects done
//! to this Pokémon by your opponent's Tera Pokémon's attacks. Hypno Splash —
//! 160; your opponent's Active Pokémon is now Asleep.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Miloticex",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(inflict(&[SpecialCondition::Asleep]))] }],
    passives: &[
        // Sparkling Scales: prevent all damage and effects done to this Pokémon by the opponent's Tera Pokémon's attacks (every
        // event they cause, the switches included: APR C-04 / C-05, id2025, id2155; the damage at step 6, APR C-16).
        Passive {
            origin: RuleSource::Ability,
            modifier: Modifier::Prevent(PreventSpec::on(
                SlotPred::All(&[SlotPred::Holder, SlotPred::IsThisPokemon]),
                EventPred::All(&[DAMAGE_OR_EFFECTS, EventPred::Cause(CausePred::All(&[CausePred::By(Who::Opp), CausePred::Kind(crate::cause::CauseKind::Attack), CausePred::Card(Pred::Tag(crate::types::tag::POKEMON_TERA))]))]),
            )),
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
