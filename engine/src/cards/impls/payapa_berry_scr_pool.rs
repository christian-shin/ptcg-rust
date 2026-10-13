//! Payapa Berry (SCR 141, Pokémon Tool): if the Pokémon this card is attached to is damaged by an attack from your
//! opponent's [P] Pokémon, it takes 60 less damage (after applying Weakness and Resistance), and discard this card.
//!
//! A `DamageTaken` declaration on the holder at step 5 of the damage calculation (APR C-16): the attacker's current type is read
//! through the derived types (`SlotPred::TypeIs`), the damage is floored at 0, and the Tool is discarded (a LeavePlay by its own
//! text, D1) only when the Pokémon was damaged: with no damage taken, or the damage prevented, it stays.
use crate::spec::prelude::*;
use crate::types::ct;

pub static SPEC: CardSpec = CardSpec {
    class: "PayapaBerrySCRPool",
    passives: &[Passive {
        origin: RuleSource::Tool,
        modifier: Modifier::DamageTaken(DamageTakenSpec { amount: 60, subject: SlotPred::Holder, source: SlotPred::TypeIs(ct::PSYCHIC), then_discard: true, ..DamageTakenSpec::DEFAULT }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
