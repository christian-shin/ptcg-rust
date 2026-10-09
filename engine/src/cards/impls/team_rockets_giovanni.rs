//! Team Rocket's Giovanni (DRI): switch your Active Team Rocket's Pokémon
//! with 1 of your Benched Team Rocket's Pokémon. If you do, switch in 1 of
//! your opponent's Benched Pokémon to the Active Spot.
//!
//! Playable when the opponent has no Benched Pokémon (only your own switch happens). Rule: each switch is a
//! ChangeActive (your own: Switch, APR C-03; the opponent's: SwitchIn, C-05), and "if you do" is `Cond::Done`. Used
//! as the effect of an attack (Look-Alike Show), the switch-in is the attack's effect, which Mist Energy and the like
//! on the chosen Pokémon prevent (id2025).
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
