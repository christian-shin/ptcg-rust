//! Magneton (SSP): Overvolt Discharge — once during your turn, you may attach
//! up to 3 Basic Energy cards from your discard pile to your [L] Pokémon in
//! any way you like. If you use this Ability, this Pokémon is Knocked Out.
//! Electric Ball — 40.
//!
//! Twinleaf: throws CANNOT_USE_POWER without a basic Energy in the discard;
//! a non-cancellable AttachEnergyPrompt (discard → Bench/Active, basic
//! Energy, min 1 (phase 4b R7E: "up to 3" in an Ability takes at least 1,
//! rulings 1853/1778; it was min 0) max 3, non-[L] Pokémon blocked); MOVE_CARDS each transfer,
//! then `damage += 999` on this card's slot. No once-per-turn marker, no
//! ABILITY_USED.
//!
//! Fixed (phase 4b, R2): with no transfer (0 chosen) Magneton was not Knocked
//! Out; the text says it is Knocked Out whenever the Ability is used.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Magneton@SSP",
    powers: &[PowerSpec {
        index: 0,
        once: Once::No,
        needs: &[],
        steps: &[
            Step::new(Op::Attach(AttachSpec { from: ZoneRef(Who::Me, Zone::Discard), predicate: Pred::BasicEnergy, slots: AttachSlots::BenchActive, target: Pred::PokemonType(ct::LIGHTNING), bounds: Bounds { min: Num::Lit(1), max: Num::Lit(3) }, ..AttachSpec::DEFAULT })),
            Step::new(Op::KnockOut(KnockOutSpec { target: SlotExpr::This, mode: KnockOutMode::Direct, when: Cond::True })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
