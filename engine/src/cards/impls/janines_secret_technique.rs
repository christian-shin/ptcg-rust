//! Janine's Secret Art (SFA): choose up to 2 of your [D] Pokémon. For each of
//! those Pokémon, search your deck for a Basic [D] Energy card and attach it
//! to that Pokémon. Then, shuffle your deck. If you attached Energy to your
//! Active Pokémon in this way, it is now Poisoned.
//!
//! Twinleaf: throws when a Supporter was already played or the deck is empty (phase 4b); the card moves to
//! the Supporter area and the play is prevented; throws without a [D]
//! Pokémon in play; one AttachEnergyPrompt over the deck (Basic Energy named
//! 'Darkness Energy', non-[D] Pokémon blocked, different targets, 0-2, no
//! cancel), one Attach event per Energy, then the shuffle, then the Poison: the Active Pokémon is now Poisoned when an
//! Energy was attached to it (printed order; `SlotExpr::Attached` is the Active Pokémon once a card went there).
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
                yes: &[Step::new(Op::Conditions(ConditionsSpec { target: SlotExpr::Attached, change: ConditionChange::Add(&[SpecialCondition::Poisoned]), gate: Gate::None, when: Cond::True }))],
                no: &[],
            })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
