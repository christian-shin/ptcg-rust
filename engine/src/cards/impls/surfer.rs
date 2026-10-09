//! Surfer (SSP / ASC): switch your Active Pokémon with 1 of your Benched
//! Pokémon; if you do, draw cards until you have 5 cards in your hand.
//!
//! Not playable with an empty Bench (the prompt would have no valid answer). Rule: the switch is a ChangeActive
//! (Switch, APR C-03); "if you do" is `Cond::Done`; then single-card draws until the hand has 5 cards or the deck is
//! empty.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Surfer",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[],
        steps: &[
            Step::new(Op::Switch(SwitchSpec { change: ActiveChange::Switch, among: SwitchAmong::Bench, msg: "CHOOSE_POKEMON_TO_SWITCH", required: true })),
            // "If you do" (the switch happened); the draw does not decide whether the card can be played.
            Step::new(Op::If(IfSpec { cond: Cond::Done, yes: &[Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::UntilHandSize(Num::Lit(5)) }))], no: &[] })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
