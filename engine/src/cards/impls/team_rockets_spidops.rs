//! Team Rocket's Spidops (DRI): Charging Up - once during your turn, you may attach a Basic Energy card from your discard pile to
//! this Pokémon. Rocket Rush - 30x; 30 damage for each of your Team Rocket's Pokémon in play.
//!
//! The attach is one Attach event of the card chosen from the discard pile onto this Pokémon (a pick of 1, public zone); the Ability
//! is once per turn by its marker.
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
            Step::new(Op::Attach(AttachSpec { from: ZoneRef(Who::Me, Zone::Discard), cards: CardSel::Chosen(0), onto: Some(SlotExpr::This), ..AttachSpec::DEFAULT })),
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
