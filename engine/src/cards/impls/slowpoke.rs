//! Slowpoke (SCR 57): Dangle Tail: put a Pokémon from your discard pile into your hand.
//!
//! The chosen Pokémon goes from the discard pile into the hand (a PutIntoHand event, this attack its cause), shown to the
//! opponent (cards that leave the discard pile for the hand are shown, as RULES.md records). An attack can be used even when
//! its effect can't be carried out (id362): with no Pokémon in the discard pile it does nothing.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Slowpoke@SCR",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::Search(SearchSpec {
            pick: PickSpec { from: ZoneRef(Who::Me, Zone::Discard), predicate: Pred::Pokemon, bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) }, ..PickSpec::DEFAULT },
            destination: SearchDestination::Hand { reveal: true },
            msg: "CHOOSE_CARD_TO_HAND",
            cancel: false,
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
