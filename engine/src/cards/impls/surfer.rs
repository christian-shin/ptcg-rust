//! Surfer (SSP / ASC): switch your Active Pokémon with 1 of your Benched
//! Pokémon; if you do, draw cards until you have 5 cards in your hand.
//!
//! Twinleaf: throws SUPPORTER_ALREADY_PLAYED when a Supporter was played;
//! fixed (phase 4b): throws CANNOT_PLAY_THIS_CARD with an empty Bench (the
//! prompt would have no valid answer); moves the card to the supporter list
//! itself and prevents the default;
//! the ChoosePokemonPrompt (no cancel)
//! is followed by `player.switchPokemon(cardList, store, state)` (fixed in
//! phase 4b, R4: it was the silent form without the move effects: Yanmega ex
//! Buzz Boost, Palafin Zero to Hero and the ability-lock order never saw the
//! switch) and a loop of
//! single-card MOVE_CARDS (count 1) until the hand has 5 cards or the deck
//! is empty.
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
