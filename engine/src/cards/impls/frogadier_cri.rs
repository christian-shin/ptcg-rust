//! Frogadier (CRI 21): Summoning Jutsu: search your deck for up to 3 Pokémon, reveal them, and put them into your hand.
//! Then, shuffle your deck. Aqua Edge: 50.
//!
//! An empty deck can't be searched (id840: nothing happens, no shuffle); a deck without a Pokémon is searched (nothing
//! found) and shuffled. The chosen Pokémon (0 to 3) are revealed and put into the hand (PutIntoHand, this attack its cause),
//! then the deck is shuffled (APR E-19).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Frogadier@CRI",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::If(IfSpec { cond: Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any), yes: &[Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::Pokemon, bounds: Bounds { min: Num::Lit(0), max: Num::Min(&Num::CardCount(ZoneRef(Who::Me, Zone::Deck), Pred::Pokemon), &Num::Lit(3)) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Hand { reveal: true },
                msg: "",
                cancel: false,
            })), Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true }))], no: &[] }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
