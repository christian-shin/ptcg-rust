//! Black Belt's Training (PRE / JTG): during this turn, attacks used by your Pokémon do 40 more damage to your opponent's
//! Active Pokémon ex (before Weakness and Resistance).
//!
//! The Supporter puts an effect on the player for the turn (APR C-19): a declared `DamageDealt` bonus at the Deal stage that
//! applies to damage dealt to the opponent's Active Pokémon ex only (never to your own, e.g. recoil), and only when the damage is
//! above 0. It ends with the turn.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "BlackBeltsTraining",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[],
        steps: &[Step::new(Op::Declare(DeclareSpec { effect: 0, on: DeclareOn::Player(Who::Me), until: DeclareUntil::EndOfTurn }))],
    }),
    // During this turn, attacks used by your Pokémon do 40 more damage to your opponent's Active
    // Pokémon ex (before Weakness and Resistance).
    effects: &[Passive {
        origin: RuleSource::TrainerEffect,
        modifier: Modifier::DamageDealt(DamageDealtSpec {
            stage: DamageStage::Deal,
            amount: 40,
            target: SlotPred::Top(Pred::PokemonEx),
            needs_damage: true,
            ..DamageDealtSpec::DEFAULT
        }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
