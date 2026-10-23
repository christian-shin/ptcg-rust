//! Erika's Tangela (ASC 7): Gathering of Blossoms - once during your turn, you may use this Ability. Search your deck for an
//! Erika's Pokémon, reveal it, and put it into your hand. Then, shuffle your deck. Bind - 50; flip a coin. If heads, your
//! opponent's Active Pokémon is now Paralyzed.
//!
//! One Search revealing what it finds (min 0: a hidden zone may be searched without finding), then one shuffle (APR E-19); the
//! Ability needs no card in the deck to be used (Cynthia's Gabite DRI). Bind: the coin is flipped after the damage.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "ErikasTangela",
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn,
        needs: &[],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec { predicate: Pred::All(&[Pred::Pokemon, Pred::Tag(crate::types::tag::ERIKAS)]), bounds: Bounds { min: Num::Lit(0), max: Num::Lit(1) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Hand { reveal: true },
                cancel: false,
            })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
        ],
    }],
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Coin(CoinSpec { heads: &[Step::new(inflict(&[SpecialCondition::Paralyzed]))], ..CoinSpec::DEFAULT }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
