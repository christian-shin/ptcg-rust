//! Milotic ex (SSP): Sparkling Scales — prevent all damage and effects done
//! to this Pokémon by your opponent's Tera Pokémon's attacks. Hypno Splash —
//! 160; your opponent's Active Pokémon is now Asleep.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Miloticex",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(inflict(&[SpecialCondition::Asleep]))] }],
    passives: &[
        // Sparkling Scales: every event the opponent's Tera Pokémon's attacks cause to this Pokémon, the switches included (APR
        // C-04 / C-05, id2025, id2155). B6-OLD -> C4: its damage half is the `PreventDamage` below.
        Passive {
            origin: RuleSource::Ability,
            modifier: Modifier::Prevent(PreventSpec::on(
                SlotPred::All(&[SlotPred::Holder, SlotPred::IsThisPokemon]),
                EventPred::Cause(CausePred::All(&[CausePred::By(Who::Opp), CausePred::Kind(crate::cause::CauseKind::Attack), CausePred::Card(Pred::Tag(crate::types::tag::POKEMON_TERA))])),
            )),
        },
        Passive {
            origin: RuleSource::Ability,
            modifier: Modifier::PreventDamage(PreventDamageSpec { how: PreventHow::Zero, subject: SlotPred::All(&[SlotPred::Holder, SlotPred::IsThisPokemon]), source: SlotPred::Tag(crate::types::tag::POKEMON_TERA), ..PreventDamageSpec::DEFAULT }),
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
