//! Iron Defender (MEG): during your opponent's next turn, all of your [M]
//! Pokémon take 30 less damage from attacks from your opponent's Pokémon.
//!
//! Twinleaf: every Iron Defender instance (in any zone) runs a
//! CheckPokemonTypeEffect on the target of every PutDamageEffect; if the
//! owner of the zone holding that instance has the marker sourced by it and
//! the target is an [M] Pokémon, it takes 30 less, but only in the attack
//! phase, when the target is that owner's Pokémon and the attacker is not
//! (phase 4b: it used to reduce any [M] target, either side, any attacker).
//! Each played copy has its own marker, so reductions stack. The markers (all
//! sources) are removed from the opponent of whoever ends a turn.
use crate::spec::prelude::*;
use crate::types::ct;

const MARKER: &str = "IRON_DEFENDER_MARKER";

pub static SPEC: CardSpec = CardSpec {
    class: "IronDefender",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[Step::new(Op::SetMarker(SetMarkerSpec { scope: MarkerScope::Player(Who::Me), name: MARKER, source: RuleSource::TrainerEffect }))],
    }),
    passives: &[Passive {
        origin: RuleSource::TrainerEffect,
        modifier: Modifier::DamageTaken(DamageTakenSpec {
            amount: 30,
            subject: SlotPred::TypeIs(ct::METAL),
            side: Side::Owner,
            guard: Cond::HasMarker { who: Who::Me, name: MARKER, from: MarkerFrom::This },
            ..DamageTakenSpec::DEFAULT
        }),
    }],
    // The marker lasts through the opponent's next turn.
    triggers: &[Trigger {
        origin: RuleSource::CardRule,
        event: Event::OnEndTurn(OnEndTurnSpec { whose: Turn::Opp }),
        steps: &[Step::new(Op::ClearMarker(ClearMarkerSpec { scope: MarkerScope::Player(Who::Me), name: MARKER, from: MarkerFrom::This }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
