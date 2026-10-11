//! Morpeko (TWM): Snack Seek — once during your turn, look at the top card of
//! your deck; you may discard it. Pick and Stick — attach up to 2 Basic
//! Energy cards from your discard pile to your Pokémon in any way you like.
//!
//! Snack Seek: a Look at the top card, then the player's choice (Discard, or PutIntoDeck on top). Twinleaf discarded the card
//! always (its info prompt answered yes): the printed "You may discard that card" is a choice.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Morpeko@TWM",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::If(IfSpec {
            cond: Cond::Nonempty(ZoneRef(Who::Me, Zone::Discard), Pred::BasicEnergy),
            yes: &[Step::new(Op::Attach(AttachSpec {
                from: ZoneRef(Who::Me, Zone::Discard),
                predicate: Pred::BasicEnergy,
                slots: AttachSlots::BenchActive,
                bounds: Bounds { min: Num::Lit(0), max: Num::Lit(2) },
                ..AttachSpec::DEFAULT
            }))],
            no: &[],
        }))],
    }],
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn("SNACK_SEARCH_MARKER"),
        needs: &[],
        // "You may look at the top card of your deck. You may discard that card.": the card is looked at (set aside), then
        // its owner chooses: discarded, or back on top of the deck.
        steps: &[
            Step::new(Op::Look(LookSpec { from: ZoneRef(Who::Me, Zone::Deck), cards: CardSel::Top(Num::Lit(1)), into: 0, ..LookSpec::DEFAULT })),
            Step::new(Op::May(MaySpec {
                asker: Who::Me,
                when: Cond::True,
                msg: "WANT_TO_DISCARD_CARD",
                yes: &[Step::new(Op::Discard(DiscardSpec { from: ZoneRef(Who::Me, Zone::Scratch(0)), ..DiscardSpec::DEFAULT }))],
                no: &[Step::new(Op::PutIntoDeck(PutIntoDeckSpec { from: ZoneRef(Who::Me, Zone::Scratch(0)), cards: CardSel::All, position: DeckPosition::Top, ..PutIntoDeckSpec::DEFAULT }))],
            })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
