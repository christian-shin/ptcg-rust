//! Cornerstone Mask Ogerpon (DRI 111): Rock Kagura - search your deck for a Basic [F] Energy card and attach it to 1 of your
//! Pokémon. Then, shuffle your deck. Mountain Ramming - 100; discard the top card of your opponent's deck.
//!
//! Rock Kagura: with a card in the deck, one search of 0 or 1 Basic [F] Energy attached to one Pokémon of yours, then the shuffle
//! (APR E-19; Hearthflame Mask Ogerpon DRI 44). Mountain Ramming: a Discard event over the opponent's deck after the damage
//! (Team Rocket's Larvitar DRI 94).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "CornerstoneMaskOgerpon",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[Step::after_damage(Op::If(IfSpec {
                cond: Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any),
                yes: &[
                    Step::new(Op::Search(SearchSpec {
                        pick: PickSpec {
                            from: ZoneRef(Who::Me, Zone::Deck),
                            predicate: Pred::All(&[Pred::BasicEnergy, Pred::Name("Fighting Energy")]),
                            bounds: Bounds { min: Num::Lit(0), max: Num::Lit(1) },
                            ..PickSpec::DEFAULT
                        },
                        destination: SearchDestination::AttachToPicked,
                        cancel: false,
                    })),
                    Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
                ],
                no: &[],
            }))],
        },
        AttackSpec {
            index: 1,
            steps: &[Step::after_damage(Op::Discard(DiscardSpec { from: ZoneRef(Who::Opp, Zone::Deck), cards: CardSel::Top(Num::Lit(1)), ..DiscardSpec::DEFAULT }))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
