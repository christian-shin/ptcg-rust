//! Team Rocket's Giovanni (DRI): switch your Active Team Rocket's Pokémon
//! with 1 of your Benched Team Rocket's Pokémon. If you do, switch in 1 of
//! your opponent's Benched Pokémon to the Active Spot.
//!
//! Playable when the opponent has no Benched Pokémon (only your own switch happens). Rule: each switch is a
//! ChangeActive (your own: Switch, APR C-03; the opponent's: SwitchIn, C-05), and "if you do" is `Cond::Done`. Used
//! as the effect of an attack (Look-Alike Show), the switch-in is the attack's effect, which Mist Energy and the like
//! on the chosen Pokémon prevent (id2025).
//!
//! Used as the effect of an attack it isn't played, so it isn't "a Team Rocket's Supporter played this turn" (`Cond::PlayedThisTurn`; id2225).
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "TeamRocketsGiovanni",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[Cond::Slot(MY_ACTIVE, SlotPred::Top(Pred::Tag(tag::TEAM_ROCKET)))],
        steps: &[
            // Switch your Active Team Rocket's Pokémon with 1 of your Benched Team Rocket's Pokémon...
            Step::new(Op::PickSlot(PickSlotOp { pick: PickSlotSpec { chooser: Who::Me, among: SlotSel::Filtered(&SlotSel::Bench(Who::Me), SlotPred::Top(Pred::Tag(tag::TEAM_ROCKET))) }, purpose: PromptPurpose::SwitchPokemon })),
            Step::new(Op::Switch(SwitchSpec { change: ActiveChange::Switch, among: SwitchAmong::Picked, required: false })),
            // ...if you do, switch in 1 of your opponent's Benched Pokémon to the Active Spot.
            Step::new(Op::If(IfSpec { cond: Cond::Done, yes: &[Step::new(Op::Switch(SwitchSpec { change: ActiveChange::SwitchIn, among: SwitchAmong::Bench, required: false }))], no: &[] })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
