//! Genesect (SFA 40): Ace Canceller — if this Pokémon has a Pokémon Tool
//! attached, your opponent can't play any ACE SPEC cards from their hand.
//! Magnet Blast — 100.
//!
//! Twinleaf: PlayItemEffect / AttachPokemonToolEffect / AttachEnergyEffect /
//! PlayStadiumEffect whose card has the ACE SPEC tag: when this card is the
//! top Pokémon of a slot with a tool on the playing player's opponent's side,
//! the ability-lock probe runs for the *playing* player, and
//! BLOCKED_BY_EFFECT is thrown when it passes. Only cards played from the hand: an Energy or
//! Tool attached by an effect from the deck or discard pile is not blocked.
//!
//! Events batch 3: attaching an ACE SPEC Energy or Tool is the Attach event from the hand, whatever
//! attaches it (the turn's attachment, an Ability, an attack: id25, id230; RULES.md "Locks on playing
//! cards"); Items and Stadiums keep the action form until PlayTrainer (batch 7).
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "Genesect@SFA",
    passives: &[Passive { origin: RuleSource::Ability, modifier: Modifier::BlockUse(BlockUseSpec {
        binds: Binds::Opponent,
        lock: LockDecl {
            actions: &[LockedAction::PlayItem, LockedAction::PlayStadium],
            card: Pred::Tag(tag::ACE_SPEC),
            except: Pred::False,
            error: "BLOCKED_BY_EFFECT",
            forbids: EventPred::All(&[EventPred::Kind(EventKind::Attach), EventPred::Source(RulesZone::Hand), EventPred::Card(Pred::Tag(tag::ACE_SPEC))]),
        },
        while_: &[LockWhile::HasTool],
        ability: true,
    }) }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
