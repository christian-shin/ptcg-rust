//! Anthea & Concordia (ASC 182, Supporter): you can use this card only if you have N's Darmanitan, N's Zoroark ex, N's
//! Vanilluxe, N's Klinklang, N's Reshiram, and N's Zekrom in play. During this turn, if your opponent's Active Pokémon is
//! Knocked Out by damage from an attack used by your N's Pokémon, take 3 more Prize cards.
//!
//! The first sentence is a printed condition (`Cond::Printed`): each of the six names is the top card of a Pokémon of the
//! player's in play. The second is Briar's form (APR C-19): an effect on the player declared by `Op::Declare` for the turn, a
//! `PrizeAdjust` of 3 at the KnockOut event of the opponent's Active Pokémon by attack damage whose Attacking Pokémon carries the
//! "N's" tag.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "AntheaConcordia",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[Cond::Printed(&Cond::All(&[
            Cond::InPlay(Who::Me, PlayScope::All, Pred::Name("N's Darmanitan")),
            Cond::InPlay(Who::Me, PlayScope::All, Pred::Name("N's Zoroark ex")),
            Cond::InPlay(Who::Me, PlayScope::All, Pred::Name("N's Vanilluxe")),
            Cond::InPlay(Who::Me, PlayScope::All, Pred::Name("N's Klinklang")),
            Cond::InPlay(Who::Me, PlayScope::All, Pred::Name("N's Reshiram")),
            Cond::InPlay(Who::Me, PlayScope::All, Pred::Name("N's Zekrom")),
        ]))],
        steps: &[Step::new(Op::Declare(DeclareSpec { effect: 0, on: DeclareOn::Player(Who::Me), until: DeclareUntil::EndOfTurn }))],
    }),
    effects: &[Passive {
        origin: RuleSource::TrainerEffect,
        modifier: Modifier::PrizeAdjust(PrizeAdjustSpec {
            delta: 3,
            subject: SlotPred::IsActive,
            by_attack_damage: true,
            by_own_attack: None,
            guard: Cond::AttackerOfKnockOut { who: Who::Opp, pred: Pred::Tag(crate::types::tag::NS) },
            ..PrizeAdjustSpec::DEFAULT
        }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
