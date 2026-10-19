//! Eelektrik (ASC 60 / SV11B): Dynamotor — once during your turn (before your attack), you may attach a [L] Energy card from your
//! discard pile to 1 of your Benched Pokémon. Electric Ball — 50.
//!
//! An Ability: one Attach from the discard pile to a Benched Pokémon, a Basic Lightning Energy card, a real choice (it can be
//! cancelled), exactly one card. The Ability's usage record "of this card" is made when it is used, whether or not a card
//! is attached (id127, id249: an Ability that did nothing is still used), so each Eelektrik uses its own Dynamotor (JP FAQ
//! audit: two Eelektrik each use Dynamotor); it ends with the turn.
use crate::spec::prelude::*;


pub static SPEC: CardSpec = CardSpec {
    class: "Eelektrik",
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn,
        needs: &[],
        steps: &[
            Step::new(Op::Attach(AttachSpec {
                from: ZoneRef(Who::Me, Zone::Discard),
                predicate: Pred::All(&[Pred::BasicEnergy, Pred::Name("Lightning Energy")]),
                slots: AttachSlots::Bench,
                bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) },
                cancel: true,
                ..AttachSpec::DEFAULT
            })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
