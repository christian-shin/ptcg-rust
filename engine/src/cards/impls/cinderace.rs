//! Cinderace (MEG 28): Explosiveness (setup, core `PLAY_DURING_SETUP` tag). Turbo Flare - 50; search your deck for up to 3
//! Basic Energy cards and attach them to your Benched Pokémon in any way you like, then shuffle your deck.
//!
//! The printed order: the search (an Attach from the deck to the Bench, one Attach event per card; min 0 / max 3, an
//! attack's "up to", id2301), then one shuffle after it (APR E-19, user decision D9). With an empty deck or no Benched
//! Pokémon the attack does its 50 damage and nothing else: an attack stays usable when its effect can't happen (APR D-01;
//! id2313, id361: you can't search a deck you know holds nothing to find).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Cinderace",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::If(IfSpec { cond: Cond::All(&[Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any), Cond::Cmp(Num::BenchCount(Who::Me), CmpOp::Gt, Num::Lit(0))]), yes: &[Step::new(Op::Attach(AttachSpec {
                chooser: Who::Me,
                from: ZoneRef(Who::Me, Zone::Deck),
                predicate: Pred::BasicEnergy,
                slots: AttachSlots::Bench,
                target: SlotPred::Any, scan: TargetScan::InPlay,
                bounds: Bounds { min: Num::Lit(0), max: Num::Lit(3) },
                same_target: false,
                different_targets: false,
                valid_types: &[],
                max_per_type: 0,
                cancel: false,
                onto: None,
                cards: CardSel::All,
             different_types: false, })), Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true }))], no: &[] }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
