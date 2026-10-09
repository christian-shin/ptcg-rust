//! Repel (SUM / MEG): your opponent switches their Active Pokémon with 1 of
//! their Benched Pokémon.
//!
//! Not playable without an opposing Bench. Rule: a ChangeActive (SwitchOut, APR C-04 specific case: Repel); the
//! opponent chooses among their own Benched Pokémon.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Repel",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[
            Step::new(Op::Switch(SwitchSpec { change: ActiveChange::SwitchOut, among: SwitchAmong::Bench, msg: "CHOOSE_POKEMON_TO_SWITCH", required: true })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
