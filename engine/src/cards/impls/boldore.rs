//! Boldore (WHT 47): Smack Down - 30+; if your opponent's Active Pokémon has [F] Resistance, this attack does 50 more damage.
//! Power Gem - 90.
//!
//! The bonus reads the printed Resistance of the opponent's Active Pokémon's card (`Pred::HasResistance`, "Pokémon with [F]
//! Resistance", Fletchling POR 66), before the damage.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Boldore",
    attacks: &[AttackSpec { index: 0, steps: &[Step::before_damage(more_damage_if(50, Cond::Slot(OPP_ACTIVE, SlotPred::Top(Pred::HasResistance(ct::FIGHTING)))))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
