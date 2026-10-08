//! Team Rocket's Spidops (DRI): Charge Up — once during your turn, attach a
//! Basic Energy from your discard pile to this Pokémon. Rocket Rush — 30
//! damage for each of your Team Rocket's Pokémon in play.
//!
//! Twinleaf: the once-per-turn marker (player marker sourced by this card)
//! is removed at every EndTurnEffect for the ending player; it is only added
//! (with the MOVE_CARDS) once a card is chosen. The discard check precedes
//! the marker check. No ABILITY_USED board effect.
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "TeamRocketsSpidops",
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn("CHARGE_UP_MARKER"),
        needs: &[Cond::Nonempty(ZoneRef(Who::Me, Zone::Discard), Pred::BasicEnergy)],
        // Attach a Basic Energy from your discard pile to this Pokémon.
        steps: &[
            Step::new(Op::Pick(PickSpec { from: ZoneRef(Who::Me, Zone::Discard), predicate: Pred::BasicEnergy, bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) }, into: 0, msg: "CHOOSE_CARD_TO_ATTACH", ..PickSpec::DEFAULT })),
            Step::new(Op::Move(MoveSpec { from: ZoneRef(Who::Me, Zone::Discard), to: ZoneRef(Who::Me, Zone::Attached(SlotExpr::This)), cards: CardSel::Chosen(0), ..MoveSpec::DEFAULT })),
        ],
    }],
    attacks: &[AttackSpec {
        index: 0,
        // Rocket Rush: 30 damage for each of your Team Rocket's Pokémon in play.
        steps: &[Step::before_damage(damage_is(Num::Mul(&Num::InPlayCount(Who::Me, PlayScope::All, Pred::Tag(tag::TEAM_ROCKET)), &Num::Lit(30))))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
