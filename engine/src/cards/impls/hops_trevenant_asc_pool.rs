//! Hop's Trevenant (ASC 96): Horrifying Revenge - 30+; 100 more if any of your Hop's Pokémon were Knocked Out by damage from an
//! attack during your opponent's last turn. Corner - 90; during your opponent's next turn the Defending Pokémon can't retreat.
//!
//! Horrifying Revenge reads `Cond::KnockedOutLastTurn` with the Hop's tag and "by attack damage" (a Knock Out by Poison or by an
//! effect doesn't count). Corner arms a lasting `CANT_RETREAT` lock on the Defending Pokémon (an ApplyEffect event) until the end
//! of the opponent's next turn.
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "HopsTrevenantASCPool",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::before_damage(more_damage_if(100, Cond::KnockedOutLastTurn { who: Who::Me, by_attack_damage: true, tag: Some(tag::HOPS) })),
        ] },
        AttackSpec { index: 1, steps: &[
            Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_defending(&CANT_RETREAT)) })),
        ] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
