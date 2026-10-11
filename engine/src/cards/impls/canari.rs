//! Canari (ASC / M2a): "You can use this card only if you discard another card from your hand." Discard it; search your deck
//! for up to 4 [L] Pokémon, reveal them, put them into your hand. Then, shuffle your deck.
//!
//! The discard is the printed cost (`Step::cost`: payable, not the effect, APR E-30); having another card in hand is the printed
//! condition (`Cond::Printed`). The search is one Search op revealing what it finds, then one shuffle after it, also when
//! nothing was found (APR E-19, id2303); the pick is up to 4 or as many as the deck holds.
use crate::spec::prelude::*;

const DECK: ZoneRef = ZoneRef(Who::Me, Zone::Deck);
const LIGHTNING_POKEMON: Pred = Pred::All(&[Pred::Pokemon, Pred::PrintedType(ct::LIGHTNING)]);

pub static SPEC: CardSpec = CardSpec {
    class: "Canari",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[Cond::Printed(&Cond::NonemptyOther(ZoneRef(Who::Me, Zone::Hand), Pred::Any))],
        steps: &[
            // Discard another card from your hand.
            Step::new(Op::Pick(PickSpec {
                from: ZoneRef(Who::Me, Zone::Hand),
                bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) },
                into: 0,
                msg: "CHOOSE_CARD_TO_DISCARD",
                ..PickSpec::DEFAULT
            })),
            Step::cost(Op::Discard(DiscardSpec { from: ZoneRef(Who::Me, Zone::Hand), cards: CardSel::Chosen(0), ..DiscardSpec::DEFAULT })),
            // Search your deck for up to 4 [L] Pokémon, reveal them, put them into your hand, shuffle.
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec {
                    predicate: LIGHTNING_POKEMON,
                    bounds: Bounds { min: Num::Lit(0), max: Num::Min(&Num::Lit(4), &Num::CardCount(DECK, LIGHTNING_POKEMON)) },
                    ..PickSpec::DEFAULT
                },
                destination: SearchDestination::Hand { reveal: true },
                msg: "",
                cancel: false,
            })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: DECK, wait: true })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
