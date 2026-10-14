//! Kieran (TWM 154 / PRE 113, Supporter): choose 1 - switch your Active Pokémon with 1 of your Benched Pokémon; or during this
//! turn attacks used by your Pokémon do 30 more damage to your opponent's Active Pokémon ex and Active Pokémon V (before
//! applying Weakness and Resistance).
//!
//! The switch option is offered only with a Benched Pokémon; it is a ChangeActive (Switch, APR C-03), which "when this Pokémon
//! moves" Abilities see (Yanmega ex, Palafin). The damage option is an effect on the player (APR C-19: a Trainer's "during this turn"
//! effect), the card's declared `DamageDealt` (+30 to damage to the opponent's Active ex / V only, never to a Benched Pokémon or
//! to the attacker itself, only when the attack does damage; JP FAQ Kieran: it applies to both of Dipplin's attacks; id2428: once
//! for each attack with Festival Lead), put by `Op::Declare` and ended with the turn (the discarded card no longer reacts).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Kieran",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[],
        steps: &[
            Step::new(Op::Choose(ChooseSpec { chooser: Who::Me, options: &[ChoiceBranch { label: "SWITCH_POKEMON", avail: Cond::AnySlot(SlotSel::Bench(Who::Me), SlotPred::Any), body: &[Step::new(Op::Switch(SwitchSpec { change: ActiveChange::Switch, among: SwitchAmong::Bench, required: false }))] }, ChoiceBranch { label: "INCREASE_DAMAGE_BY_30_AGAINST_OPPONENTS_EX_AND_V_POKEMON", avail: Cond::True, body: &[Step::new(Op::Declare(DeclareSpec { effect: 0, on: DeclareOn::Player(Who::Me), until: DeclareUntil::EndOfTurn }))] }] })),
        ],
    }),
    effects: &[
        Passive { origin: RuleSource::TrainerEffect, modifier: Modifier::DamageDealt(DamageDealtSpec { amount: 30, target: SlotPred::Top(Pred::OneOf(&[Pred::PokemonEx, Pred::PokemonV])), needs_damage: true, ..DamageDealtSpec::DEFAULT }) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
