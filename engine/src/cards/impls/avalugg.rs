//! Avalugg (CRI 24): Iceberg Breaker - 60×; discard the top 6 cards of your deck, and this attack does 60 damage for each
//! Basic [W] Energy card you discarded in this way. Frost Stamp - 160.
//!
//! Mega Abomasnow ex MEG 36's Hammer-lanche: the top 6 cards are discarded before the damage (a Discard event caused by the
//! attack) and the damage is 60 times the Basic [W] Energy cards among them (a Special Energy card isn't a Basic Energy
//! card).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Avalugg",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::before_damage(Op::Discard(DiscardSpec { from: ZoneRef(Who::Me, Zone::Deck), cards: CardSel::Top(Num::Lit(6)), into: Some(0) })),
            Step::before_damage(damage_is(Num::Mul(&Num::CardCount(ZoneRef(Who::Me, Zone::Scratch(0)), Pred::All(&[Pred::BasicEnergy, Pred::Provides(ct::WATER)])), &Num::Lit(60)))),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
