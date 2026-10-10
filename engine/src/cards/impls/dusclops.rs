//! Dusclops (SFA): Cursed Blast — put 5 damage counters on 1 of your
//! opponent's Pokémon, then this Pokémon is Knocked Out. Will-o-Wisp — 50.
//!
//! Twinleaf: no once-per-turn marker; the self-KO is `damage += 999` on the
//! slot holding this card (the checkState reducer then knocks it out).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Dusclops",
    powers: &[PowerSpec {
        index: 0,
        once: Once::No,
        needs: &[],
        steps: &[
            Step::new(Op::PlaceCounters(PlaceCountersSpec { target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::PokemonBenchFirst(Who::Opp), msg: "CHOOSE_POKEMON_TO_DAMAGE" }), counters: Num::Lit(5) })),
            Step::new(Op::KnockOut(KnockOutSpec { target: SlotExpr::This, when: Cond::True })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
