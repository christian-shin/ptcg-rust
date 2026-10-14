//! Grimsley's Move (PFL; class GrimsleysGambit): look at the top 7 cards of your deck and put a [D] Pokémon you find there onto
//! your Bench. Shuffle the other cards and put them on the bottom of your deck. You can't use this card during your first
//! turn.
//!
//! "You can't use this card during your first turn" is printed (`Cond::Printed`), so it binds the card even when its effect
//! is used by Look-Alike Show (D5). The Look sets the 7 cards aside (no zone change); the [D] Pokémon (an Evolution too: JP FAQ
//! Grimsley's Move) enters play on the Bench from there, and the other cards go to the bottom of the deck shuffled (a
//! PutIntoDeck, order Shuffled). Played from the hand, a [D] Pokémon among the cards must be taken (the text isn't a "may"; scenarios
//! fix-grimsleys-move-*-r7d); used through Look-Alike Show none is required.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "GrimsleysGambit",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[Cond::Printed(&Cond::Cmp(Num::Turn, CmpOp::Gt, Num::Lit(2))), Cond::BenchSpace(Who::Me)],
        steps: &[
            Step::new(Op::Look(LookSpec { from: ZoneRef(Who::Me, Zone::Deck), cards: CardSel::Top(Num::Lit(7)), into: 0, ..LookSpec::DEFAULT })),
            Step::new(Op::Pick(PickSpec { from: ZoneRef(Who::Me, Zone::Scratch(0)), predicate: Pred::All(&[Pred::Pokemon, Pred::PokemonType(ct::DARK)]), bounds: Bounds { min: Num::If(&Cond::All(&[Cond::Not(&Cond::TrainerViaAttack), Cond::Nonempty(ZoneRef(Who::Me, Zone::Scratch(0)), Pred::All(&[Pred::Pokemon, Pred::PokemonType(ct::DARK)]))]), &Num::Lit(1), &Num::Lit(0)), max: Num::Lit(1) }, into: 1, ..PickSpec::DEFAULT })),
            Step::new(Op::PlayFromZone(PlayFromZoneSpec { cards: 1, who: Who::Me })),
            Step::new(Op::PutIntoDeck(PutIntoDeckSpec { from: ZoneRef(Who::Me, Zone::Scratch(0)), cards: CardSel::All, position: DeckPosition::Bottom, order: DeckOrder::Shuffled, ..PutIntoDeckSpec::DEFAULT })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
