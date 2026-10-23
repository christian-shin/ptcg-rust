//! Drednaw (SCR 44): Impervious Shell - prevent all damage done to this Pokémon by attacks from your opponent's Pokémon if that
//! damage is 200 or more. Hard Crunch - 80+; if your opponent's Active Pokémon already has any damage counters on it, this attack
//! does 80 more damage.
//!
//! Impervious Shell is a `Prevent` over the Damage events of the opponent's attacks on this Pokémon whose final amount is at least
//! 200 (after Weakness, Resistance and the effects on the Pokémon, APR D steps 2-5; `EventPred::AmountAtLeast`, id472 / id2197).
//! Hard Crunch reads the damage counters on the Defending Pokémon before this attack does any (Basculin WHT 24).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Drednaw",
    attacks: &[AttackSpec { index: 0, steps: &[Step::before_damage(more_damage_if(80, Cond::Slot(OPP_ACTIVE, SlotPred::Damaged)))] }],
    passives: &[Passive {
        origin: RuleSource::Ability,
        modifier: Modifier::Prevent(PreventSpec::on(SlotPred::All(&[SlotPred::Holder, SlotPred::IsThisPokemon]), EventPred::All(&[DAMAGE_BY_OPP_ATTACKS, EventPred::AmountAtLeast(200)]))),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
