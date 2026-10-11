//! Hassel (TWM 151): Supporter. You can use this card only if any of your Pokémon were Knocked Out during your
//! opponent's last turn. Look at the top 8 cards of your deck and put up to 3 of them into your hand. Shuffle the other
//! cards back into your deck.
//!
//! The printed condition is `Cond::Printed` (it still binds a Hassel used through Look-Alike Show, D5). The 8 cards are
//! set aside (`Op::Look`); a Supporter's "up to 3" is 1 to 3 (id2301; 0 when used through Look-Alike Show, which may
//! choose a card that would do nothing, APR D-05); the cards taken are one PutIntoHand event, the others go back into the
//! deck (PutIntoDeck) and the deck is shuffled once.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Hassel",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[Cond::Printed(&Cond::KnockedOutLastTurn { who: Who::Me, by_attack_damage: false, tag: None })],
        steps: &[
            Step::new(Op::Look(LookSpec { from: ZoneRef(Who::Me, Zone::Deck), cards: CardSel::Top(Num::Lit(8)), into: 0, ..LookSpec::DEFAULT })),
            Step::new(Op::Pick(PickSpec { from: ZoneRef(Who::Me, Zone::Scratch(0)), bounds: Bounds { min: Num::If(&Cond::TrainerViaAttack, &Num::Lit(0), &Num::Lit(1)), max: Num::Lit(3) }, into: 1, msg: "CHOOSE_CARD_TO_HAND", ..PickSpec::DEFAULT })),
            Step::new(Op::PutIntoHand(PutIntoHandSpec { from: ZoneRef(Who::Me, Zone::Scratch(0)), cards: CardSel::Chosen(1), ..PutIntoHandSpec::DEFAULT })),
            Step::new(Op::PutIntoDeck(PutIntoDeckSpec { from: ZoneRef(Who::Me, Zone::Scratch(0)), cards: CardSel::All, position: DeckPosition::Bottom, ..PutIntoDeckSpec::DEFAULT })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
