//! Toxtricity (M2 / PFL 68): Sinister Surge — once during your turn, search
//! your deck for a Basic [D] Energy and attach it to 1 of your Benched [D]
//! Pokémon, then shuffle; put 2 damage counters on that Pokémon. Thwap — 100.
//!
//! Fixed (phase 4b, W4): Twinleaf also allowed the Active Pokémon as the target;
//! the card says Benched only.
//!
//! Twinleaf: ABILITY_USED and the once-per-turn marker are set in the prompt
//! callback (so a cancelled prompt still uses the ability); SHUFFLE_DECK runs
//! before the 20 damage is added directly (`target.damage += 20`).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Toxtricity",
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn("BAD_BOOST_MARKER"),
        needs: &[Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any), Cond::AnySlot(SlotSel::Bench(Who::Me), SlotPred::PrintedTypeIs(ct::DARK))],
        // Attach a Basic [D] Energy from your deck to 1 of your Benched [D] Pokémon, then shuffle; put 2 damage counters on that Pokémon.
        steps: &[
            Step::new(Op::Attach(AttachSpec {
                chooser: Who::Me,
                from: ZoneRef(Who::Me, Zone::Deck),
                predicate: Pred::All(&[Pred::BasicEnergy, Pred::Name("Darkness Energy")]),
                slots: AttachSlots::Bench,
                target: Pred::PokemonType(ct::DARK),
                scan: TargetScan::InPlay,
                bounds: Bounds { min: Num::Lit(0), max: Num::Lit(1) },
                same_target: false,
                different_targets: false,
                valid_types: &[],
                max_per_type: 0,
                cancel: true,
                route: AttachRoute::Move,
                none_shuffles: true,
            })),
            Step::new(Op::If(IfSpec {
                cond: Cond::Slot(SlotExpr::Chosen, SlotPred::Any),
                yes: &[
                    Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck) })),
                    Step::new(Op::PlaceCounters(PlaceCountersSpec { target: SlotTarget::Slot(SlotExpr::Chosen), counters: Num::Lit(2), cause: CounterCause::Effect })),
                ],
                no: &[],
            })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
