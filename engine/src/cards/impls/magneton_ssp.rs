//! Magneton (SSP): Overvolt Discharge — once during your turn, you may attach
//! up to 3 Basic Energy cards from your discard pile to your [L] Pokémon in
//! any way you like. If you use this Ability, this Pokémon is Knocked Out.
//! Electric Ball — 40.
//!
//! "Up to 3" in an Ability takes at least 1 (id2399, id2301); only [L] Pokémon are offered. Each card is an Attach
//! event by the Ability. Using the Ability Knocks this Pokémon Out whatever was attached: `Op::KnockOut` on it is a
//! KnockOut by an effect that waits for the state check with every other Knock Out (D1), the opponent taking the Prize.
//! No once-per-turn marker: the card leaves play.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Magneton@SSP",
    powers: &[PowerSpec {
        index: 0,
        once: Once::No,
        needs: &[],
        steps: &[
            Step::new(Op::Attach(AttachSpec { from: ZoneRef(Who::Me, Zone::Discard), predicate: Pred::BasicEnergy, slots: AttachSlots::BenchActive, target: Pred::PokemonType(ct::LIGHTNING), bounds: Bounds { min: Num::Lit(1), max: Num::Lit(3) }, ..AttachSpec::DEFAULT })),
            Step::new(Op::KnockOut(KnockOutSpec { target: SlotExpr::This, when: Cond::True })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
