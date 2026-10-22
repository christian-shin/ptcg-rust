//! Wiglett (TEF 47): Lucky Find - search your deck for an Item card, reveal it, and put it into your hand. Then, shuffle your
//! deck. Aqua Bomb - 40; this Pokémon also does 20 damage to itself.
//!
//! Lucky Find: with a card in the deck, one Search of 0 or 1 Item revealed into the hand, then the shuffle (APR E-19; Inkay PBL 51).
//! Aqua Bomb: the recoil is one Damage event on this Pokémon after the main damage (Arrokuda DRI 62).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Wiglett",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[Step::after_damage(Op::If(IfSpec {
                cond: Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any),
                yes: &[
                    Step::new(Op::Search(SearchSpec {
                        pick: PickSpec { predicate: Pred::Item, bounds: Bounds { min: Num::Lit(0), max: Num::Lit(1) }, ..PickSpec::DEFAULT },
                        destination: SearchDestination::Hand { reveal: true },
                        cancel: false,
                    })),
                    Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
                ],
                no: &[],
            }))],
        },
        AttackSpec { index: 1, steps: &[Step::after_damage(self_damage(20))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
