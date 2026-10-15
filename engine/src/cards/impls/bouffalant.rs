//! Bouffalant (SCR): Curly Wall — if you have any other Bouffalant in play, each of your Basic [C] Pokémon takes 60 less damage
//! from your opponent's attacks (after Weakness and Resistance); only 1 Curly Wall applies. Boundless Power — 130; during your
//! next turn this Pokémon can't attack.
//!
//! Curly Wall is an Ability `DamageTaken` over the owner's Basic Pokémon that are [C] as the game checks their type (`TypeIs`),
//! guarded by 2 or more Pokémon named Bouffalant in play (a Bouffalant in play only: an Ability lock turns it off); it is
//! non-stacking (`NonStack::SameAbility`: only 1 Curly Wall applies) and never reduces the owner's own attacks' damage. Boundless Power arms a lasting
//! `CANT_ATTACK` lock on this Pokémon until the end of your next turn.
use crate::spec::prelude::*;
use crate::types::ct;

pub static SPEC: CardSpec = CardSpec {
    class: "BouffalantSCR",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_this_pokemon(&CANT_ATTACK, LockUntil::YourNextTurn)) }))] }],
    passives: &[Passive {
        origin: RuleSource::Ability,
        // Each of your Basic [C] Pokémon takes 60 less damage while you have another Bouffalant in play;
        // only 1 Curly Wall applies. Only a Bouffalant in play has the Ability.
        modifier: Modifier::DamageTaken(DamageTakenSpec {
            amount: 60,
            subject: SlotPred::All(&[SlotPred::Basic, SlotPred::TypeIs(ct::COLORLESS)]),
            side: Side::Owner,
            guard: Cond::Cmp(Num::SlotCount(SlotSel::Pokemon(Who::Me), SlotPred::Named("Bouffalant")), CmpOp::Ge, Num::Lit(2)),
            nonstacking: Some(NonStack::SameAbility),
            ..DamageTakenSpec::DEFAULT
        }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
