//! Cinccino ex (CRI 73): Smooth Coat - if any damage is done to this Pokémon by attacks, flip a coin. If heads, prevent that damage.
//! Energized Slap - 40x; this attack does 40 damage for each Energy attached to this Pokémon.
//!
//! Smooth Coat is a coin `Prevent` over the Damage events of any attack, the owner's own included (id248: "checked to prevent
//! damage done by your own Pokémon's attacks"): the card's owner flips only when there is damage no other prevention stops
//! (Fezandipiti's Adrena-Pheromone; the coin comes before a Tool's reaction to the damage, id1160). Energized Slap counts the Energy
//! this Pokémon provides, in units, before the damage (Leafeon ex's Energy count).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "CinccinoEx",
    attacks: &[AttackSpec { index: 0, steps: &[Step::before_damage(damage_is(Num::Mul(&Num::EnergyOn(SlotSel::One(MY_ACTIVE), EnergyUnit::ProvidedUnits), &Num::Lit(40))))] }],
    passives: &[Passive { origin: RuleSource::Ability, modifier: Modifier::Prevent(PreventSpec::on_coin(SlotPred::All(&[SlotPred::Holder, SlotPred::IsThisPokemon]), DAMAGE_BY_ATTACKS)) }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
