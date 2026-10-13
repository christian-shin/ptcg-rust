//! Toxtricity (PFL 68): Sinister Surge — once during your turn, search your deck for a Basic [D] Energy card and attach it to 1 of your
//! Benched [D] Pokémon, then shuffle; if you attached Energy in this way, place 2 damage counters on that Pokémon. Gentle Slap — 100.
//!
//! The attach is an Attach event from the deck onto a Benched [D] Pokémon (the type is read after effects, as for Wonder Patch); the
//! 2 damage counters are one PlaceCounters event by the Ability on the player's own Pokémon (Battle Cage doesn't stop it: JP FAQ
//! ストリンダー / バトルコロシアム 「はい、のせます。」; a Pokémon with 20 HP or less left is Knocked Out at the state check).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Toxtricity",
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn("BAD_BOOST_MARKER"),
        needs: &[],
        // Attach a Basic [D] Energy from your deck to 1 of your Benched [D] Pokémon, then shuffle; put 2 damage counters on that Pokémon.
        steps: &[
            Step::new(Op::Attach(AttachSpec {
                chooser: Who::Me,
                from: ZoneRef(Who::Me, Zone::Deck),
                predicate: Pred::All(&[Pred::BasicEnergy, Pred::Name("Darkness Energy")]),
                slots: AttachSlots::Bench,
                target: Pred::PokemonType(ct::DARK),
                scan: TargetScan::EffectiveTypes(&[ct::DARK]),
                bounds: Bounds { min: Num::Lit(0), max: Num::Lit(1) },
                same_target: false,
                different_targets: false,
                valid_types: &[],
                max_per_type: 0,
                cancel: true,
                onto: None,
                cards: CardSel::All,
             different_types: false, })),
            // Then shuffle (whatever was found: APR E-19; user decision D9); if you attached Energy in this way, 2 damage
            // counters on that Pokémon.
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
            Step::new(Op::If(IfSpec {
                cond: Cond::Slot(SlotExpr::Attached, SlotPred::Any),
                yes: &[Step::new(Op::PlaceCounters(PlaceCountersSpec { target: SlotTarget::Slot(SlotExpr::Attached), counters: Num::Lit(2) }))],
                no: &[],
            })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
