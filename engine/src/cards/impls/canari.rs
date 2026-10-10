//! Canari (ASC / M2a): discard another card from your hand; search your deck
//! for up to 4 [L] Pokémon, reveal them, put them into your hand, shuffle.
//!
//! Twinleaf: `playedCanari` is set on play (before any check) and cleared on
//! every EndTurnEffect for that player; the discard prompt runs on a copy of
//! the hand; the deck prompt blocks everything but [L] Pokémon (max 4 or the
//! number present); ShowCards only when something was taken; the final
//! shuffle prompt has no wait.
use crate::spec::prelude::*;

const DECK: ZoneRef = ZoneRef(Who::Me, Zone::Deck);
const LIGHTNING_POKEMON: Pred = Pred::All(&[Pred::Pokemon, Pred::PrintedType(ct::LIGHTNING)]);

pub static SPEC: CardSpec = CardSpec {
    class: "Canari",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[Cond::NonemptyOther(ZoneRef(Who::Me, Zone::Hand), Pred::Any), Cond::Nonempty(DECK, Pred::Any)],
        steps: &[
            // Discard another card from your hand.
            Step::new(Op::Pick(PickSpec {
                from: ZoneRef(Who::Me, Zone::Hand),
                bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) },
                into: 0,
                msg: "CHOOSE_CARD_TO_DISCARD",
                ..PickSpec::DEFAULT
            })),
            Step::new(Op::Discard(DiscardSpec { from: ZoneRef(Who::Me, Zone::Hand), cards: CardSel::Chosen(0), ..DiscardSpec::DEFAULT })),
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
                shuffle_first: false,
            })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: DECK, wait: true })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
