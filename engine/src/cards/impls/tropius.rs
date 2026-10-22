//! Tropius (PBL 1): Fruity Aroma - look at the top 6 cards of your deck, and you may reveal any number of Pokémon you find there
//! and put them into your hand. Shuffle the other cards back into your deck. Solar Beam - 60.
//!
//! The 6 cards are set aside (`Op::Look`); 0 to all of the Pokémon among them are revealed and put into the hand by one PutIntoHand
//! (cause: this attack), the others go back into the deck (PutIntoDeck), and the deck is shuffled once (Tatsugiri TWM 131).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Tropius",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::after_damage(Op::Look(LookSpec { from: ZoneRef(Who::Me, Zone::Deck), cards: CardSel::Top(Num::Lit(6)), into: 0, ..LookSpec::DEFAULT })),
            Step::after_damage(Op::Pick(PickSpec {
                from: ZoneRef(Who::Me, Zone::Scratch(0)),
                predicate: Pred::Pokemon,
                bounds: Bounds { min: Num::Lit(0), max: Num::Lit(6) },
                into: 1,
                ..PickSpec::DEFAULT
            })),
            Step::after_damage(Op::PutIntoHand(PutIntoHandSpec { from: ZoneRef(Who::Me, Zone::Scratch(0)), cards: CardSel::Chosen(1), reveal: Some(Who::Opp), ..PutIntoHandSpec::DEFAULT })),
            Step::after_damage(Op::PutIntoDeck(PutIntoDeckSpec { from: ZoneRef(Who::Me, Zone::Scratch(0)), cards: CardSel::All, position: DeckPosition::Bottom, ..PutIntoDeckSpec::DEFAULT })),
            Step::after_damage(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
