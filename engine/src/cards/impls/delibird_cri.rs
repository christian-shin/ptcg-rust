//! Delibird (CRI 18): Pleasing Present - each player may attach up to 3 Basic Energy cards from their hand to their Pokémon in any
//! way they like. Your opponent does this first. Flap - 40.
//!
//! Your opponent chooses first (id2353), then you: two Attach ops in that order, each asked at step D when its player has a Basic
//! Energy card in hand (nothing is asked otherwise); each card is an Attach event caused by this attack, so an attack-effect
//! protection on the Pokémon discards the card instead (id2354). The cards are put after the choices, all at the same time from the
//! game's perspective: no attached-card effect resolves between the two players' attachments (id2353). Choosing none is allowed
//! ("may", up to 3).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "DelibirdCRI",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::after_damage(Op::Attach(AttachSpec {
                chooser: Who::Opp,
                from: ZoneRef(Who::Opp, Zone::Hand),
                predicate: Pred::BasicEnergy,
                slots: AttachSlots::ActiveBench,
                bounds: Bounds { min: Num::Lit(0), max: Num::Lit(3) },
                ..AttachSpec::DEFAULT
            })),
            Step::after_damage(Op::Attach(AttachSpec {
                from: ZoneRef(Who::Me, Zone::Hand),
                predicate: Pred::BasicEnergy,
                slots: AttachSlots::ActiveBench,
                bounds: Bounds { min: Num::Lit(0), max: Num::Lit(3) },
                ..AttachSpec::DEFAULT
            })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
