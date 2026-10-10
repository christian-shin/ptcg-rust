//! Fezandipiti (TWM): Adrena-Pheromone — if this Pokémon has any [D] Energy attached and is damaged by an attack, flip a
//! coin; if heads, prevent that damage. Energy Feather — 30 damage for each Energy attached to this Pokémon.
//!
//! Adrena-Pheromone is a `Prevent` over `Kind(Damage)` with a coin (`PreventSpec::on_coin`, decision D8): at step 6
//! of the damage calculation, after the hard preventions, and only for damage there is (nothing is flipped for 0 damage
//! or damage already prevented). It needs the Ability working and a [D] Energy unit provided on this Pokémon (a Legacy
//! Energy counts, id1969), and the flip belongs to the Pokémon's owner. Energy Feather counts every Energy unit
//! provided on this Pokémon. The text says "damaged by an attack" with no owner; the declaration covers the
//! opponent's attacks only (`DAMAGE_BY_OPP_ATTACKS`).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Fezandipiti",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::before_damage(damage_is(Num::Mul(&Num::EnergyOn(SlotSel::One(SlotExpr::This), EnergyUnit::ProvidedUnits), &Num::Lit(30)))),
        ] },
    ],
    passives: &[
        Passive { origin: RuleSource::Ability, modifier: Modifier::Prevent(PreventSpec::on_coin(SlotPred::All(&[SlotPred::Holder, SlotPred::IsThisPokemon, SlotPred::Provides(ct::DARK)]), DAMAGE_BY_OPP_ATTACKS)) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
