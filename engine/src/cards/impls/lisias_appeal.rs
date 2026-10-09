//! Lisia's Appeal (SSP, supporter): switch in 1 of your opponent's Benched
//! Basic Pokémon to the Active Spot. If you do, the new Active Pokémon is now Confused.
//!
//! Not playable unless the opponent has a Benched Basic Pokémon (phase 4b #44). The Confusion is a GainCondition
//! (Slowpoke's Dopey Face prevents it).
//!
//! Rule: "If you do, the new Active Pokémon is now Confused": a switch-in (ChangeActive) that doesn't happen leaves
//! no new Active Pokémon (`Cond::Done`). Used as the effect of an attack (Look-Alike Show), the switch-in is an
//! effect of the attack done to the Benched Pokémon chosen, which Mist Energy and the like prevent (id2025; JP Q&A:
//! Ninetales' Supernatural Shapeshifter with Boss's Orders vs Mist Energy), so nothing is Confused.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "LisiasAppeal",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[Cond::AnySlot(SlotSel::Bench(Who::Opp), SlotPred::Basic)],
        steps: &[
            Step::new(Op::Switch(SwitchSpec { change: ActiveChange::SwitchIn, among: SwitchAmong::BenchBasic, msg: "CHOOSE_POKEMON_TO_SWITCH", required: false })),
            Step::new(Op::Conditions(ConditionsSpec { target: SlotExpr::Active(Who::Opp), change: ConditionChange::Add(&[SpecialCondition::Confused]), gate: Gate::TrainerTarget, when: Cond::Done })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
