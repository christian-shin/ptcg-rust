//! Aromatisse (POR 36): Scent Collection - once during your turn, you may use this Ability. Search your deck for up to 2 Basic [P]
//! Energy cards, reveal them, and put them into your hand. Then, shuffle your deck. Draining Kiss - 50; heal 30 damage from this
//! Pokémon.
//!
//! An Ability (once per turn for each copy); the search may find nothing (APR E-19) and the Ability is used up either way (id127,
//! Ethan's Quilava). The Energy is revealed and goes into the hand (a PutIntoHand event, this Ability its cause), then the deck is
//! shuffled. Draining Kiss heals this Pokémon after the damage (Tangrowth MEG 7).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Aromatisse",
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn,
        needs: &[],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec { predicate: Pred::All(&[Pred::BasicEnergy, Pred::Name("Psychic Energy")]), bounds: Bounds { min: Num::Lit(0), max: Num::Lit(2) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Hand { reveal: true },
                cancel: true,
            })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
        ],
    }],
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(heal_active(30))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
