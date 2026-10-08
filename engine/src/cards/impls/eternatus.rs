//! Eternatus (SSP): Dynablast — 10+; 80 more if the opponent's Active is a
//! Pokémon ex. World's End — 230; discard a Stadium in play (to its owner's
//! discard); with no Stadium the damage is set to 0.
//!
//! Fixed (phase 4b, R7F-3; ruling 1589): the Stadium was discarded in the
//! attack handler, before the damage (a Stadium that raises the target's HP
//! or reduces damage was already gone); it is now discarded in
//! AfterAttackEffect, after the damage and before the Knock Out check.
use crate::spec::prelude::*;

const MY_STADIUM: ZoneRef = ZoneRef(Who::Me, Zone::Stadium);
const OPP_STADIUM: ZoneRef = ZoneRef(Who::Opp, Zone::Stadium);
const STADIUM_IN_PLAY: Cond = Cond::Any(&[Cond::Nonempty(MY_STADIUM, Pred::Any), Cond::Nonempty(OPP_STADIUM, Pred::Any)]);

pub static SPEC: CardSpec = CardSpec {
    class: "Eternatus",
    attacks: &[
        // Dynablast: 80 more damage if your opponent's Active Pokémon is a Pokémon ex.
        AttackSpec {
            index: 0,
            steps: &[Step::before_damage(more_damage_if(80, Cond::Slot(OPP_ACTIVE, SlotPred::Tag(crate::types::tag::POKEMON_EX_LOWER))))],
        },
        // World's End: discard a Stadium in play (to its owner's discard pile); with no Stadium the
        // damage is 0.
        AttackSpec {
            index: 1,
            steps: &[
                Step::before_damage(damage_is(Num::If(&STADIUM_IN_PLAY, &Num::Lit(230), &Num::Lit(0)))),
                Step::after_damage(Op::If(IfSpec {
                    cond: Cond::Nonempty(MY_STADIUM, Pred::Any),
                    yes: &[Step::new(Op::Move(MoveSpec { from: MY_STADIUM, to: ZoneRef(Who::Me, Zone::Discard), cards: CardSel::All, ..MoveSpec::DEFAULT }))],
                    no: &[],
                })),
                Step::after_damage(Op::If(IfSpec {
                    cond: Cond::Nonempty(OPP_STADIUM, Pred::Any),
                    yes: &[Step::new(Op::Move(MoveSpec { from: OPP_STADIUM, to: ZoneRef(Who::Opp, Zone::Discard), cards: CardSel::All, ..MoveSpec::DEFAULT }))],
                    no: &[],
                })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
