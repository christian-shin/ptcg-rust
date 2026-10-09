//! Team Rocket's Giovanni (DRI): switch your Active Team Rocket's Pokémon
//! with 1 of your Benched Team Rocket's Pokémon. If you do, switch in 1 of
//! your opponent's Benched Pokémon to the Active Spot.
//!
//! Twinleaf: the card goes to the supporter pile (and `rocketSupporter` is
//! set) before the Active / Bench checks throw. Each switch clears the old
//! Active's effects first.
//!
//! Fixed in phase 4b (R4): playable when the opponent has no Benched Pokémon
//! (only your own switch happens; the opponent's is the "if you do" part and is
//! skipped), and both switches dispatch MovedToActive / MovedFromActiveToBench
//! (they were silent: Yanmega ex Buzz Boost, Palafin Zero to Hero and the
//! ability-lock activation order never saw them).
//!
//! R7C: `rocket_supporter` is not set when used as the effect of an attack (ruling 1727).
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "TeamRocketsGiovanni",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[Cond::Slot(MY_ACTIVE, SlotPred::Top(Pred::Tag(tag::TEAM_ROCKET))), Cond::AnySlot(SlotSel::Bench(Who::Me), SlotPred::Top(Pred::Tag(tag::TEAM_ROCKET)))],
        steps: &[
            // Switch your Active Team Rocket's Pokémon with 1 of your Benched Team Rocket's Pokémon...
            Step::new(Op::PickSlot(PickSlotSpec { chooser: Who::Me, among: SlotSel::Filtered(&SlotSel::Bench(Who::Me), SlotPred::Top(Pred::Tag(tag::TEAM_ROCKET))), msg: "CHOOSE_POKEMON_TO_SWITCH" })),
            Step::new(Op::Switch(SwitchSpec { change: ActiveChange::Switch, among: SwitchAmong::Picked, msg: "", required: false })),
            // ...if you do, switch in 1 of your opponent's Benched Pokémon to the Active Spot.
            Step::new(Op::If(IfSpec { cond: Cond::Done, yes: &[Step::new(Op::Switch(SwitchSpec { change: ActiveChange::SwitchIn, among: SwitchAmong::Bench, msg: "CHOOSE_POKEMON_TO_SWITCH", required: false }))], no: &[] })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
