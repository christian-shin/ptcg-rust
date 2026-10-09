//! Risky Ruins (MEG): whenever any player puts a Basic non-[D] Pokémon onto
//! their Bench during their turn, put 2 damage counters on that Pokémon.
//!
//! Twinleaf checks the target slot is empty when the play effect is reduced
//! (so evolutions never trigger). Fixed in phase 4b (R4): it also requires a
//! Bench slot and the owner's own turn (a Basic put onto the Bench during the
//! opponent's turn, e.g. Dream Ball taken as a Prize card, took counters).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "DangerousRuins",
    passives: &[
        // Automatically active: it can't be announced and used.
        Passive { origin: RuleSource::Stadium, modifier: Modifier::BlockUse(BlockUseSpec::USE_STADIUM) },
    ],
    // Whenever any player puts a Basic non-[D] Pokémon onto their Bench during their turn, put 2
    // damage counters on that Pokémon.
    triggers: &[Trigger {
        origin: RuleSource::Stadium,
        event: Event::OnEnterPlay(OnEnterPlaySpec { method: EnterMethod::PutOnBench { basic: true, not_type: Some(ct::DARK) } }),
        steps: &[Step::new(Op::PlaceCounters(PlaceCountersSpec { target: SlotTarget::Slot(SlotExpr::Picked), counters: Num::Lit(2), cause: CounterCause::Direct }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
