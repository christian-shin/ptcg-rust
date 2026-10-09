//! Kieran (TWM, supporter): choose 1 — switch your Active with 1 of your
//! Benched Pokémon; or your attacks do 30 more damage to the opponent's
//! Active Pokémon ex / V this turn.
//!
//! Without a Benched Pokémon the switch option is removed from the
//! SelectPrompt. Fixed in phase 4b (R4): the +30 only applies to damage dealt
//! to the opponent's Active Pokémon (Twinleaf added it to bench hits and to
//! self-damage whenever the opponent's Active was ex/V); the switch
//! dispatches MovedToActive / MovedFromActiveToBench (it was a silent board
//! change: Yanmega ex Buzz Boost, Palafin Zero to Hero and the ability-lock
//! activation order never saw it).
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "Kieran",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[],
        steps: &[
            Step::new(Op::Choose(ChooseSpec { chooser: Who::Me, msg: "CHOOSE_OPTION", options: &[ChoiceBranch { label: "SWITCH_POKEMON", avail: Cond::AnySlot(SlotSel::Bench(Who::Me), SlotPred::Any), body: &[Step::new(Op::Switch(SwitchSpec { change: ActiveChange::Switch, among: SwitchAmong::Bench, msg: "CHOOSE_POKEMON_TO_SWITCH", required: false }))] }, ChoiceBranch { label: "INCREASE_DAMAGE_BY_30_AGAINST_OPPONENTS_EX_AND_V_POKEMON", avail: Cond::True, body: &[Step::new(Op::SetMarker(SetMarkerSpec { scope: MarkerScope::Player(Who::Me), name: "KIERAN_MARKER", source: RuleSource::TrainerEffect }))] }] })),
        ],
    }),
    passives: &[
        Passive { origin: RuleSource::TrainerEffect, modifier: Modifier::DamageDealt(DamageDealtSpec { amount: 30, target: SlotPred::OneOf(&[SlotPred::Tag(tag::POKEMON_V), SlotPred::Tag(tag::POKEMON_VSTAR), SlotPred::Tag(tag::POKEMON_VMAX), SlotPred::Tag(tag::POKEMON_EX_LOWER)]), needs_damage: true, guard: Cond::HasMarker { who: Who::Me, name: "KIERAN_MARKER", from: MarkerFrom::This }, ..DamageDealtSpec::DEFAULT }) },
    ],
    triggers: &[
        Trigger { origin: RuleSource::Ability, event: Event::OnEndTurn(OnEndTurnSpec { whose: Turn::Owner }), steps: &[Step::new(Op::ClearMarker(ClearMarkerSpec { scope: MarkerScope::Player(Who::Me), name: "KIERAN_MARKER", from: MarkerFrom::This }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
