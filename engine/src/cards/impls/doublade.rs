//! Doublade (POR 57): Weaponized Swords - 60x; reveal any number of Honedge, Doublade, and Aegislash from your hand, and this attack
//! does 60 damage for each card you revealed in this way.
//!
//! Before the damage the player chooses any number (0 allowed) of the cards of the hand named exactly Honedge, Doublade or Aegislash
//! (id2320: "Aegislash ex" is not "Aegislash"), they are revealed to the opponent, and the damage is 60 for each of them.
use crate::spec::prelude::*;

const HAND: ZoneRef = ZoneRef(Who::Me, Zone::Hand);
const BLADES: Pred = Pred::OneOf(&[Pred::Name("Honedge"), Pred::Name("Doublade"), Pred::Name("Aegislash")]);

pub static SPEC: CardSpec = CardSpec {
    class: "Doublade",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::before_damage(Op::Pick(PickSpec { from: HAND, predicate: BLADES, bounds: Bounds { min: Num::Lit(0), max: Num::CardCount(HAND, BLADES) }, into: 0, ..PickSpec::DEFAULT })),
            Step::before_damage(Op::Reveal(RevealSpec { cards: RevealWhat::Chosen(0), by: Who::Me, to: Who::Opp, when_empty: false })),
            Step::before_damage(damage_is(Num::Mul(&Num::RegCount(0), &Num::Lit(60)))),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
