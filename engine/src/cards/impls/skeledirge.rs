//! Skeledirge (SSP): Unaware — prevent all effects of attacks used by your
//! opponent's Pokémon done to this Pokémon (damage is not an effect).
//! Torcherto — 60+, 20 more damage for each Benched Pokémon (both sides).
//!
//! Unaware is one `Prevent` over `EFFECTS_OF_OPP_ATTACKS` on this Pokémon, no kind named: it stops every event with an
//! effect that the opponent's attacks cause to this Pokémon (counters, conditions, switches, lasting effects), and never
//! the Damage event (APR C-17). It doesn't touch the owner's own attacks or effects (a heal, counters from your own
//! Cofagrigus). Torcherto sets the main damage to 60 plus 20 per Benched Pokémon on both sides.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Skeledirge@SSP",
    // Unaware: prevent all effects of attacks used by the opponent's Pokémon done to this Pokémon.
    passives: &[
        // Every event the opponent's attacks cause to this Pokémon, the switches included (APR C-04 / C-05, id2025,
        // id2155); damage is not an effect.
        Passive { origin: RuleSource::Ability, modifier: Modifier::Prevent(PreventSpec::on(SlotPred::All(&[SlotPred::Holder, SlotPred::IsThisPokemon]), EFFECTS_OF_OPP_ATTACKS)) },
    ],
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::before_damage(damage_is(Num::Add(&Num::Lit(60), &Num::Mul(&Num::Add(&Num::BenchCount(Who::Me), &Num::BenchCount(Who::Opp)), &Num::Lit(20)))))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
