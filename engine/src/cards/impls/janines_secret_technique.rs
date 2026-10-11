//! Janine's Secret Art (SFA / PRE): choose up to 2 of your [D] Pokémon. For each of those Pokémon, search your deck for a Basic
//! [D] Energy card and attach it to that Pokémon. Then, shuffle your deck. If you attached Energy to your Active Pokémon in
//! this way, it is now Poisoned.
//!
//! The Search is one Attach event per Energy from the deck (caused by the Supporter; each to a different [D] Pokémon, 0 to 2,
//! none required: a search for a kind). The shuffle follows the search (APR E-19), then the Active Pokémon that got an Energy
//! is Poisoned (a Pokémon that can't be affected, like one with Boost Energy Ancient, still gets the Energy: JP FAQ Janine's
//! Secret Art). With an empty deck the card has no effect (id925).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "JaninesSecretTechnique",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[Cond::InPlay(Who::Me, PlayScope::All, Pred::PokemonType(ct::DARK))],
        steps: &[
            Step::new(Op::Attach(AttachSpec { predicate: Pred::All(&[Pred::BasicEnergy, Pred::Name("Darkness Energy")]), slots: AttachSlots::BenchActive, target: Pred::PokemonType(ct::DARK), bounds: Bounds { min: Num::Lit(0), max: Num::Lit(2) }, different_targets: true, ..AttachSpec::DEFAULT })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
            // "If you attached Energy to your Active Pokémon in this way, it is now Poisoned."
            Step::new(Op::If(IfSpec {
                cond: Cond::Slot(SlotExpr::Attached, SlotPred::IsActive),
                yes: &[Step::new(Op::Conditions(ConditionsSpec { target: SlotExpr::Attached, change: ConditionChange::Add(&[SpecialCondition::Poisoned]), when: Cond::True }))],
                no: &[],
            })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
