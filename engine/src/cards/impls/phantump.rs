//! Phantump (CRI 38): Spiteful Evolution - once during your turn, you may use this Ability. Choose a card in your hand that
//! evolves from this Pokémon and put it onto this Pokémon to evolve it. If you do, place 2 damage counters on the Pokémon you
//! evolved in this way. You can't use this Ability during your first turn. Mumble - 10.
//!
//! The card is chosen in the hand (an Evolution card whose "Evolves from" names this Pokémon) and put onto this Pokémon by the
//! effect: an Evolve event on the effect path, which no first-turn or came-into-play-this-turn limit stops (APR C-12; id2349: the
//! Ability can be used the turn Phantump comes into play, but not during the player's first turn of the game, turns 1 and 2).
//! "If you do" is `Cond::Done` of the Evolve (a refused evolving places no counters); the counters are a PlaceCounters event on
//! the evolved Pokémon, this Ability its cause (Dwebble's Ascension is the same Evolve from the deck).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Phantump",
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn,
        // You can't use this Ability during your first turn.
        needs: &[Cond::Printed(&Cond::Cmp(Num::Turn, CmpOp::Gt, Num::Lit(2)))],
        steps: &[
            Step::new(Op::Pick(PickSpec {
                from: ZoneRef(Who::Me, Zone::Hand),
                predicate: Pred::All(&[Pred::Pokemon, Pred::EvolvesFrom("Phantump")]),
                bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) },
                into: 0,
                ..PickSpec::DEFAULT
            })),
            Step::new(Op::Evolve(EvolveSpec { how: EvolveHow::PutOnto { slot: SlotExpr::This, card: 0 } })),
            Step::new(Op::If(IfSpec {
                cond: Cond::Done,
                yes: &[Step::new(Op::PlaceCounters(PlaceCountersSpec { target: SlotTarget::Slot(SlotExpr::This), counters: Num::Lit(2) }))],
                no: &[],
            })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
