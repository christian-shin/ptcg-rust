//! Gladion's Decisive Battle (M5, supporter): usable only as the last card
//! in your hand. During this turn, attacks used by your Pokémon without a
//! Rule Box do 80 more damage to the opponent's Active (before W/R).
//!
//! Twinleaf: the bonus applies to every DealDamageEffect of a marked player
//! targeting the opponent's Active whose source slot holds no Rule Box card;
//! each copy only honors its own marker.
use crate::spec::prelude::*;

const MARKER: &str = "M5_GLADIONS_DECISIVE_BATTLE";

pub static SPEC: CardSpec = CardSpec {
    class: "GladionsFinalBattle",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        // Usable only as the last card in your hand.
        needs: &[Cond::LastCardInHand(Who::Me)],
        steps: &[Step::new(Op::SetMarker(SetMarkerSpec { scope: MarkerScope::Player(Who::Me), name: MARKER, source: RuleSource::TrainerEffect }))],
    }),
    passives: &[Passive {
        origin: RuleSource::TrainerEffect,
        modifier: Modifier::DamageDealt(DamageDealtSpec {
            amount: 80,
            attacker: SlotPred::Not(&SlotPred::RuleBox),
            guard: Cond::HasMarker { who: Who::Me, name: MARKER, from: MarkerFrom::This },
            ..DamageDealtSpec::DEFAULT
        }),
    }],
    triggers: &[Trigger {
        origin: RuleSource::CardRule,
        event: Event::OnEndTurn(OnEndTurnSpec { whose: Turn::Owner }),
        steps: &[Step::new(Op::ClearMarker(ClearMarkerSpec { scope: MarkerScope::Player(Who::Me), name: MARKER, from: MarkerFrom::This }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
