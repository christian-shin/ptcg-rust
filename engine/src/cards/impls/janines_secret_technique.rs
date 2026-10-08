//! Janine's Secret Art (SFA): choose up to 2 of your [D] Pokémon. For each of
//! those Pokémon, search your deck for a Basic [D] Energy card and attach it
//! to that Pokémon. Then, shuffle your deck. If you attached Energy to your
//! Active Pokémon in this way, it is now Poisoned.
//!
//! Twinleaf: throws when a Supporter was already played or the deck is empty (phase 4b); the card moves to
//! the Supporter area and the play is prevented; throws without a [D]
//! Pokémon in play; one AttachEnergyPrompt over the deck (Basic Energy named
//! 'Darkness Energy', non-[D] Pokémon blocked, different targets, 0-2, no
//! cancel). With no transfers only a shuffle follows; otherwise each Energy
//! is moved to its target (Poisoning the Active directly) and the shuffle
//! comes last.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "JaninesSecretTechnique",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[Cond::InPlay(Who::Me, PlayScope::All, Pred::PokemonType(ct::DARK))],
        steps: &[
            Step::new(Op::Attach(AttachSpec { predicate: Pred::All(&[Pred::BasicEnergy, Pred::Name("Darkness Energy")]), slots: AttachSlots::BenchActive, target: Pred::PokemonType(ct::DARK), bounds: Bounds { min: Num::Lit(0), max: Num::Lit(2) }, different_targets: true, route: AttachRoute::MovePoisonActive, ..AttachSpec::DEFAULT })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck) })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
