//! Turtonator (SSP): Fully Singe — discard an Energy from your opponent's
//! Active Pokémon ex. Steaming Stomp — 100.
//!
//! Fully Singe: when the opponent's Active is a Pokémon ex with an Energy
//! card attached, a non-cancellable ChooseCardsPrompt (min 1, max 1) over
//! that Active's Energy, then a DiscardCardsEffect on the chosen card.
//!
//! Fixed (phase 4b, W4): Twinleaf tested `activeCard.cardTag.includes(
//! CardTag.POKEMON_ex)` — the deprecated `cardTag` array, which is empty for
//! every pool Pokémon — so the attack never had an effect. It now uses
//! `hasTag(POKEMON_ex)`. Steaming Stomp costs [F][C][C] (printed data, was [R]).
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "Turtonator",
    attacks: &[AttackSpec {
        index: 0,
        // Fully Singe: discard an Energy from your opponent's Active Pokémon ex.
        steps: &[Step::after_damage(Op::If(IfSpec {
            cond: Cond::Slot(OPP_ACTIVE, SlotPred::Top(Pred::Tag(tag::POKEMON_EX_LOWER))),
            yes: &[Step::new(Op::EnergyChoice(EnergyChoiceSpec {
                from: SlotTarget::Slot(OPP_ACTIVE),
                how: EnergyHow::Cards { min: Num::Lit(1), max: Num::Lit(1), kind: EnergyKind::Any, cancel: false, energies_only: false },
                ..EnergyChoiceSpec::DEFAULT
            }))],
            no: &[],
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
