//! Fidough (SCR 66): Pleasant Aroma - search your deck for a Basic Pokémon and put it onto your Bench. Then, shuffle
//! your deck. Stampede - 10.
//!
//! The Pokémon goes onto the Bench through the EnterPlay event (source the deck, mode effect), then the deck is shuffled
//! once (APR E-19). With an empty deck or a full Bench the attack does nothing (no search, no shuffle).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "FidoughSCRPool",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::after_damage(Op::If(IfSpec { cond: Cond::All(&[Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any), Cond::BenchSpace(Who::Me)]), yes: &[Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::Basic, bounds: Bounds { min: Num::Lit(0), max: Num::Lit(1) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Bench,
                msg: "",
                cancel: true,
            })), Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true }))], no: &[] })),
        ] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
