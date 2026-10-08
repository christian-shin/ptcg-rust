//! Sacred Ash (FLF): shuffle 5 Pokémon from your discard pile into your deck.
//!
//! Twinleaf: `min = max = min(5, Pokémon in discard)`, cancellable (a cancel
//! ends the effect; the card is still cleaned up as played); the final
//! ShuffleDeckPrompt has no trailing wait.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "SacredAsh@FLF",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[
            // Exactly min(5, Pokémon in the discard pile); the choice can be cancelled.
            Step::new(Op::Pick(PickSpec {
                from: ZoneRef(Who::Me, Zone::Discard),
                predicate: Pred::Pokemon,
                bounds: Bounds { min: Num::Min(&Num::Lit(5), &Num::CardCount(ZoneRef(Who::Me, Zone::Discard), Pred::Pokemon)), max: Num::Min(&Num::Lit(5), &Num::CardCount(ZoneRef(Who::Me, Zone::Discard), Pred::Pokemon)) },
                into: 0,
                cancel: true,
                msg: "CHOOSE_CARD_TO_DECK",
                ..PickSpec::DEFAULT
            })),
            Step::new(Op::If(IfSpec {
                cond: Cond::Chosen(0),
                yes: &[
                    Step::new(Op::Move(MoveSpec { from: ZoneRef(Who::Me, Zone::Discard), to: ZoneRef(Who::Me, Zone::Deck), cards: CardSel::Chosen(0), ..MoveSpec::DEFAULT })),
                    Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
                ],
                no: &[],
            })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
