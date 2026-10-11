//! Genesect (SFA 40): ACE Nullifier - if this Pokémon has a Pokémon Tool attached, your opponent can't play any ACE SPEC
//! cards from their hand. Magnetic Blast - 100.
//!
//! A lock on the opponent while this Pokémon has a Tool and its Ability: over an ACE SPEC Trainer played from the hand
//! (its PlayTrainer, `Use(Played) & Source(Hand)`) and over an ACE SPEC card attached from the hand (the Attach event,
//! whatever attaches it: the turn's attachment, an Ability, an attack; id25, id230; RULES.md "Cards moving between zones;
//! Trainer plays; turn actions"). An ACE SPEC card attached by an effect from the deck or the discard pile, or a Supporter
//! used through Look-Alike Show (not played, id2226), isn't covered.
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "Genesect@SFA",
    passives: &[Passive { origin: RuleSource::Ability, modifier: Modifier::BlockUse(BlockUseSpec {
        binds: Binds::Opponent,
        lock: LockDecl::on(
            EventPred::All(&[
                EventPred::Any(&[
                    EventPred::All(&[EventPred::Kind(EventKind::PlayTrainer), EventPred::Use(TrainerUse::Played), EventPred::Source(RulesZone::Hand)]),
                    EventPred::All(&[EventPred::Kind(EventKind::Attach), EventPred::Source(RulesZone::Hand)]),
                ]),
                EventPred::Card(Pred::Tag(tag::ACE_SPEC)),
            ]),
            "BLOCKED_BY_EFFECT",
        ),
        while_: &[LockWhile::HasTool],
        ability: true,
    }) }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
