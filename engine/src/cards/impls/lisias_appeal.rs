//! Lisia's Appeal (SSP, supporter): switch in 1 of your opponent's Benched
//! Basic Pokémon to the Active Spot; the new Active Pokémon is now Confused.
//!
//! Fixed (phase 4b #44): `reduceEffect` checks the supporter turn
//! (SUPPORTER_ALREADY_PLAYED) and fails (CANNOT_PLAY_THIS_CARD) unless the
//! opponent has a Benched Basic Pokémon (a Bench of only evolutions used to
//! leave a prompt with no valid answer).
//! SWITCH_IN_OPPONENT_BENCHED_POKEMON with the
//! non-Basic Bench spots blocked (no cancel); `opponent.switchPokemon` with
//! `store, state`; then, unless a TrainerTargetEffect on the new Active is
//! blocked, Confused is added directly (`addSpecialCondition`).
//!
//! Rule: "the new Active Pokémon" exists only when the switch happened: the Pokémon that was Active (the picked
//! slot after the switch) has left the Active Spot. Used as the effect of an attack (Look-Alike Show), a switch the
//! attack-effect preventions stop leaves no new Active Pokémon, so nothing is Confused (id2025).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "LisiasAppeal",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[Cond::AnySlot(SlotSel::Bench(Who::Opp), SlotPred::Basic)],
        steps: &[
            Step::new(Op::Switch(SwitchSpec { side: Who::Opp, chooser: Who::Me, kind: SwitchKind::PlainBasic, msg: "CHOOSE_POKEMON_TO_SWITCH", required: false })),
            Step::new(Op::Conditions(ConditionsSpec { target: SlotExpr::Active(Who::Opp), change: ConditionChange::Add(&[SpecialCondition::Confused]), gate: Gate::TrainerTarget, when: Cond::Slot(SlotExpr::Picked, SlotPred::Not(&SlotPred::IsActive)) })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
