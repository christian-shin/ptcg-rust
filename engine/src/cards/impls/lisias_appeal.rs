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
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "LisiasAppeal",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[Cond::AnySlot(SlotSel::Bench(Who::Opp), SlotPred::Top(Pred::Basic))],
        steps: &[
            Step::new(Op::Switch(SwitchSpec { side: Who::Opp, chooser: Who::Me, kind: SwitchKind::PlainBasic, msg: "CHOOSE_POKEMON_TO_SWITCH", required: false })),
            Step::new(Op::Conditions(ConditionsSpec { target: SlotExpr::Active(Who::Opp), change: ConditionChange::Add(&[SpecialCondition::Confused]), cause: Cause::Direct, gate: Gate::TrainerTarget, when: Cond::True })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
