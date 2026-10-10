//! Bouffalant (SCR): Curly Wall — if you have any other Bouffalant in play,
//! each of your Basic [C] Pokémon takes 60 less damage from your opponent's
//! attacks (after Weakness and Resistance); only 1 Curly Wall applies.
//! Boundless Power — 130; during your next turn this Pokémon can't attack
//! (THIS_POKEMON_CANNOT_ATTACK_NEXT_TURN: `cannotAttackNextTurnPending` on
//! the Active).
//!
//! Only a Bouffalant in play applies: it needs 2+ Pokémon named Bouffalant in
//! its owner's play, then an ability-lock probe; any PutDamageEffect from
//! the opponent's attack on the owner's Basic [C] Pokémon is reduced, once
//! per effect via the `nonstackingDamageReducers` source 'Curly Wall'.
//!
//! Fixed (phase 4b, W4): Boundless Power's lock was missing, and Curly Wall
//! also reduced the owner's own attacks (self-damage).
use crate::spec::prelude::*;
use crate::types::ct;

pub static SPEC: CardSpec = CardSpec {
    class: "Bouffalant",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_this_pokemon(&CANT_ATTACK, LockUntil::YourNextTurn)) }))] }],
    passives: &[Passive {
        origin: RuleSource::Ability,
        // Each of your Basic [C] Pokémon takes 60 less damage while you have another Bouffalant in play;
        // only 1 Curly Wall applies. Only a Bouffalant in play has the Ability.
        modifier: Modifier::DamageTaken(DamageTakenSpec {
            amount: 60,
            subject: SlotPred::All(&[SlotPred::Basic, SlotPred::PrintedTypeIs(ct::COLORLESS)]),
            side: Side::Owner,
            guard: Cond::Cmp(Num::SlotCount(SlotSel::Pokemon(Who::Me), SlotPred::Named("Bouffalant")), CmpOp::Ge, Num::Lit(2)),
            nonstacking: Some(NonStack::CurlyWall),
            ..DamageTakenSpec::DEFAULT
        }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
