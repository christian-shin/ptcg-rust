//! Misty's Gyarados (DRI 49): Splashing Panic - 70×; discard the top 7 cards of your deck, and this attack does 70 damage
//! for each Misty's Pokémon that you discarded in this way. Waterfall - 120.
//!
//! Mega Abomasnow ex MEG 36's Hammer-lanche with Misty's Pokémon (Pokémon cards with the Misty's tag, Misty's Lapras DRI
//! 50's search): the top 7 cards are discarded before the damage and the damage is 70 times those among them.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MistysGyarados",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::before_damage(Op::Discard(DiscardSpec { from: ZoneRef(Who::Me, Zone::Deck), cards: CardSel::Top(Num::Lit(7)), into: Some(0) })),
            Step::before_damage(damage_is(Num::Mul(&Num::CardCount(ZoneRef(Who::Me, Zone::Scratch(0)), Pred::All(&[Pred::Pokemon, Pred::Tag(crate::types::tag::MISTYS)])), &Num::Lit(70)))),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
