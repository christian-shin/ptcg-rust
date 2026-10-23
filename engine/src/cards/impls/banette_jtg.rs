//! Banette (JTG 60): Cursed Words - your opponent chooses 3 cards from their hand and shuffles those cards into their deck.
//! Spooky Shot - 70.
//!
//! Cursed Words: the opponent chooses the cards (exactly 3, or the whole hand when they hold fewer: "chooses 3 cards" asks for as
//! many as they have), they go from their hand into their deck (a PutIntoDeck event, this attack its cause) and their deck is
//! shuffled (APR E-19; Team Rocket's Meowth DRI 149). Nothing happens with an empty hand. This is a different card from Banette PBL 34.
use crate::spec::prelude::*;

const OPP_HAND: ZoneRef = ZoneRef(Who::Opp, Zone::Hand);

pub static SPEC: CardSpec = CardSpec {
    class: "BanetteJTG",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::If(IfSpec {
            cond: Cond::Nonempty(OPP_HAND, Pred::Any),
            yes: &[
                Step::new(Op::Pick(PickSpec { chooser: Who::Opp, from: OPP_HAND, bounds: Bounds { min: Num::Min(&Num::ZoneSize(OPP_HAND), &Num::Lit(3)), max: Num::Min(&Num::ZoneSize(OPP_HAND), &Num::Lit(3)) }, into: 0, ..PickSpec::DEFAULT })),
                Step::new(Op::PutIntoDeck(PutIntoDeckSpec { from: OPP_HAND, cards: CardSel::Chosen(0), position: DeckPosition::Bottom, ..PutIntoDeckSpec::DEFAULT })),
                Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Opp, Zone::Deck), wait: true })),
            ],
            no: &[],
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
