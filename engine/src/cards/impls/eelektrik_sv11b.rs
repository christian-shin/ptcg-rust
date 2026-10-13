//! Eelektrik (ASC 60 / SV11B): Dynamotor — once during your turn (before your attack), you may attach a [L] Energy card from your
//! discard pile to 1 of your Benched Pokémon. Electric Ball — 50.
//!
//! An Ability: one Attach from the discard pile to a Benched Pokémon, a Basic Lightning Energy card, a real choice (it can be
//! cancelled), exactly one card. It counts as used only when a card was attached (`Cond::Attached`): then the Ability's
//! usage record "of this card" is made (`Op::Used`), so each Eelektrik uses its own Dynamotor (JP FAQ audit: two Eelektrik each
//! use Dynamotor); it ends with the turn.
use crate::spec::prelude::*;


pub static SPEC: CardSpec = CardSpec {
    class: "Eelektrik@ASC",
    // Dynamotor: once during your turn, you may attach a [L] Energy card from your discard pile to
    // 1 of your Benched Pokémon (it counts as used only when an Energy was attached).
    powers: &[PowerSpec {
        index: 0,
        once: Once::No,
        needs: &[Cond::Not(&Cond::Used(UsedSpec::ability(0, UsedBy::Copy)))],
        steps: &[
            Step::new(Op::Attach(AttachSpec {
                from: ZoneRef(Who::Me, Zone::Discard),
                predicate: Pred::All(&[Pred::BasicEnergy, Pred::Name("Lightning Energy")]),
                slots: AttachSlots::Bench,
                bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) },
                cancel: true,
                ..AttachSpec::DEFAULT
            })),
            Step::new(Op::If(IfSpec {
                cond: Cond::Attached,
                yes: &[Step::new(Op::Used(UsedSpec::ability(0, UsedBy::Copy)))],
                no: &[],
            })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
