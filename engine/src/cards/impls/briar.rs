//! Briar (SCR): usable only if the opponent has exactly 2 Prize cards left; this turn, if the opponent's Active is
//! Knocked Out by damage from an attack of your Tera Pokémon, take 1 more Prize card.
//!
//! Briar sets a marker on the player; `Modifier::PrizeAdjust` adds the Prize at the KnockOut event (`ko_by` =
//! AttackDamage, the Knocked Out Pokémon is the opponent's Active Pokémon, the Attacking Pokémon is a Tera Pokémon).
//! "During this turn": the marker is cleared at every Pokémon Checkup, so it never reaches a later turn or the
//! opponent's turns.
use crate::spec::prelude::*;

const BRIAR: &str = "BRIAR_EXTRA_PRIZE_MARKER";

pub static SPEC: CardSpec = CardSpec {
    class: "Briar",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[Cond::Cmp(Num::PrizesLeft(Who::Opp), CmpOp::Eq, Num::Lit(2))],
        steps: &[Step::new(Op::SetMarker(SetMarkerSpec { scope: MarkerScope::Player(Who::Me), name: BRIAR, source: RuleSource::TrainerEffect }))],
    }),
    // This turn, if the opponent's Active Pokémon is Knocked Out by damage from an attack of your
    // Tera Pokémon, take 1 more Prize card.
    passives: &[Passive {
        origin: RuleSource::TrainerEffect,
        modifier: Modifier::PrizeAdjust(PrizeAdjustSpec {
            delta: 1,
            subject: SlotPred::IsActive,
            by_attack_damage: true,
            by_own_attack: None,
            guard: Cond::All(&[
                Cond::HasMarker { who: Who::Me, name: BRIAR, from: MarkerFrom::This },
                Cond::AttackerOfKnockOut { who: Who::Opp, pred: Pred::Tag(crate::types::tag::POKEMON_TERA) },
            ]),
            ..PrizeAdjustSpec::DEFAULT
        }),
    }],
    triggers: &[Trigger {
        origin: RuleSource::TrainerEffect,
        event: Event::OnCheckup(OnCheckupSpec {}),
        steps: &[Step::new(Op::ClearMarker(ClearMarkerSpec { scope: MarkerScope::Player(Who::Me), name: BRIAR, from: MarkerFrom::This }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
